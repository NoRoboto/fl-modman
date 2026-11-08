//! Storage: disk usage per mod and for the workspace (stat only, measured when the tab opens and
//! after Free, never per frame), the cross-filesystem warning, and "Free X" behind a confirm
//! (Cancel focused), which deletes exactly the listed disabled, unapplied mods, orphan folders and
//! `staging/` leftovers.

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
        (false, true) => "Off, still in the game until you apply",
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
            for (id, b) in &u.orphans {
                let label = format!("{id}\nOrphan folder: not in the mod list");
                Row::new(label, human_size(*b)).nav(ui, &mut app.nav);
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
