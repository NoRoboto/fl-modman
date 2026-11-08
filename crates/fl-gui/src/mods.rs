//! My mods (main screen): the ordered list where the lower mod wins (grilling Q5), toggles,
//! move mode (Y + D-pad) and mouse drag, conflict badges, and Apply with a plan summary. An
//! interrupted apply or drift replaces the list with the choices the engine allows.

use std::collections::BTreeMap;

use eframe::egui::{RichText, Ui};
use fl_core::build::Conflict;
use fl_core::plan::{Op, Plan, human_size};

use crate::app::{Act, App, done};
use crate::input::Action;
use crate::ui::{self, Row, WARNING};

#[derive(Default)]
pub struct Mods {
    pub moving: bool,
    /// The "Already installed" card is expanded (collapsed by default).
    base_open: bool,
}

/// The collapsed / expanded header of the base card: (label, meta).
pub fn base_header(n: usize, open: bool) -> (String, String) {
    let arrow = if open { "⏷" } else { "▶" };
    (format!("{arrow}  Already installed (base)"), format!("{n}"))
}

/// The conflicts `id` takes part in.
pub fn badges(id: &str, conflicts: &[Conflict]) -> Vec<String> {
    let mine = conflicts.iter().filter(|c| c.winner == id || c.loser == id);
    mine.map(Conflict::to_string).collect()
}

/// The plan as the confirm dialog shows it.
pub fn plan_summary(plan: &Plan) -> String {
    let (mut puts, mut bytes, mut dels, mut docs) = (0, 0, 0, Vec::new());
    let mut by: BTreeMap<&str, usize> = BTreeMap::new();
    for op in &plan.ops {
        match op {
            Op::Put { size, by: who, .. } => {
                (puts, bytes) = (puts + 1, bytes + size);
                *by.entry(who.as_deref().unwrap_or("game (restored)"))
                    .or_default() += 1;
            }
            Op::Del { .. } => dels += 1,
            Op::Doc { path, .. } => docs.push(path.rsplit('/').next().unwrap_or(path)),
        }
    }
    let mut s = format!("Place {puts} file(s), {}\n", human_size(bytes));
    for (who, n) in by {
        s += &format!("    {n} from {who}\n");
    }
    s += &format!("Remove {dels} file(s)\nUpdate {} document(s)", docs.len());
    if !docs.is_empty() {
        s += &format!(": {}", docs.join(", "));
    }
    let n = plan.conflicts.len();
    s += &format!("\n{n} conflict(s), the lower mod wins\n");
    for c in &plan.conflicts {
        s += &format!("    {c}\n");
    }
    for w in &plan.warnings {
        s += &format!("Warning: {w}\n");
    }
    s
}

fn pending(app: &mut App, ui: &mut Ui, n: u32) {
    ui.heading(RichText::new(format!("Generation {n} was interrupted")).color(WARNING));
    ui::body(ui, |ui| {
        ui.label("An apply stopped halfway: finish it, or roll back to the last complete one.");
    });
    if let Some(i) = ui::actions(ui, &mut app.nav, &["Roll back", "Finish"]) {
        let finish = i == 1;
        app.with_ws("Recovering…", move |ws| {
            let n = ws.recover(finish)?;
            let how = if finish { "finished" } else { "rolled back" };
            done(format!("Generation {n} {how}."))
        });
    }
}

fn drift(app: &mut App, ui: &mut Ui, files: &[String]) {
    let title = format!("{} file(s) changed outside fl-modman", files.len());
    ui.heading(RichText::new(title).color(WARNING));
    let back = app.nav.take(Action::Back);
    ui::body(ui, |ui| {
        ui.label(
            "A game update or a hand edit changed them. Accept them as the new base (the next \
             Apply re-applies your mods), or restore what fl-modman placed.",
        );
        ui::card(ui, "Changed files", |ui| {
            files.iter().take(200).for_each(|f| drop(ui.weak(f)));
        });
    });
    let labels = ["Close", "Restore", "Accept as new base"];
    let pressed = ui::actions(ui, &mut app.nav, &labels);
    if back || pressed.is_some() {
        app.drift = None;
    }
    if pressed == Some(2) {
        app.with_ws("Taking the changed files into the base…", |ws| {
            let n = ws.accept_drift()?.len();
            done(format!(
                "{n} file(s) taken into the base; press Apply changes to re-apply the mods."
            ))
        });
    }
    if pressed == Some(1) {
        app.with_ws("Restoring…", |ws| {
            ws.restore()?;
            done("Restored what fl-modman placed.".into())
        });
    }
}

