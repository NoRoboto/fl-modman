//! The app: the workspace, background jobs (long operations never block the UI thread), tabs,
//! the dismissable banner, the confirm dialog and the frame loop.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::time::{Duration, Instant};

use eframe::egui::{CentralPanel, Context, Frame, Margin, Panel, RichText, Ui};
use fl_core::Result;
use fl_core::base::Snapshot;
use fl_core::build::{Conflict, build};
use fl_core::error::{Error, input};
use fl_core::manifest::Package;
use fl_core::target::Target;
use fl_core::workspace::{Status, Workspace};

use crate::add::Stage;
use crate::input::{Action, Input};
use crate::nav::Nav;
use crate::ui::{self, A, B, Hint, S1, S2, S3, S4, SURFACE, WARNING, Y};
use crate::widgets::NumPicker;
use crate::{add, history, mods, pick, settings, storage};

/// What the mod list needs to compute conflicts without touching the disk.
#[derive(Clone)]
pub struct Cache {
    pub t: Target,
    pub base: Snapshot,
    pub pkgs: BTreeMap<String, Package>,
}

impl Cache {
    pub fn load(ws: &Workspace) -> Result<Self> {
        let (t, base, pkgs) = (ws.t.clone(), ws.base()?, ws.packages()?);
        Ok(Cache { t, base, pkgs })
    }

    /// What the base already carries (hand-installed before fl-modman, Q8), as (name, type):
    /// one line per livecpk folder, mapped stadium team and ReShade.
    pub fn installed(&self) -> Vec<(String, &'static str)> {
        use fl_core::codec::{map_teams::MapTeams, sider_ini::SiderIni};
        let doc = |p: &str| self.base.docs.get(p).map(|d| d.text.as_str()).unwrap_or("");
        let mut out: Vec<(String, &str)> = SiderIni::parse(doc(&self.t.sider_ini))
            .entries("cpk.root")
            .iter()
            .filter_map(|e| e.value.trim_end_matches('\\').rsplit('\\').next())
            .filter(|name| !name.eq_ignore_ascii_case("root"))
            .map(|name| (name.to_owned(), "Livecpk"))
            .collect();
        let teams = MapTeams::parse(doc(&self.t.map_teams)).rows();
        let teams = teams.iter().filter_map(|r| r.fields.get(2));
        out.extend(teams.map(|n| (n.trim().to_owned(), "Stadium")));
        if self.base.files.contains_key("dxgi.dll") {
            out.push(("ReShade (dxgi.dll)".into(), "Loose"));
        }
        out
    }

    /// "Livecpk · 12 file(s)": the type a managed mod looks like and the files it puts.
    pub fn meta(&self, id: &str) -> String {
        let Some(p) = self.pkgs.get(id) else {
            return String::new();
        };
        let (m, puts) = (&p.manifest, &p.puts);
        let kind = match () {
            _ if !m.map_teams.is_empty() => "Stadium",
            _ if !m.lua_module.is_empty() => "Lua",
            _ if !m.cpk_root.is_empty() => "Livecpk",
            _ if puts.keys().all(|k| k.starts_with("download/")) && !puts.is_empty() => "Download",
            _ => "Loose",
        };
        format!("{kind} · {} file(s)", puts.len())
    }

    /// The mod id, with its display name when that differs.
    pub fn label(&self, id: &str) -> String {
        let name = self.pkgs.get(id).and_then(|p| p.manifest.name.as_deref());
        match name.filter(|n| *n != id) {
            Some(n) => format!("{id}   ({n})"),
            None => id.into(),
        }
    }
}

/// What a finished background job does to the app (on the UI thread).
pub type Then = Box<dyn FnOnce(&mut App) + Send>;

/// A job result that only reports `msg` on the banner.
pub fn done(msg: String) -> Result<Then> {
    Ok(Box::new(move |app: &mut App| app.info(msg)))
}

fn opened(ws: Workspace) -> Result<Then> {
    let cache = Arc::new(Cache::load(&ws)?);
    Ok(Box::new(move |app: &mut App| {
        (app.ws, app.cache) = (Some(ws), Some(cache));
        app.goto(Tab::Mods);
    }))
}

pub struct Job<T> {
    label: String,
    started: Instant,
    rx: Receiver<Result<T>>,
}

