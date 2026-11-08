//! Storage: disk usage per mod and for the workspace (stat only, measured when the tab opens and
//! after Free, never per frame), the cross-filesystem warning, and "Free X" behind a confirm
//! (Cancel focused), which deletes exactly the listed disabled, unapplied mods, orphan folders and
//! `staging/` leftovers.

use std::sync::Arc;

use eframe::egui::{Grid, RichText, Ui};
use fl_core::plan::human_size;
use fl_core::usage::{ModUsage, Usage};

use crate::app::{Act, App, Then};
use crate::ui::{self, MUTED, Row, S1, S3};

#[derive(Default)]
pub struct Storage {
    pub usage: Option<Usage>,
}

/// Measures the workspace on a background thread.
pub fn load(app: &mut App) {
    app.storage.usage = None;
    app.with_ws("Measuring the workspace…", |ws| {
        let u = ws.usage()?;
        Ok(Box::new(move |app: &mut App| app.storage.usage = Some(u)))
    });
}

/// The job behind the confirmed Free: remove exactly the listed items, then measure again.
pub fn free(app: &mut App, ids: Vec<String>, staging: bool) {
    app.with_ws("Freeing space…", move |ws| {
        let f = ws.remove_disabled(&ids, staging)?;
        let u = ws.usage()?;
        let then: Then = Box::new(move |app: &mut App| {
            if let Some(c) = &mut app.cache {
                let c = Arc::make_mut(c);
                f.removed.iter().for_each(|id| drop(c.pkgs.remove(id)));
            }
            app.storage.usage = Some(u);
            let msg = format!("Freed {} ({}).", human_size(f.bytes), f.removed.join(", "));
            match f.kept.is_empty() {
                true => app.info(msg),
                false => app.error(format!("{msg} Kept: {}.", f.kept.join("; "))),
            }
        });
        Ok(then)
    });
}

fn state(m: &ModUsage) -> &'static str {
    match (m.enabled, m.applied) {
        (true, true) => "Applied",
        (true, false) => "On, not applied yet",
        (false, true) => "Off, in the game until you apply",
        (false, false) => "Off",
    }
}

pub fn show(app: &mut App, ui: &mut Ui) {
    let Some(u) = app.storage.usage.clone() else {
        let msg =
            "Could not measure the workspace (see the message above). Open the tab again to retry.";
        ui.weak(msg);
        return;
    };
    ui::body(ui, |ui| {
        if !u.same_fs {
            ui::badge(
                ui,
                "The workspace and the game are on different drives: mods are copied, not \
                 linked, so every applied file takes its space twice.",
            );
        }
        u.notes.iter().for_each(|n| ui::badge(ui, n));
        // Dense text, not full-height rows, so the totals and several mods fit on one screen.
        let free = human_size(u.reclaimable());
        let title = format!("Workspace · {free} can be freed (mods off and not applied)");
        ui::card(ui, &title, |ui| {
            let only: u64 = u.mods.iter().map(ModUsage::only_here).sum();
            let cells = [
                ("Mods", u.mods_dir),
                ("Only in the workspace", only),
                ("Game files kept for undo", u.originals),
                ("History", u.history),
                ("Leftovers of failed imports", u.staging),
                ("Total", u.total),
            ];
            let grid = Grid::new("totals").spacing([S3, S1]).min_row_height(0.0);
            grid.show(ui, |ui| {
                for line in cells.chunks(3) {
                    for (k, v) in line {
                        ui.label(RichText::new(*k).color(MUTED));
                        ui.label(human_size(*v));
                    }
                    ui.end_row();
                }
            });
        });
        ui::card(ui, "Mods: size (shared with the game)", |ui| {
            for m in &u.mods {
                let meta = format!(
                    "{} · {}  ({} shared)",
                    state(m),
                    human_size(m.bytes),
                    human_size(m.shared)
                );
                let mut row = Row::new(&m.id, meta);
                row.muted = !m.enabled;
                row.nav(ui, &mut app.nav);
            }
            for (id, b) in &u.orphans {
                let meta = format!("Orphan folder, not in the list · {}", human_size(*b));
                Row::new(id, meta).nav(ui, &mut app.nav);
            }
            if u.mods.is_empty() && u.orphans.is_empty() {
                ui.weak("No mods yet.");
            }
        });
    });
    let size = u.reclaimable();
    if size == 0 {
        return;
    }
    let label = format!("Free {}", human_size(size));
    if ui::actions(ui, &mut app.nav, &[&label]).is_some() {
        let ids = u.reclaimable_ids();
        let size_of = |id: &String| {
            let m = u.mods.iter().find(|m| &m.id == id).map(|m| m.bytes);
            m.or(u.orphans.iter().find(|o| &o.0 == id).map(|o| o.1))
                .unwrap_or(0)
        };
        let mut body: Vec<String> = ids
            .iter()
            .map(|id| format!("{id}  {}", human_size(size_of(id))))
            .collect();
        let staging = u.staging > 0;
        if staging {
            body.push(format!("staging/ leftovers  {}", human_size(u.staging)));
        }
        let body = format!(
            "Exactly these are deleted from the workspace (re-import to get a mod back):\n\n{}",
            body.join("\n")
        );
        app.ask(
            &format!("{label}?"),
            body,
            "Delete",
            Act::Free(ids, staging),
        );
    }
}
