//! History: the generations, newest first, with "Go back to this" (rollback) behind a confirm.

use eframe::egui::Ui;
use fl_core::workspace::{Generation, Status};

use crate::app::{Act, App};
use crate::ui::{self, Row};

#[derive(Default)]
pub struct History {
    /// Generation number, generation, age of its file in seconds.
    gens: Vec<(u32, Generation, Option<u64>)>,
    /// The generation "Go back to this" acts on: the last focused row.
    sel: Option<u32>,
}

/// "just now", "5 min ago", "3 h ago", "2 days ago".
pub fn ago(secs: u64) -> String {
    match secs {
        0..60 => "just now".into(),
        60..3600 => format!("{} min ago", secs / 60),
        3600..86_400 => format!("{} h ago", secs / 3600),
        _ => format!("{} day(s) ago", secs / 86_400),
    }
}

pub fn load(app: &mut App) {
    let Some(ws) = &app.ws else {
        return;
    };
    let age = |n: u32| {
        let m = std::fs::metadata(ws.dir.join(format!("history/{n:04}.toml")));
        let t = m.and_then(|m| m.modified()).ok()?;
        Some(t.elapsed().ok()?.as_secs())
    };
    match ws.history() {
        Ok(h) => {
            app.history.gens = h.into_iter().rev().map(|(n, g)| (n, g, age(n))).collect();
            app.history.sel = None;
        }
        Err(e) => app.error(e.to_string()),
    }
}

fn mods_line(g: &Generation) -> String {
    match g.mods.is_empty() {
        true => "(no mods)".into(),
        false => g.mods.join(", "),
    }
}

pub fn show(app: &mut App, ui: &mut Ui) {
    let Some(base) = app.ws.as_ref().map(|w| w.state.base.clone()) else {
        return;
    };
    let h = &mut app.history;
    let mut back = None;
    ui::body(ui, |ui| {
        ui::card(ui, "Generations: each Apply is one", |ui| {
            for (n, g, age) in &h.gens {
                let status = match g.status {
                    Status::Complete => "",
                    Status::InProgress => "  (interrupted)",
                    Status::RolledBack => "  (rolled back)",
                };
                let mods = mods_line(g);
                let label = format!("#{n}  {}{status}\n{mods}", g.note);
                let mut row = Row::new(label, age.map_or(String::new(), ago));
                if g.status != Status::Complete || g.base != base {
                    row.muted = true;
                    row.show(ui, |_| {});
                    continue;
                }
                if app.nav.focus == app.nav.next_index() || h.sel.is_none() {
                    h.sel = Some(*n);
                }
                row.hot = h.sel == Some(*n);
                if row.nav(ui, &mut app.nav) {
                    back = Some((*n, mods));
                }
            }
            if h.gens.is_empty() {
                ui.weak("No generations yet: apply your mods first.");
            }
        })
    });
    let sel = h.sel.and_then(|s| h.gens.iter().find(|g| g.0 == s));
    if let Some((n, g, _)) = sel
        && ui::actions(ui, &mut app.nav, &["Go back to this"]).is_some()
    {
        back = Some((*n, mods_line(g)));
    }
    if let Some((n, mods)) = back {
        let body =
            format!("The game gets exactly these mods again; the others are disabled.\n\n{mods}");
        app.ask(
            &format!("Go back to generation {n}?"),
            body,
            "Go back",
            Act::Rollback(n),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ages_read_naturally() {
        assert_eq!(ago(5), "just now");
        assert_eq!(ago(125), "2 min ago");
        assert_eq!(ago(7200), "2 h ago");
        assert_eq!(ago(3 * 86_400 + 5), "3 day(s) ago");
    }
}
