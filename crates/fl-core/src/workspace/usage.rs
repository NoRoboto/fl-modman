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
use std::time::{Duration, SystemTime};

use super::{Workspace, check_id};
use crate::adapters::fs::{file_id, remove_path, walk_files};
use crate::error::{Result, input, io};
use crate::manifest::Package;

/// `staging/` is not wiped while anything in it changed this recently (an import may be running).
pub const STAGING_QUIET: Duration = Duration::from_secs(600);

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
    /// Folders under `mods/` that are not in the list (left by a crash or by hand), with bytes.
    pub orphans: Vec<(String, u64)>,
    /// All of `mods/`, including the orphans.
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
    /// What could not be measured (a symlinked or missing mod folder, a broken mod, no game).
    pub notes: Vec<String>,
}

impl Usage {
    /// What `remove --disabled` (GUI: Free) can delete: removable mods, orphans, `staging/`.
    pub fn reclaimable(&self) -> u64 {
        let mods = self.mods.iter().filter(|m| m.removable());
        let orphans = self.orphans.iter().map(|o| o.1).sum::<u64>();
        mods.map(|m| m.bytes).sum::<u64>() + orphans + self.staging
    }

    /// The ids `remove_disabled` takes for everything [`Usage::reclaimable`] counts.
    pub fn reclaimable_ids(&self) -> Vec<String> {
        let mods = self
            .mods
            .iter()
            .filter(|m| m.removable())
            .map(|m| m.id.clone());
        mods.chain(self.orphans.iter().map(|o| o.0.clone()))
            .collect()
    }
}

/// What [`Workspace::remove_disabled`] did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Freed {
    /// Mods and orphan folders deleted, and `staging/` when it was wiped.
    pub removed: Vec<String>,
    /// Asked for but kept, as "<id>: <reason>".
    pub kept: Vec<String>,
    /// The bytes of what was removed.
    pub bytes: u64,
}

impl Freed {
    fn record(&mut self, item: &str, r: Result<u64>) {
        match r {
            Ok(b) => {
                self.bytes += b;
                self.removed.push(item.into());
            }
            Err(e) => self.kept.push(format!("{item}: {e}")),
        }
    }
}

/// Hardlinks need one filesystem: the workspace and the game share a device id.
pub fn same_filesystem(workspace_dev: u64, game_dev: u64) -> bool {
    workspace_dev == game_dev
}

/// Regular-file bytes at or under `p` (0 if missing); symlinks are not followed.
fn du(p: &Path) -> Result<u64> {
    let Ok(m) = fs::symlink_metadata(p) else {
        return Ok(0);
    };
    if !m.is_dir() {
        return Ok(if m.is_file() { m.len() } else { 0 });
    }
    let mut n = 0;
    for e in fs::read_dir(p).map_err(io(p))? {
        n += du(&e.map_err(io(p))?.path())?;
    }
    Ok(n)
}

