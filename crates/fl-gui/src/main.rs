//! `fl-modman-gui`: the gamepad-first egui front end (grilling Q3, Q16). It calls fl-core
//! directly; long operations run on background threads.

mod add;
mod app;
mod history;
mod input;
mod mods;
mod nav;
mod pick;
mod settings;
mod ui;
mod widgets;

use std::path::PathBuf;

use eframe::egui::ViewportBuilder;

const USAGE: &str = "usage: fl-modman-gui [--windowed] [-w|--workspace <dir>]

Workspace: -w, else $FL_MODMAN, else <game>/../fl-modman for the picked game.
Scale: FL_MODMAN_SCALE (pixels per point, default 1.6).";

fn main() -> eframe::Result<()> {
    let mut ws = std::env::var_os("FL_MODMAN").map(PathBuf::from);
    let (mut args, mut windowed) = (std::env::args().skip(1), false);
    while let Some(a) = args.next() {
        match (a.as_str(), args.len()) {
            ("--windowed", _) => windowed = true,
            ("-w" | "--workspace", 1..) => ws = args.next().map(PathBuf::from),
            _ => {
                println!("{USAGE}");
                return Ok(());
            }
        }
    }
    let scale = std::env::var("FL_MODMAN_SCALE")
        .ok()
        .and_then(|s| s.parse().ok());
    let scale = scale.filter(|s| (0.5..=4.0).contains(s)).unwrap_or(1.6);
    let viewport = ViewportBuilder::default()
        .with_title("fl-modman")
        .with_inner_size([1200.0, 750.0])
        .with_fullscreen(!windowed);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "fl-modman",
        options,
        Box::new(move |cc| {
            ui::install(&cc.egui_ctx);
            Ok(Box::new(app::App::new(cc.egui_ctx.clone(), ws, scale)))
        }),
    )
}
