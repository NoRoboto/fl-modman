//! Game settings (design.md Appendix C): the fields of the prefix's `settings.dat`, saved with a
//! backup after a confirm and never while the game runs; plus the Proton overrides diagnostic
//! (grilling Q18: shown, never written to Steam). The in-game rendering options (`GRAPHICS000000`)
//! follow, edited on a working copy so the preset turns custom as soon as a field changes.

use std::path::PathBuf;

use eframe::egui::{Label, Ui};
use fl_core::detect::{OVERRIDES, overrides_set};
use fl_core::{HOST, graphics, settings};

use crate::app::{Act, App, home};
use crate::nav::Nav;
use crate::pick::EXE;
use crate::ui;
use crate::widgets::NumPicker;

/// (key, label, card), in screen order; `udp_port` is edited on the numeric picker.
const FIELDS: [(&str, &str, &str); 12] = [
    ("resolution", "Resolution", "Display"),
    ("fullscreen", "Fullscreen", "Display"),
    ("hdr", "HDR", "Display"),
    ("fps_game", "Frame rate in game", "Frame rate & sync"),
    ("fps_replay", "Frame rate in replays", "Frame rate & sync"),
    ("vsync", "Vsync (on2 = half refresh)", "Frame rate & sync"),
    ("sound_buffer", "Sound buffer (size index)", "Audio & input"),
    ("xinput", "XInput controller", "Audio & input"),
    ("p2p", "P2P (online)", "Online"),
    ("udp_port", "UDP port", "Online"),
    ("auto_udp", "Automatic UDP port", "Online"),
    ("voice_chat", "Voice chat", "Online"),
];
const CARDS: [&str; 4] = ["Display", "Frame rate & sync", "Audio & input", "Online"];

pub struct Row {
    key: &'static str,
    card: &'static str,
    label: String,
    options: Vec<String>,
    cur: usize,
    orig: usize,
}

#[derive(Default)]
pub struct Settings {
    file: Option<PathBuf>,
    rows: Vec<Row>,
    port: Option<(NumPicker, u32)>,
    doctor: Option<Option<PathBuf>>,
    /// `GRAPHICS000000` as read and as edited.
    gfx: Option<(Vec<u8>, Vec<u8>)>,
    render: Vec<Row>,
}

/// The choices of a field, with the current value added when it is not one of them.
pub fn options(key: &str, cur: &str) -> Vec<String> {
    let list: &[&str] = match key {
        "resolution" => &[
            "1280x720",
            "1366x768",
            "1600x900",
            "1920x1080", // 16:9 (ROG Ally, TVs)
            "1280x800",
            "1440x900",
            "1680x1050",
            "1920x1200", // 16:10 (Steam Deck, Legion Go)
        ],
        "fps_game" | "fps_replay" => &["30", "60"],
        "vsync" => &["off", "on1", "on2"],
        "sound_buffer" => &["0", "1", "2", "3", "4", "5"],
        _ => &["off", "on"],
    };
    let mut out: Vec<String> = list.iter().map(|s| s.to_string()).collect();
    if !out.iter().any(|o| o == cur) {
        out.push(cur.into());
    }
    out
}