pub fn show(app: &mut App, ui: &mut Ui) {
    if let Some(n) = app.pending {
        return pending(app, ui, n);
    }
    if let Some(files) = app.drift.clone() {
        return drift(app, ui, &files);
    }
    let Some(ws) = &app.ws else {
        return;
    };
    let (list, disabled) = (ws.state.mods.clone(), ws.state.disabled.clone());
    let n = list.len();
    let focus = app.nav.focus;
    let mut moving = app.mods.moving && focus < n;
    let mut swap = None;
    if moving {
        if app.nav.take(Action::Up) && focus > 0 {
            swap = Some((focus, focus - 1));
        } else if app.nav.take(Action::Down) && focus + 1 < n {
            swap = Some((focus, focus + 1));
        }
        moving &= !(app.nav.take(Action::Back) || app.nav.take(Action::Accept));
    }
    let (mut toggle, mut drop_on) = (None, None);
    let cache = app.cache.clone();
    let installed = cache.as_ref().map(|c| c.installed()).unwrap_or_default();
    ui::body(ui, |ui| {
        ui::card(ui, "Load order: the lower mod wins", |ui| {
            for (i, id) in list.iter().enumerate() {
                let badges = badges(id, &app.conflicts);
                let hot = moving && i == focus;
                let (name, mut meta) = match &cache {
                    Some(c) => (c.label(id), c.meta(id)),
                    None => (id.clone(), String::new()),
                };
                if hot {
                    meta = "↕ moving".into();
                }
                let row = ui.horizontal(|ui| {
                    ui.dnd_drag_source(ui.scope_id().with(("drag", i)), i, |ui| {
                        ui.label(RichText::new("☰").color(ui::MUTED))
                    });
                    let row = Row {
                        hot,
                        badges: &badges,
                        ..Row::new(format!("{}. {name}", i + 1), meta)
                    };
                    row.show(ui, |ui| ui::toggle(ui, !disabled.contains(id)))
                });
                if let Some(src) = row.response.dnd_release_payload::<usize>() {
                    drop_on = Some((*src, i));
                }
                if app.nav.item(ui, &row.inner) || app.nav.on_focused(Action::Toggle) {
                    toggle = Some(i);
                }
                if app.nav.on_focused(Action::Move) {
                    moving = !moving;
                }
            }
            if n == 0 {
                ui.weak("No managed mods yet: add one in Add mod.");
            }
        });
        if installed.is_empty() {
            return;
        }
        ui::card(ui, "", |ui| {
            let (label, meta) = base_header(installed.len(), app.mods.base_open);
            if Row::new(label, meta).nav(ui, &mut app.nav) {
                app.mods.base_open ^= true;
            }
            for (name, kind) in installed.iter().filter(|_| app.mods.base_open) {
                let row = Row {
                    muted: true,
                    ..Row::new(name, *kind)
                };
                row.nav(ui, &mut app.nav);
            }
        });
    });
    app.mods.moving = moving;
    let apply = ui::actions(ui, &mut app.nav, &["Apply changes"]).is_some();
    let Some(ws) = &mut app.ws else {
        return;
    };
    if let Some(i) = toggle.filter(|_| !moving) {
        let id = &ws.state.mods[i];
        if !ws.state.disabled.remove(id) {
            ws.state.disabled.insert(id.clone());
        }
        app.changed();
    } else if let Some((from, to)) = swap.or(drop_on).filter(|(a, b)| a != b) {
        let m = ws.state.mods.remove(from);
        ws.state.mods.insert(to, m);
        app.nav.set_focus(to);
        app.changed();
    }
    if apply {
        app.with_ws("Checking the game files…", |ws| {
            ws.refuse_running()?;
            let list = ws.enabled();
            let plan = ws.apply(&list, "apply", false, true)?.0;
            Ok(Box::new(move |app: &mut App| match plan.ops.is_empty() {
                true => app.info("Nothing to change: the game already matches the list."),
                false => app.ask(
                    "Apply these changes?",
                    plan_summary(&plan),
                    "Apply",
                    Act::Apply,
                ),
            }))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fl_core::plan::Replaces;

    fn conflict(winner: &str, loser: &str, n: usize) -> Conflict {
        let items = (0..n).map(|i| format!("f{i}")).collect();
        let (winner, loser) = (winner.into(), loser.into());
        Conflict {
            winner,
            loser,
            items,
        }
    }

    #[test]
    fn the_base_card_starts_collapsed_with_a_count() {
        assert!(!Mods::default().base_open);
        let (label, meta) = base_header(12, false);
        assert_eq!(
            (label.as_str(), meta.as_str()),
            ("▶  Already installed (base)", "12")
        );
        assert!(base_header(12, true).0.starts_with('⏷'));
    }

    #[test]
    fn badges_name_both_sides() {
        let c = [conflict("b", "a", 3), conflict("c", "b", 1)];
        assert_eq!(badges("a", &c), ["b overrides 3 file(s) of a"]);
        assert_eq!(
            badges("b", &c),
            ["b overrides 3 file(s) of a", "c overrides 1 file(s) of b"]
        );
        assert!(badges("d", &c).is_empty());
    }

    #[test]
    fn plan_summary_counts_puts_dels_docs_and_conflicts() {
        let put = |path: &str, by: Option<&str>| Op::Put {
            path: path.into(),
            b3: String::new(),
            size: 1024,
            by: by.map(String::from),
            replaces: Replaces::Nothing,
        };
        let plan = Plan {
            target: "fl27".into(),
            base_snapshot: String::new(),
            exe_sha256: None,
            version: None,
            mods: vec!["a".into(), "b".into()],
            ops: vec![
                Op::Del {
                    path: "x".into(),
                    size: 1,
                },
                put("p1", Some("b")),
                put("p2", Some("b")),
                put("p3", None),
                Op::Doc {
                    path: "SiderAddons/sider.ini".into(),
                    b3: String::new(),
                    size: 0,
                    changes: vec![],
                },
            ],
            conflicts: vec![conflict("b", "a", 2)],
            warnings: vec!["w".into()],
        };
        assert_eq!(
            plan_summary(&plan),
            "Place 3 file(s), 3.0 KiB\n    2 from b\n    1 from game (restored)\nRemove 1 file(s)\n\
             Update 1 document(s): sider.ini\n1 conflict(s), the lower mod wins\n    \
             b overrides 2 file(s) of a\nWarning: w\n"
        );
    }
}
