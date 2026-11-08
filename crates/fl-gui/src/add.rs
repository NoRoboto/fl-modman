//! Add mod (grilling Q4): browse to an archive or folder → import → preview (root, type, files,
//! conflicts) → keep or discard. Ambiguous roots are picked from the candidates; a stadium
//! asks its team id on the numeric picker. Nothing reaches the game until Apply.

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::Ui;
use fl_core::import::{Imported, Options, import};
use fl_core::kinds::Kind;
use fl_core::manifest::Package;

use crate::app::{App, Tab, home};
use crate::input::Action;
use crate::ui::{self, Row};
use crate::widgets::{Browser, NumPicker};

pub enum Stage {
    Browse,
    Roots(Vec<(String, Kind)>),
    Team(NumPicker),
    Preview(Box<Imported>),
}

pub struct Add {
    pub stage: Stage,
    browser: Browser,
    src: Option<PathBuf>,
    root: Option<String>,
}

impl Default for Add {
    fn default() -> Self {
        let downloads = home().join("Downloads");
        let start = if downloads.is_dir() {
            downloads
        } else {
            home()
        };
        Add {
            stage: Stage::Browse,
            browser: Browser::new(&start, true),
            src: None,
            root: None,
        }
    }
}

fn start(app: &mut App, team: Option<String>) {
    let (Some(src), root) = (app.add.src.clone(), app.add.root.clone()) else {
        return;
    };
    app.with_ws("Importing…", move |ws| {
        let (team, root) = (team.as_deref(), root.as_deref());
        let m = import(
            ws,
            &src,
            Options {
                team,
                root,
                ..Default::default()
            },
        )?;
        let pkg = Package::load(&ws.dir.join("mods").join(&m.id), &ws.t)?;
        Ok(Box::new(move |app: &mut App| {
            if let Some(c) = &mut app.cache {
                Arc::make_mut(c).pkgs.insert(m.id.clone(), pkg);
            }
            app.add.stage = Stage::Preview(Box::new(m));
        }))
    });
}

/// Shows the imported mod; Some(true) to keep it, Some(false) to discard it.
fn preview(app: &mut App, ui: &mut Ui, m: &Imported) -> Option<bool> {
    ui.heading(format!("Imported {}", m.id));
    let conflicts: Vec<String> = m.conflicts.iter().map(ToString::to_string).collect();
    ui::body(ui, |ui| {
        ui::card(ui, "Preview", |ui| {
            let (n, kind) = (m.files.len().to_string(), format!("{:?}", m.kind));
            for (k, v) in [("Type", &kind), ("Root", &m.root), ("Files", &n)] {
                Row::new(k, v).show(ui, |_| {});
            }
            Row::new("Conflicts", conflicts.len().to_string()).show(ui, |_| {});
            conflicts.iter().for_each(|c| ui::badge(ui, c));
            ui.weak("Enabled at the bottom of the list (it wins).");
        });
        ui::card(ui, "Files", |ui| {
            m.files.iter().take(100).for_each(|f| drop(ui.weak(f)));
        });
    });
    let pressed = ui::actions(ui, &mut app.nav, &["Discard", "Keep"]);
    pressed.map(|i| i == 1)
}

pub fn show(app: &mut App, ui: &mut Ui) {
    let editing = matches!(&app.add.stage, Stage::Team(p) if p.editing);
    let browsing = matches!(app.add.stage, Stage::Browse);
    let cancel = !browsing && !editing && app.nav.take(Action::Back);
    match std::mem::replace(&mut app.add.stage, Stage::Browse) {
        Stage::Browse => {
            ui.heading("Pick an archive (zip, 7z, rar) or a mod folder");
            let picked = app
                .add
                .browser
                .ui(ui, &mut app.nav, false, "Import this folder");
            if let Some(Some(src)) = picked {
                (app.add.src, app.add.root) = (Some(src), None);
                start(app, None);
            }
        }
        _ if cancel => app.nav.reset(),
        Stage::Roots(roots) => {
            ui.heading("Where is the mod inside the archive?");
            let mut picked = None;
            ui::body(ui, |ui| {
                ui::card(ui, "Several folders look like a mod root", |ui| {
                    for (root, kind) in &roots {
                        if Row::new(root, format!("{kind:?}")).nav(ui, &mut app.nav) {
                            picked = Some(root.clone());
                        }
                    }
                })
            });
            match picked {
                Some(r) => {
                    app.add.root = Some(r);
                    start(app, None);
                }
                None => app.add.stage = Stage::Roots(roots),
            }
        }
        Stage::Team(mut picker) => {
            ui.heading("This is a stadium: which team plays there?");
            ui::body(ui, |ui| {
                ui::card(ui, "Team id (as in map_teams.txt)", |ui| {
                    picker.ui(ui, &mut app.nav);
                })
            });
            let label = format!("Import for team {}", picker.value());
            if ui::actions(ui, &mut app.nav, &[&label]).is_some() {
                let team = picker.value();
                start(app, Some(team.to_string()));
            } else {
                app.add.stage = Stage::Team(picker);
            }
        }
        Stage::Preview(m) => match preview(app, ui, &m) {
            Some(true) => app.goto(Tab::Mods),
            Some(false) => {
                let id = m.id.clone();
                app.with_ws("Discarding…", move |ws| {
                    ws.state.disabled.insert(id.clone());
                    ws.remove(&id)?;
                    Ok(Box::new(move |app: &mut App| {
                        if let Some(c) = &mut app.cache {
                            Arc::make_mut(c).pkgs.remove(&id);
                        }
                        app.add.stage = Stage::Browse;
                        app.info(format!("Discarded {id}."));
                    }))
                });
            }
            None => app.add.stage = Stage::Preview(m),
        },
    }
}
