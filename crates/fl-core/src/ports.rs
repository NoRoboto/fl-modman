//! What changes with the host OS (grilling Q1: Linux now, Windows later). One implementation
//! per OS lives in `adapters`, picked at compile time as [`crate::HOST`].

use std::path::Path;

use crate::Result;

pub trait Host: Sync {
    /// True if a process is the game exe (`exe` is its file name, e.g. `FL_2027.exe`).
    fn game_running(&self, exe: &str) -> bool;
    /// Extracts an archive (zip, 7z, rar) into the folder `to`.
    fn extract(&self, src: &Path, to: &Path) -> Result<()>;
}