pub fn load(app: &mut App) {
    if app.settings.doctor.is_none() {
        app.settings.doctor = Some(overrides_set(&home()));
    }
    let prefix = app.ws.as_ref().and_then(|w| w.state.prefix.clone());
    let file = prefix
        .and_then(|p| settings::fl_docs(&p))
        .map(|d| d.join("settings.dat"));
    let (mut rows, mut port) = (Vec::new(), None);
    let bytes = file.as_ref().map(std::fs::read);
    match bytes {
        Some(Ok(b)) if settings::check(&b).is_ok() => {
            let values = settings::get(&b);
            for (key, label, card) in FIELDS {
                let Some((_, value)) = values.iter().find(|v| v.0 == key) else {
                    continue;
                };
                if key == "udp_port" {
                    let v = value.parse().unwrap_or(0);
                    port = Some((NumPicker::new(5, v), v));
                    continue;
                }
                let options = options(key, value);
                let cur = options.iter().position(|o| o == value).unwrap_or(0);
                rows.push(Row {
                    key,
                    card,
                    label: label.into(),
                    options,
                    cur,
                    orig: cur,
                });
            }
        }
        Some(Ok(_)) => app.error("settings.dat is not a FL27 settings file (no WECF magic)."),
        Some(Err(e)) => app.error(format!("settings.dat: {e}")),
        None => {}
    }
    let gfx = file
        .as_ref()
        .map(|f| std::fs::read(f.with_file_name(graphics::FILE)));
    let gfx = gfx.and_then(Result::ok);
    if gfx.as_ref().is_some_and(|b| graphics::check(b).is_err()) {
        app.error("GRAPHICS000000 is not 19 bytes; the rendering options are not shown.");
    }
    let s = &mut app.settings;
    s.gfx = gfx
        .filter(|b| graphics::check(b).is_ok())
        .map(|b| (b.clone(), b));
    (s.file, s.rows, s.port, s.render) = (file, rows, port, render_rows(&s.gfx));
}

