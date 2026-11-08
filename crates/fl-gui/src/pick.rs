//! Pick game (first run, grilling Q10): detected installs paired with Wine prefixes, plus a
//! folder chosen in the in-app browser. The user always confirms; then init scans the game.

use std::path::{Path, PathBuf};

use eframe::egui::Ui;
use fl_core::detect::{self, Found};

use crate::app::{Act, App, home};
use crate::ui::{self, Row};
use crate::widgets::Browser;

pub const EXE: &str = "FL_2027.exe";

type Candidate = (PathBuf, Option<PathBuf>);

#[derive(Default)]
pub struct Pick {
    games: Vec<Candidate>,
    prefixes: Vec<PathBuf>,
    /// The browser, and whether it picks a prefix (else a game folder).
    browse: Option<(bool, Browser)>,
}

impl Pick {
    pub fn browsing(&self) -> bool {
        self.browse.is_some()
    }
}

/// Every game with the prefix it lives in, else with each known prefix, else with none.
pub fn candidates(games: &[Candidate], prefixes: &[PathBuf]) -> Vec<Candidate> {
    let mut out = Vec::new();
    for (g, own) in games {
        match own {
            Some(p) => out.push((g.clone(), Some(p.clone()))),
            None if prefixes.is_empty() => out.push((g.clone(), None)),
            None => out.extend(prefixes.iter().map(|p| (g.clone(), Some(p.clone())))),
        }
    }
    out.dedup();
    out
}

pub fn detect(app: &mut App) {
    app.run("Looking for FL27 installs…", || {
        let found = detect::detect(&home(), EXE);
        Ok(Box::new(|app: &mut App| detected(app, found)))
    });
}

/// Merges a detection; opens the workspace directly when exactly one install has one.
pub fn detected(app: &mut App, found: Found) {
    for g in found.games {
        if !app.pick.games.contains(&g) {
            app.pick.games.push(g);
        }
    }
    for p in found.prefixes {
        if !app.pick.prefixes.contains(&p) {
            app.pick.prefixes.push(p);
        }
    }
    let dirs = app.pick.games.iter().map(|g| app.ws_dir_for(&g.0));
    let mut dirs: Vec<PathBuf> = dirs.filter(|d| d.join("state.toml").is_file()).collect();
    dirs.dedup();
    if app.ws_override.is_none() && dirs.len() == 1 {
        app.open(dirs.remove(0));
    }
}

fn show_path(p: Option<&Path>) -> String {
    p.map_or("none (game settings unavailable)".into(), |p| {
        p.display().to_string()
    })
}

fn browse(app: &mut App, ui: &mut Ui) {
    let Some((for_prefix, b)) = &mut app.pick.browse else {
        return;
    };
    let for_prefix = *for_prefix;
    let (title, what) = match for_prefix {
        true => (
            "Choose the Wine prefix (the folder holding drive_c)",
            "the Wine prefix",
        ),
        false => ("Choose the folder holding FL_2027.exe", "the game"),
    };
    ui.heading(title);
    let Some(picked) = b.ui(ui, &mut app.nav, true, &format!("Use as {what}")) else {
        return;
    };
    app.pick.browse = None;
    app.nav.reset();
    match picked {
        Some(p) if for_prefix => app.pick.prefixes.push(p),
        Some(p) if p.join(EXE).is_file() => app.pick.games.push((p, None)),
        Some(p) => app.error(format!("No {EXE} in {}.", p.display())),
        None => {}
    }
}

pub fn show(app: &mut App, ui: &mut Ui) {
    if app.pick.browse.is_some() {
        return browse(app, ui);
    }
    ui.heading("Pick your Football Life 2027 install");
    let mut chosen = None;
    ui::body(ui, |ui| {
        ui::card(ui, "Game folder and Wine prefix", |ui| {
            for (g, p) in candidates(&app.pick.games, &app.pick.prefixes) {
                let label = format!("{}\nprefix: {}", g.display(), show_path(p.as_deref()));
                let has = app.ws_dir_for(&g).join("state.toml").is_file();
                if Row::new(label, if has { "workspace" } else { "" }).nav(ui, &mut app.nav) {
                    chosen = Some((g, p));
                }
            }
            if app.pick.games.is_empty() {
                ui.weak("No install found in ~/Games or the Steam libraries.");
            }
        })
    });
    let labels = [
        "Scan again",
        "Choose a Wine prefix…",
        "Choose another folder…",
    ];
    match ui::actions(ui, &mut app.nav, &labels) {
        Some(0) => detect(app),
        Some(i) => {
            app.pick.browse = Some((i == 1, Browser::new(&home(), false)));
            app.nav.reset();
        }
        None => {}
    }
    let Some((game, prefix)) = chosen else {
        return;
    };
    let dir = app.ws_dir_for(&game);
    if dir.join("state.toml").is_file() {
        let body = format!("Workspace: {}", dir.display());
        return app.ask("Open this workspace?", body, "Open", Act::Open(dir));
    }
    let body = format!(
        "Game:      {}\nPrefix:    {}\nWorkspace: {}\n\nThe first run scans every game file once \
         (several minutes on a full install).\nNothing in the game is changed.",
        game.display(),
        show_path(prefix.as_deref()),
        dir.display()
    );
    let act = Act::Init(game, prefix);
    app.ask("Set up fl-modman here?", body, "Set up", act);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn games_pair_with_their_own_prefix_or_every_known_one() {
        let p = |s: &str| PathBuf::from(s);
        let games = [
            (p("/pfx/drive_c/FL"), Some(p("/pfx"))),
            (p("/Games/FL27"), None),
        ];
        let got = candidates(&games, &[p("/a"), p("/b")]);
        assert_eq!(
            got,
            [
                (p("/pfx/drive_c/FL"), Some(p("/pfx"))),
                (p("/Games/FL27"), Some(p("/a"))),
                (p("/Games/FL27"), Some(p("/b"))),
            ]
        );
        assert_eq!(candidates(&games[1..], &[]), [(p("/Games/FL27"), None)]);
    }
}