/// True if `p` or anything under it (symlinks not followed) was modified within `quiet`.
fn recent(p: &Path, quiet: Duration) -> Result<bool> {
    let m = fs::symlink_metadata(p).map_err(io(p))?;
    let cut = SystemTime::now() - quiet;
    if m.modified().is_ok_and(|t| t > cut) {
        return Ok(true);
    }
    if m.is_dir() {
        for e in fs::read_dir(p).map_err(io(p))? {
            if recent(&e.map_err(io(p))?.path(), quiet)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

impl Workspace {
    pub fn usage(&self) -> Result<Usage> {
        self.state.mods.iter().try_for_each(|m| check_id(m))?;
        let last = self.last()?.map(|g| g.mods).unwrap_or_default();
        let applied: BTreeSet<String> = last.into_iter().collect();
        let lstat = |p: &Path| fs::symlink_metadata(p).map_err(io(p));
        let mut u = Usage::default();
        let ws_dev = file_id(&lstat(&self.dir)?).0;
        u.same_fs = match lstat(&self.state.game) {
            Ok(g) => same_filesystem(ws_dev, file_id(&g).0),
            Err(_) => {
                let game = self.state.game.display();
                u.notes
                    .push(format!("game folder {game} not found; shared bytes are 0"));
                true
            }
        };
        for id in &self.state.mods {
            let dir = self.dir.join("mods").join(id);
            let mut m = ModUsage {
                id: id.clone(),
                enabled: !self.state.disabled.contains(id),
                applied: applied.contains(id),
                ..Default::default()
            };
            let is_dir = match fs::symlink_metadata(&dir) {
                Ok(meta) if meta.is_dir() => true,
                Ok(_) => {
                    u.notes.push(format!(
                        "mods/{id} is not a folder (a symlink?); not measured"
                    ));
                    false
                }
                Err(_) => {
                    u.notes.push(format!("mods/{id} is missing"));
                    false
                }
            };
            // Package-relative path -> (file id, size).
            let mut payload = BTreeMap::new();
            if is_dir {
                let mut visit = |rel: &str, _: &Path, meta: &fs::Metadata| {
                    (m.files, m.bytes) = (m.files + 1, m.bytes + meta.len());
                    payload.insert(rel.to_string(), (file_id(meta), meta.len()));
                    Ok(())
                };
                walk_files(&dir, "", &mut visit, &mut Vec::new())?;
            }
            if m.applied && is_dir {
                match Package::load(&dir, &self.t) {
                    Ok(pkg) => {
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
                    Err(e) => u.notes.push(format!("{id}: shared bytes unknown: {e}")),
                }
            }
            u.mods.push(m);
        }
        let listed: BTreeMap<&str, u64> = u.mods.iter().map(|m| (m.id.as_str(), m.bytes)).collect();
        let mut orphans = Vec::new();
        for e in fs::read_dir(&self.dir).map_err(io(&self.dir))? {
            let p = e.map_err(io(&self.dir))?.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            let bytes = match name {
                "mods" if lstat(&p)?.is_dir() => {
                    let mut n = 0;
                    for e in fs::read_dir(&p).map_err(io(&p))? {
                        let d = e.map_err(io(&p))?.path();
                        let id = d.file_name().and_then(|n| n.to_str());
                        let b = match id.and_then(|id| listed.get(id)) {
                            Some(b) => *b,
                            None => du(&d)?,
                        };
                        let orphan =
                            id.filter(|id| !listed.contains_key(id) && check_id(id).is_ok());
                        if let Some(id) = orphan.filter(|_| lstat(&d).is_ok_and(|m| m.is_dir())) {
                            orphans.push((id.to_string(), b));
                        }
                        n += b;
                    }
                    u.mods_dir = n;
                    n
                }
                _ => du(&p)?,
            };
            match name {
                "originals" => u.originals = bytes,
                "history" => u.history = bytes,
                "staging" => u.staging = bytes,
                _ => {}
            }
            u.total += bytes;
        }
        orphans.sort();
        u.orphans = orphans;
        Ok(u)
    }

    /// Deletes exactly `ids` (as listed to and confirmed by the user, e.g.
    /// [`Usage::reclaimable_ids`]), each only if it is still [`Workspace::removable`] or an
    /// orphan folder, and `staging/` if `staging`. Refused while the game runs or an apply is
    /// interrupted. A failure keeps that item and the rest go on; symlinks lose only the link.
    pub fn remove_disabled(&mut self, ids: &[String], staging: bool) -> Result<Freed> {
        self.refuse_running()?;
        self.refuse_pending()?;
        let removable = self.removable()?;
        let orphans: Vec<String> = self.usage()?.orphans.into_iter().map(|o| o.0).collect();
        let mut f = Freed::default();
        for id in ids {
            let r = check_id(id).and_then(|()| {
                let dir = self.dir.join("mods").join(id);
                let bytes = du(&dir)?;
                match () {
                    _ if removable.contains(id) => self.remove(id)?,
                    _ if orphans.contains(id) => remove_path(&dir)?,
                    _ => return Err(input("no longer removable (enabled or applied)")),
                }
                Ok(bytes)
            });
            f.record(id, r);
        }
        if staging {
            let r = self.wipe_staging();
            f.record("staging/", r);
        }
        Ok(f)
    }

    /// Wipes `staging/` unless something in it changed within [`STAGING_QUIET`]. A symlinked
    /// `staging` loses only the link.
    fn wipe_staging(&self) -> Result<u64> {
        let s = self.dir.join("staging");
        let link = fs::symlink_metadata(&s).is_ok_and(|m| m.file_type().is_symlink());
        if !link && s.exists() && recent(&s, STAGING_QUIET)? {
            return Err(input(
                "changed in the last 10 minutes (an import may be running); kept",
            ));
        }
        let bytes = du(&s)?;
        remove_path(&s)?;
        Ok(bytes)
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
