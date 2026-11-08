//! fl-modman core: scan a game tree once (the base), compute the tree that an ordered list of
//! mods produces and the plan that realizes it, and apply it with history, undo and drift checks.
//!
//! List order decides every conflict: the lower mod in the list wins (grilling Q5). Layers
//! (docs/engine.md): `domain` is the pure model, `kinds` the mod types, `codec` the byte-exact
//! file formats, `ports` + `adapters` the host OS, `workspace` the state next to the game.

mod adapters;
pub mod codec;
mod domain;
pub mod error;
pub mod kinds;
pub mod ports;
pub mod workspace;

pub use adapters::{HOST, detect};
pub use codec::graphics;
pub use domain::{base, build, hash, manifest, paths, plan, target};
pub use error::{Error, Result};
pub use workspace::{import, settings};

/// With `FL_MODMAN_TIMING` set, prints to stderr the time since the previous call.
pub fn lap(label: &str) {
    use std::sync::Mutex;
    use std::time::Instant;
    static LAST: Mutex<Option<Instant>> = Mutex::new(None);
    if std::env::var_os("FL_MODMAN_TIMING").is_some() {
        let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        let ms = last.map_or(0.0, |t| (now - t).as_secs_f64() * 1e3);
        eprintln!("timing {label:<12} {ms:8.1} ms");
        *last = Some(now);
    }
}