/// The rendering rows from the edited copy, marked against the file as read.
fn render_rows(gfx: &Option<(Vec<u8>, Vec<u8>)>) -> Vec<Row> {
    let Some((orig, work)) = gfx else {
        return Vec::new();
    };
    let pairs = graphics::get(work).into_iter().zip(graphics::get(orig));
    let row = |((key, v), (_, o)): ((&'static str, String), (&str, String))| {
        let mut options = graphics::options(key);
        if !options.contains(&v) {
            options.push(v.clone());
        }
        // An original value that is not an option never matches, so the row shows `*`.
        let pos = |x: &String| options.iter().position(|y| y == x).unwrap_or(usize::MAX);
        let (cur, orig, s) = (pos(&v), pos(&o), key.replace('_', " "));
        let label = s[..1].to_uppercase() + &s[1..];
        Row {
            key,
            card: "",
            label,
            options,
            cur,
            orig,
        }
    };
    pairs.map(row).collect()
}

/// The changed fields as `key=value`.
fn changes(s: &Settings) -> Vec<String> {
    let rows = s.rows.iter().filter(|r| r.cur != r.orig);
    let mut out: Vec<String> = rows
        .map(|r| format!("{}={}", r.key, r.options[r.cur]))
        .collect();
    if let Some((p, orig)) = &s.port
        && p.value() != *orig
    {
        out.push(format!("udp_port={}", p.value()));
    }
    // A preset alone rewrites every field; a custom file is the changed fields.
    if let Some((orig, work)) = &s.gfx {
        let (o, w) = (graphics::get(orig), graphics::get(work));
        let diff = w.iter().zip(&o).filter(|(a, b)| a != b);
        match w[0].1.as_str() {
            "custom" => out.extend(diff.map(|((k, v), _)| format!("{k}={v}"))),
            p if work != orig => out.push(format!("preset={p}")),
            _ => {}
        }
    }
    out
}

pub fn save(app: &mut App) {
    let (Some(docs), Some(ws)) = (app.settings.file.as_ref().and_then(|f| f.parent()), &app.ws)
    else {
        return;
    };
    if HOST.game_running(EXE) {
        return app.error(format!("{EXE} is running; close the game first."));
    }
    match settings::update(docs, &ws.dir, &changes(&app.settings)) {
        Ok(baks) => app.info(format!("Settings saved (backups {baks:?}).")),
        Err(e) => app.error(e.to_string()),
    }
    load(app);
}

impl Settings {
    /// The UDP port picker is taking the D-pad.
    pub fn editing(&self) -> bool {
        self.port.as_ref().is_some_and(|p| p.0.editing)
    }
}

/// One stepper row; true when the value changed.
fn cycle(nav: &mut Nav, ui: &mut Ui, r: &mut Row) -> bool {
    let step = ui::stepper(ui, nav, &r.label, &r.options[r.cur], r.cur != r.orig);
    let n = r.options.len();
    r.cur = (r.cur + n).saturating_add_signed(step.into()) % n;
    step != 0
}

fn doctor(ui: &mut Ui, found: Option<&PathBuf>) {
    ui::card(ui, "Proton launch options", |ui| {
        match found {
            Some(f) => drop(ui.weak(format!("OK: set in {}", f.display()))),
            None => ui::badge(ui, "Not set in any Steam shortcut or game config."),
        }
        ui.weak("Steam > the game > Properties > Launch options (fl-modman never edits Steam):");
        let text = format!("WINEDLLOVERRIDES=\"{OVERRIDES}\" %command%");
        ui.add(Label::new(eframe::egui::RichText::new(text).monospace()).wrap());
    });
}

pub fn show(app: &mut App, ui: &mut Ui) {
    let s = &mut app.settings;
    ui::body(ui, |ui| {
        if s.rows.is_empty() {
            ui.weak("No settings.dat: the workspace has no Wine prefix, or the game never ran.");
        }
        let cards = CARDS
            .into_iter()
            .filter(|c| s.rows.iter().any(|r| r.card == *c));
        for card in cards.collect::<Vec<_>>() {
            ui::card(ui, card, |ui| {
                for r in s.rows.iter_mut().filter(|r| r.card == card) {
                    cycle(&mut app.nav, ui, r);
                }
                if let (Some((picker, _)), "Online") = (&mut s.port, card) {
                    ui.horizontal(|ui| {
                        ui.add_space(ui::S2);
                        ui.add_sized([320.0, 30.0], Label::new("UDP port"));
                        picker.ui(ui, &mut app.nav);
                    });
                }
            });
        }
        let mut edit = None;
        if !s.rows.is_empty() || !s.render.is_empty() {
            ui::card(ui, "Rendering (in-game)", |ui| {
                if s.render.is_empty() {
                    ui.weak(format!("No GRAPHICS000000: {}.", settings::GFX_HINT));
                }
                for r in &mut s.render {
                    if cycle(&mut app.nav, ui, r) {
                        edit = Some((r.key, r.options[r.cur].clone()));
                    }
                }
            });
        }
        if let (Some((k, v)), Some((_, work))) = (edit, &mut s.gfx) {
            let _ = graphics::set(work, k, &v); // a raw unknown value is not settable: kept
            s.render = render_rows(&s.gfx);
        }
        doctor(ui, s.doctor.as_ref().and_then(Option::as_ref));
    });
    let n = changes(s).len();
    let (save, mut pressed) = (format!("Save ({n} change(s))"), None);
    if !s.rows.is_empty() {
        pressed = ui::actions(ui, &mut app.nav, &["Revert", &save]);
    }
    if pressed == Some(0) {
        load(app);
    } else if pressed == Some(1) && n > 0 {
        let body = changes(&app.settings).join("\n")
            + "\n\nA backup of each changed file is kept in the workspace.";
        app.ask("Save the game settings?", body, "Save", Act::SaveSettings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_keep_the_current_value() {
        assert_eq!(options("resolution", "1280x800").len(), 8);
        assert_eq!(options("resolution", "2560x1600")[8], "2560x1600");
        assert_eq!(options("hdr", "on"), ["off", "on"]);
        assert_eq!(options("vsync", "unset").last().unwrap(), "unset");
    }

    fn set(s: &mut Settings, k: &str, v: &str) {
        graphics::set(&mut s.gfx.as_mut().unwrap().1, k, v).unwrap();
    }

    #[test]
    fn render_changes_follow_the_preset() {
        let mut b = vec![50u8; 19];
        graphics::set(&mut b, "preset", "medium").unwrap();
        let mut s = Settings {
            gfx: Some((b.clone(), b)),
            ..Default::default()
        };
        set(&mut s, "grass", "high");
        s.render = render_rows(&s.gfx);
        assert_eq!(s.render[0].options[s.render[0].cur], "custom");
        assert_eq!(changes(&s), ["preset=custom", "grass=high"]);
        set(&mut s, "preset", "high");
        assert_eq!(changes(&s), ["preset=high"]);
        set(&mut s, "preset", "medium");
        assert!(changes(&s).is_empty());
    }
}
