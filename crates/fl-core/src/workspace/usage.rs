//! Disk usage of the workspace (`fl-modman usage`, the GUI Storage tab). Stat only, no hashing,
//! symlinks never followed.
//!
//! **Shared bytes.** Apply hardlinks a payload file into the game when both are on one
//! filesystem, so the bytes exist once. A payload file counts as shared when the file at its
//! tree path in the game has the same (device, inode): that is exactly the hardlink apply made.
//! A copy, a file another mod won, or a link made by something else does not count. It costs
//! one extra `lstat` per placed file of an applied mod. (`nlink > 1` alone would be cheaper but
//! would also count links fl-modman did not make.)

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use super::Workspace;
use crate::adapters::fs::{file_id, walk_files};
use crate::error::{Result, io};
use crate::manifest::Package;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModUsage {
    pub id: String,
    pub enabled: bool,
    /// In the last complete generation (its files are in the game).
    pub applied: bool,
    pub files: u64,
    /// Every file under `mods/<id>/`.
    pub bytes: u64,
    /// Bytes that are also the game's placed file (hardlinked), so they cost nothing extra.
    pub shared: u64,
}

impl ModUsage {
    /// Bytes that exist only in the workspace.
    pub fn only_here(&self) -> u64 {
        self.bytes - self.shared
    }

    /// Disabled and not applied: `remove` may delete it (same rule as [`Workspace::removable`]).
    pub fn removable(&self) -> bool {
        !self.enabled && !self.applied
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Usage {
    /// In list order.
    pub mods: Vec<ModUsage>,
    /// All of `mods/`, including folders that are not in the list.
    pub mods_dir: u64,
    pub originals: u64,
    pub history: u64,
    /// Leftovers of failed imports.
    pub staging: u64,
    /// The whole workspace folder.
    pub total: u64,
    /// Workspace and game on one filesystem; otherwise apply copies and every applied byte
    /// exists twice.
    pub same_fs: bool,
}

impl Usage {
    /// What `remove --disabled` (GUI: Free) deletes: removable mods plus `staging/`.
    pub fn reclaimable(&self) -> u64 {
        let mods = self.mods.iter().filter(|m| m.removable());
        mods.map(|m| m.bytes).sum::<u64>() + self.staging
    }
}

/// Hardlinks need one filesystem: the workspace and the game share a device id.
pub fn same_filesystem(workspace_dev: u64, game_dev: u64) -> bool {
    workspace_dev == game_dev
}

/// Regular-file bytes under `dir` (0 if it does not exist); symlinks are not followed.
fn du(dir: &Path) -> Result<u64> {
    let mut n = 0;
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(0);
    };
    for e in entries {
        let p = e.map_err(io(dir))?.path();
        let m = fs::symlink_metadata(&p).map_err(io(&p))?;
        n += match () {
            _ if m.is_dir() => du(&p)?,
            _ if m.is_file() => m.len(),
            _ => 0,
        };
    }
    Ok(n)
}

impl Workspace {
    pub fn usage(&self) -> Result<Usage> {
        let applied: BTreeSet<String> = self
            .last()?
            .map(|g| g.mods)
            .unwrap_or_default()
            .into_iter()
            .collect();
        let lstat = |p: &Path| fs::symlink_metadata(p).map_err(io(p));
        let mut u = Usage {
            same_fs: same_filesystem(
                file_id(&lstat(&self.dir)?).0,
                file_id(&lstat(&self.state.game)?).0,
            ),
            ..Default::default()
        };
        for id in &self.state.mods {
            let dir = self.dir.join("mods").join(id);
            let mut m = ModUsage {
                id: id.clone(),
                enabled: !self.state.disabled.contains(id),
                applied: applied.contains(id),
                ..Default::default()
            };
            // Package-relative path -> (file id, size).
            let mut payload = BTreeMap::new();
            if dir.is_dir() {
                let mut visit = |rel: &str, _: &Path, meta: &fs::Metadata| {
                    (m.files, m.bytes) = (m.files + 1, m.bytes + meta.len());
                    payload.insert(rel.to_string(), (file_id(meta), meta.len()));
                    Ok(())
                };
                walk_files(&dir, "", &mut visit, &mut Vec::new())?;
            }
            if m.applied && dir.is_dir() {
                let pkg = Package::load(&dir, &self.t)?;
                let mut shared = BTreeSet::new();
                for (path, put) in &pkg.puts {
                    let Some((fid, _)) = payload.get(&put.source) else {
                        continue;
                    };
                    let placed = fs::symlink_metadata(self.state.game.join(path));
                    if placed.is_ok_and(|p| file_id(&p) == *fid) {
                        shared.insert(&put.source);
                    }
                }
                m.shared = shared.iter().map(|s| payload[*s].1).sum();
            }
            u.mods.push(m);
        }
        let listed: BTreeMap<&str, u64> = u.mods.iter().map(|m| (m.id.as_str(), m.bytes)).collect();
        for e in fs::read_dir(&self.dir).map_err(io(&self.dir))? {
            let p = e.map_err(io(&self.dir))?.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            let meta = lstat(&p)?;
            let bytes = match name {
                "mods" if meta.is_dir() => {
                    let mut n = 0;
                    for e in fs::read_dir(&p).map_err(io(&p))? {
                        let d = e.map_err(io(&p))?.path();
                        let id = d.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                        n += match listed.get(id) {
                            Some(b) => *b,
                            None if lstat(&d)?.is_dir() => du(&d)?,
                            None => lstat(&d)?.len(),
                        };
                    }
                    u.mods_dir = n;
                    n
                }
                _ if meta.is_dir() => du(&p)?,
                _ if meta.is_file() => meta.len(),
                _ => 0,
            };
            match name {
                "originals" => u.originals = bytes,
                "history" => u.history = bytes,
                "staging" => u.staging = bytes,
                _ => {}
            }
            u.total += bytes;
        }
        Ok(u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_devices_are_different_filesystems() {
        assert!(same_filesystem(64769, 64769));
        assert!(!same_filesystem(64769, 2049));
    }

    #[test]
    fn reclaimable_counts_disabled_unapplied_mods_and_staging() {
        let m = |id: &str, enabled, applied, bytes| ModUsage {
            id: id.into(),
            enabled,
            applied,
            bytes,
            ..Default::default()
        };
        let u = Usage {
            mods: vec![
                m("on", true, true, 1),
                m("off-applied", false, true, 10),
                m("off", false, false, 100),
                m("on-new", true, false, 1000),
            ],
            staging: 5,
            ..Default::default()
        };
        assert_eq!(u.reclaimable(), 105);
    }
}