impl<T: Send + 'static> Job<T> {
    pub fn spawn(
        ctx: &Context,
        label: &str,
        f: impl FnOnce() -> Result<T> + Send + 'static,
    ) -> Self {
        let (tx, rx) = channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(f());
            ctx.request_repaint();
        });
        let (label, started) = (label.into(), Instant::now());
        Job { label, started, rx }
    }

    /// The result once the job ended; a job that died without one reports an error.
    pub fn poll(&self) -> Option<Result<T>> {
        match self.rx.try_recv() {
            Ok(r) => Some(r),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(input("the background task stopped"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Mods,
    Add,
    History,
    Settings,
    Storage,
}

const TABS: [(Tab, &str); 5] = [
    (Tab::Mods, "My mods"),
    (Tab::Add, "Add mod"),
    (Tab::History, "History"),
    (Tab::Settings, "Game settings"),
    (Tab::Storage, "Storage"),
];

/// What a confirmed dialog does.
pub enum Act {
    Init(PathBuf, Option<PathBuf>),
    Open(PathBuf),
    Apply,
    Rollback(u32),
    SaveSettings,
    /// The listed mod ids and orphan folders, and whether staging/ was listed.
    Free(Vec<String>, bool),
    Quit,
}

pub struct Confirm {
    title: String,
    body: String,
    yes: String,
    act: Act,
    back_focus: usize,
}

/// `$HOME`, where detection and the folder browsers start.
pub fn home() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
}

#[derive(Default)]
pub struct App {
    pub ctx: Context,
    pub ws_override: Option<PathBuf>,
    scale: f32,
    pub ws: Option<Workspace>,
    pub cache: Option<Arc<Cache>>,
    pub tab: Tab,
    pub nav: Nav,
    input: Input,
    busy: Option<Job<Then>>,
    analysis: Option<Job<Vec<Conflict>>>,
    /// The list changed since the last analysis started.
    dirty: bool,
    pub conflicts: Vec<Conflict>,
    banner: Option<(String, bool)>,
    confirm: Option<Confirm>,
    pub drift: Option<Vec<String>>,
    pub pending: Option<u32>,
    pub pick: pick::Pick,
    pub mods: mods::Mods,
    pub add: add::Add,
    pub history: history::History,
    pub settings: settings::Settings,
    pub storage: storage::Storage,
}

impl App {
    pub fn new(ctx: Context, ws_override: Option<PathBuf>, scale: f32) -> Self {
        let mut app = App {
            ctx,
            ws_override,
            scale,
            ..Default::default()
        };
        match app.ws_override.clone() {
            Some(d) if d.join("state.toml").is_file() => app.open(d),
            _ => pick::detect(&mut app),
        }
        app
    }

    pub fn run(&mut self, label: &str, f: impl FnOnce() -> Result<Then> + Send + 'static) {
        self.busy = Some(Job::spawn(&self.ctx, label, f));
    }

    /// Runs `f` on a fresh copy of the workspace (state is saved on every change).
    pub fn with_ws(
        &mut self,
        label: &str,
        f: impl FnOnce(&mut Workspace) -> Result<Then> + Send + 'static,
    ) {
        if let Some(dir) = self.ws.as_ref().map(|w| w.dir.clone()) {
            self.run(label, move || f(&mut Workspace::open(&dir)?));
        }
    }

    pub fn open(&mut self, dir: PathBuf) {
        self.run("Loading the mod list…", move || {
            opened(Workspace::open(&dir)?)
        });
    }

    /// The workspace of a game: `-w` / `$FL_MODMAN`, else `<game>/../fl-modman` (as the CLI).
    pub fn ws_dir_for(&self, game: &Path) -> PathBuf {
        let game = std::fs::canonicalize(game).unwrap_or(game.into());
        let next_to = game.parent().unwrap_or(&game).join("fl-modman");
        self.ws_override.clone().unwrap_or(next_to)
    }

    pub fn ask(&mut self, title: &str, body: String, yes: &str, act: Act) {
        let (title, yes, back_focus) = (title.into(), yes.into(), self.nav.focus);
        self.confirm = Some(Confirm {
            title,
            body,
            yes,
            act,
            back_focus,
        });
        self.nav.reset();
    }

    pub fn info(&mut self, msg: impl Into<String>) {
        self.banner = Some((msg.into(), false));
    }

    pub fn error(&mut self, msg: impl Into<String>) {
        self.banner = Some((msg.into(), true));
    }

    /// Saves the mod list after an edit and schedules a conflict analysis.
    pub fn changed(&mut self) {
        if let Some(Err(e)) = self.ws.as_ref().map(Workspace::save) {
            self.error(e.to_string());
        }
        self.dirty = true;
    }

    pub fn goto(&mut self, tab: Tab) {
        self.tab = tab;
        self.nav.reset();
        self.load_tab();
        // Measured on open only: `load_tab` also runs after every job.
        if tab == Tab::Storage {
            storage::load(self);
        }
    }

    fn load_tab(&mut self) {
        match self.tab {
            Tab::History => history::load(self),
            Tab::Settings => settings::load(self),
            Tab::Mods | Tab::Add | Tab::Storage => {}
        }
    }

    fn finish(&mut self, r: Result<Then>) {
        self.nav.reset();
        match r {
            Ok(then) => then(self),
            Err(Error::Drift(files)) => {
                self.drift = Some(files);
                self.tab = Tab::Mods;
            }
            Err(Error::AmbiguousRoot(roots)) => self.add.stage = add::Stage::Roots(roots),
            Err(Error::NeedsTeam) => self.add.stage = add::Stage::Team(NumPicker::new(5, 0)),
            Err(e) => self.error(e.to_string()),
        }
        self.refresh();
    }

    /// Re-reads the workspace from disk after a job and checks for an interrupted apply.
    fn refresh(&mut self) {
        let Some(dir) = self.ws.as_ref().map(|w| w.dir.clone()) else {
            return;
        };
        match Workspace::open(&dir) {
            Ok(ws) => {
                let hist = ws.history().unwrap_or_default();
                let mut pending = hist.iter().filter(|g| g.1.status == Status::InProgress);
                self.pending = pending.next_back().map(|g| g.0);
                self.ws = Some(ws);
            }
            Err(e) => self.error(e.to_string()),
        }
        self.dirty = true;
        self.load_tab();
    }

    fn act(&mut self, act: Act) {
        match act {
            Act::Init(game, prefix) => {
                let dir = self.ws_dir_for(&game);
                let label = "Scanning the game: the first run hashes every file (~78 GB can take \
                             several minutes)…";
                self.run(label, move || opened(Workspace::init(&dir, &game, prefix)?));
            }
            Act::Open(dir) => self.open(dir),
            Act::Apply => self.with_ws("Applying the mods…", |ws| {
                let list = ws.enabled();
                let n = ws.apply(&list, "apply", true, true)?.1.unwrap_or(0);
                done(format!("Applied as generation {n}."))
            }),
            Act::Rollback(n) => self.with_ws("Going back…", move |ws| {
                let m = ws.rollback(n, true)?.1.unwrap_or(0);
                let msg = format!("Back to generation {n} (recorded as generation {m}).");
                done(msg)
            }),
            Act::SaveSettings => settings::save(self),
            Act::Free(ids, staging) => storage::free(self, ids, staging),
            Act::Quit => self.ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    /// Recomputes the conflicts in the background whenever the list changed.
    fn analyze(&mut self) {
        if let Some(r) = self.analysis.as_ref().and_then(Job::poll) {
            self.analysis = None;
            match r {
                Ok(c) if !self.dirty => self.conflicts = c,
                Ok(_) => {}
                Err(e) => self.error(e.to_string()),
            }
        }
        if let (true, None, Some(ws), Some(c)) = (self.dirty, &self.analysis, &self.ws, &self.cache)
        {
            let (c, list) = (c.clone(), ws.enabled());
            self.dirty = false;
            self.analysis = Some(Job::spawn(&self.ctx, "", move || {
                Ok(build(&c.t, &c.base, &c.pkgs, &list)?.conflicts)
            }));
        }
    }

    fn tabs(&mut self, ui: &mut Ui) {
        let i = TABS.iter().position(|t| t.0 == self.tab).unwrap_or(0);
        let mut to = None;
        if self.nav.take(Action::PrevTab) {
            to = Some(TABS[(i + TABS.len() - 1) % TABS.len()].0);
        }
        if self.nav.take(Action::NextTab) {
            to = Some(TABS[(i + 1) % TABS.len()].0);
        }
        ui.horizontal(|ui| {
            ui.weak("L1");
            for (t, title) in TABS {
                ui.add_space(S1);
                if ui.selectable_label(self.tab == t, title).clicked() {
                    to = Some(t);
                }
            }
            ui.add_space(S1);
            ui.weak("R1");
        });
        ui.add_space(S2);
        if let Some(t) = to {
            self.goto(t);
        }
    }

    fn banner(&mut self, ui: &mut Ui) {
        let Some((msg, is_error)) = &self.banner else {
            return;
        };
        let mut close = self.nav.take(Action::Back);
        let (icon, color) = match is_error {
            true => ("⚠", WARNING),
            false => ("ℹ", ui.visuals().text_color()),
        };
        let frame = Frame::new().fill(SURFACE).corner_radius(ui::RADIUS);
        frame.inner_margin(S2).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new(format!("{icon} {msg}")).color(color));
                close |= ui.small_button("Dismiss").clicked();
            });
        });
        ui.add_space(S2);
        if close {
            self.banner = None;
        }
    }

    fn confirm(&mut self, ui: &mut Ui) {
        let Some(c) = &self.confirm else {
            return;
        };
        let back = self.nav.take(Action::Back);
        ui.heading(&c.title);
        ui.add_space(S2);
        ui::body(ui, |ui| ui::card(ui, "", |ui| ui.monospace(&c.body)));
        // Deleting files starts on Cancel.
        let focus = usize::from(!matches!(c.act, Act::Free(..)));
        let pressed = ui::actions_focus(ui, &mut self.nav, &["Cancel", &c.yes], focus);
        let yes = pressed == Some(1);
        if yes || back || pressed.is_some() {
            let c = self.confirm.take().expect("confirm shown");
            self.nav.set_focus(c.back_focus);
            if yes {
                self.act(c.act);
            }
        }
    }

    /// The controls of the current screen, for the hint bar.
    pub fn hints(&self) -> Vec<Hint> {
        let (ud, lr) = ("⏶⏷", "◀▶");
        let mut h: Vec<Hint> = match (&self.ws, self.tab) {
            _ if self.busy.is_some() => return Vec::new(),
            _ if self.confirm.is_some() => return vec![(A, "Select"), (B, "Cancel")],
            (None, _) if self.pick.browsing() => vec![(A, "Open"), (B, "Up one folder")],
            (None, _) => vec![(A, "Select"), (B, "Quit")],
            (_, Tab::Mods) if self.pending.is_some() => vec![(A, "Select"), (B, "Quit")],
            (_, Tab::Mods) if self.drift.is_some() => vec![(A, "Select"), (B, "Close")],
            (_, Tab::Mods) if self.mods.moving => vec![(ud, "Carry"), ("[A]/[B]/[Y]", "Drop")],
            (_, Tab::Mods) => vec![("[A]/[X]", "Toggle"), (Y, "Move"), (B, "Quit")],
            (_, Tab::Settings | Tab::Add) if self.editing_digits() => {
                vec![(lr, "Digit"), (ud, "Change"), (A, "Done")]
            }
            (_, Tab::Add) => match self.add.stage {
                Stage::Browse => vec![(A, "Open"), (B, "Up one folder")],
                Stage::Preview(_) => vec![(A, "Select"), (B, "Back")],
                _ => vec![(A, "Select"), (B, "Cancel")],
            },
            (_, Tab::History) => vec![(A, "Go back"), (B, "Quit")],
            (_, Tab::Settings) => vec![("[A]/◀▶", "Change"), (B, "Quit")],
            (_, Tab::Storage) => vec![(A, "Select"), (B, "Quit")],
        };
        if self.banner.is_some() {
            h.retain(|k| k.0 != B);
            h.push((B, "Dismiss"));
        }
        if self.ws.is_some() {
            h.push(("L1/R1", "Tabs"));
        }
        h
    }

    fn editing_digits(&self) -> bool {
        let team = matches!(&self.add.stage, Stage::Team(p) if p.editing);
        team && self.tab == Tab::Add || self.settings.editing() && self.tab == Tab::Settings
    }

    fn frame(&mut self, ui: &mut Ui) {
        if let Some(job) = &self.busy {
            ui.add_space(ui.available_height() / 3.0);
            ui.vertical_centered(|ui| {
                ui.add(egui::Spinner::new().size(S4));
                ui.add_space(S2);
                ui.label(&job.label);
                let s = job.started.elapsed().as_secs();
                ui.weak(format!("{}:{:02}", s / 60, s % 60));
            });
            self.ctx.request_repaint_after(Duration::from_millis(500));
            return;
        }
        if self.confirm.is_some() {
            return self.confirm(ui);
        }
        if self.ws.is_some() {
            self.tabs(ui);
        }
        self.banner(ui);
        match (&self.ws, self.tab) {
            (None, _) => pick::show(self, ui),
            (_, Tab::Mods) => mods::show(self, ui),
            (_, Tab::Add) => add::show(self, ui),
            (_, Tab::History) => history::show(self, ui),
            (_, Tab::Settings) => settings::show(self, ui),
            (_, Tab::Storage) => storage::show(self, ui),
        }
        // B that no screen used: ask before quitting (handhelds have no window close button).
        if self.nav.take(Action::Back) {
            let body = "The mod list is saved; unapplied changes stay pending.".into();
            self.ask("Quit fl-modman?", body, "Quit", Act::Quit);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if self.ctx.pixels_per_point() != self.scale {
            self.ctx.set_pixels_per_point(self.scale);
        }
        let actions = self.input.poll(&self.ctx);
        if self.input.gilrs.is_some() {
            self.ctx.request_repaint_after(Duration::from_millis(30));
        }
        if let Some(r) = self.busy.as_ref().and_then(Job::poll) {
            self.busy = None;
            self.finish(r);
        }
        self.analyze();
        let busy = self.busy.is_some();
        self.nav.begin(if busy { Vec::new() } else { actions });
        let hints = self.hints();
        let bar = Frame::new()
            .fill(SURFACE)
            .inner_margin(Margin::symmetric(S3 as i8, S1 as i8));
        Panel::bottom("hints")
            .frame(bar)
            .resizable(false)
            .show_separator_line(false)
            .show(ui, |ui| ui.small(ui::hint_text(&hints)));
        // Top 40: the Ally's Gaming Mode overlay covers the first pixels.
        let margin = Margin {
            left: S3 as i8,
            right: S3 as i8,
            top: 40,
            bottom: S2 as i8,
        };
        let panel = Frame::central_panel(ui.style()).inner_margin(margin);
        CentralPanel::default()
            .frame(panel)
            .show(ui, |ui| self.frame(ui));
        self.nav.end();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::hint_text;

    #[test]
    fn the_hint_bar_lists_the_controls_of_each_screen() {
        let mut app = App::default();
        let hints = |app: &App| hint_text(&app.hints());
        assert_eq!(hints(&app), "[A] Select    [B] Quit");
        let (dir, t, state) = (PathBuf::new(), Target::fl27(), Default::default());
        app.ws = Some(Workspace { dir, t, state });
        let tabs = "    L1/R1 Tabs";
        assert_eq!(
            hints(&app),
            format!("[A]/[X] Toggle    [Y] Move    [B] Quit{tabs}")
        );
        app.mods.moving = true;
        assert_eq!(hints(&app), format!("⏶⏷ Carry    [A]/[B]/[Y] Drop{tabs}"));
        app.tab = Tab::Add;
        assert_eq!(hints(&app), format!("[A] Open    [B] Up one folder{tabs}"));
        let mut picker = NumPicker::new(5, 0);
        picker.editing = true;
        app.add.stage = Stage::Team(picker);
        let digits = format!("◀▶ Digit    ⏶⏷ Change    [A] Done{tabs}");
        assert_eq!(hints(&app), digits);
        app.tab = Tab::Settings;
        assert_eq!(hints(&app), format!("[A]/◀▶ Change    [B] Quit{tabs}"));
        app.tab = Tab::Storage;
        assert_eq!(hints(&app), format!("[A] Select    [B] Quit{tabs}"));
        app.banner = Some(("saved".into(), false));
        assert_eq!(hints(&app), format!("[A] Select    [B] Dismiss{tabs}"));
        app.ask("Quit?", String::new(), "Quit", Act::Quit);
        assert_eq!(hints(&app), "[A] Select    [B] Cancel");
    }
}
