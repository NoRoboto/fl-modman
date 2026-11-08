//! The game's settings files in the Wine prefix: `settings.dat` ([`settings_dat`]) and the
//! rendering options ([`graphics`]), read and written together with backups.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::adapters::fs::write_atomic;
pub use crate::codec::settings_dat::{check, get, set};
use crate::codec::{graphics, settings_dat};
use crate::error::{Result, input, io};

/// `{prefix}/drive_c/users/*/Documents/SPGAME/Football Life`, if one exists.
pub fn fl_docs(prefix: &Path) -> Option<PathBuf> {
    let users = fs::read_dir(prefix.join("drive_c/users")).ok()?;
    let docs = users
        .flatten()
        .map(|u| u.path().join("Documents/SPGAME/Football Life"));
    docs.filter(|p| p.is_dir()).min()
}

/// Applies `key=value` pairs to `settings.dat` and [`graphics::FILE`] under `docs`, each key to
/// the file that has it. Every pair is checked in memory first; then each changed file is copied
/// to `<backup_dir>/<file name>.bak-<unix ms>` and rewritten. Returns the backup paths.
pub fn update(docs: &Path, backup_dir: &Path, pairs: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = [docs.join("settings.dat"), docs.join(graphics::FILE)].map(|f| (f, None));
    for kv in pairs {
        let (k, v) = kv
            .split_once('=')
            .ok_or_else(|| input(format!("{kv:?} is not key=value")))?;
        let (k, v, gfx) = (k.trim(), v.trim(), graphics::is_key(k.trim()));
        let (file, b) = &mut files[usize::from(gfx)];
        if b.is_none() {
            let bytes = fs::read(&*file).map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound if gfx => {
                    input(format!("no {}: {GFX_HINT}", file.display()))
                }
                _ => io(&*file)(e),
            })?;
            if gfx {
                graphics::check(&bytes)
            } else {
                settings_dat::check(&bytes)
            }?;
            *b = Some(bytes);
        }
        let b = b.as_mut().expect("read above");
        if gfx {
            graphics::set(b, k, v)
        } else {
            settings_dat::set(b, k, v)
        }?;
    }
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let mut baks = Vec::new();
    for (file, b) in files {
        let Some(b) = b else { continue };
        let name = file.file_name().unwrap_or_default().to_string_lossy();
        let bak = backup_dir.join(format!("{name}.bak-{ms}"));
        fs::copy(&file, &bak).map_err(io(&bak))?;
        write_atomic(&file, &b)?;
        baks.push(bak);
    }
    Ok(baks)
}

/// Why the rendering options file can be missing.
pub const GFX_HINT: &str = "open the in-game rendering options menu once (the game writes it then)";
