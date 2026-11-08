//! Storage: disk usage per mod and for the workspace (stat only, measured when the tab opens and
//! after Free, never per frame), the cross-filesystem warning, and "Free X" behind a confirm,
//! which removes disabled, unapplied mods and the `staging/` leftovers.

use std::sync::Arc;

use eframe::egui::Ui;
use fl_core::plan::human_size;
use fl_core::usage::{ModUsage, Usage};

use crate::app::{Act, App, Then};
use crate::ui::{self, Row};

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

/// The job behind the confirmed Free: remove, then measure again.
pub fn free(app: &mut App) {
    app.with_ws("Freeing space…", |ws| {
        let ids = ws.remove_disabled()?;
        let u = ws.usage()?;
        let then: Then = Box::new(move |app: &mut App| {
            if let Some(c) = &mut app.cache {
                let c = Arc::make_mut(c);
                ids.iter().for_each(|id| drop(c.pkgs.remove(id)));
            }
            app.storage.usage = Some(u);
            app.info(format!("Removed {} mod(s).", ids.len()));
        });
        Ok(then)
    });
}

fn state(m: &ModUsage) -> &'static str {
    match (m.enabled, m.applied) {
        (true, true) => "Applied",
        (true, false) => "On, not applied yet",
        (false, true) => "Off, still in the game until you apply",
        (false, false) => "Off",
    }
}

pub fn show(app: &mut App, ui: &mut Ui) {
    let Some(u) = app.storage.usage.clone() else {
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
        ui::card(ui, "Workspace", |ui| {
            let only: u64 = u.mods.iter().map(ModUsage::only_here).sum();
            let rows = [
                ("Mods", u.mods_dir),
                ("  of which only in the workspace", only),
                ("Game files kept for undo", u.originals),
                ("History", u.history),
                ("Leftovers of failed imports", u.staging),
                ("Total", u.total),
                (
                    "Can be freed (mods that are off and not applied)",
                    u.reclaimable(),
                ),
            ];
            for (k, v) in rows {
                Row::new(k, human_size(v)).show(ui, |_| {});
            }
        });
        ui::card(ui, "Mods: size (shared with the game)", |ui| {
            for m in &u.mods {
                let label = format!("{}\n{} · {} file(s)", m.id, state(m), m.files);
                let meta = format!("{}  ({} shared)", human_size(m.bytes), human_size(m.shared));
                let mut row = Row::new(label, meta);
                row.muted = !m.enabled;
                row.nav(ui, &mut app.nav);
            }
            if u.mods.is_empty() {
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
        let mut body: Vec<String> = u
            .mods
            .iter()
            .filter(|m| m.removable())
            .map(|m| format!("{}  {}", m.id, human_size(m.bytes)))
            .collect();
        if u.staging > 0 {
            body.push(format!("staging/ leftovers  {}", human_size(u.staging)));
        }
        let body = format!(
            "These are deleted from the workspace (re-import to get a mod back):\n\n{}",
            body.join("\n")
        );
        app.ask(&format!("{label}?"), body, "Delete", Act::Free);
    }
}
