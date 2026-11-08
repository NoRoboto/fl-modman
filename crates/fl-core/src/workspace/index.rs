//! Mod packages on disk: `mods/<id>/mod.toml` plus the payload index `index.json` (path →
//! BLAKE3 + size, written once at import, Q20a). Plan and conflicts read the index and never
//! rehash payloads; `check` does.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::Workspace;
use crate::adapters::fs::{b3_file, read_json, read_toml, walk_files, write_atomic};
use crate::base::{FileHash, TreeManifest};
use crate::error::{Error, Result, io};
use crate::manifest::{FILE_NAME, INDEX, Package};
use crate::paths;
use crate::target::Target;

impl Package {
    /// Loads and validates `<dir>/mod.toml`, taking the payload hashes from `index.json` (no
    /// payload is read) or, without one, hashing the payload.
    pub fn load(dir: &Path, target: &Target) -> Result<Self> {
        let path = dir.join(FILE_NAME);
        let manifest = read_toml(&path)?;
        let id = dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::Input(format!("{}: bad mod folder name", dir.display())))?
            .to_string();
        let files = match read_json(&dir.join(INDEX))? {
            Some(files) => checked_index(&id, files)?,
            None => hash_payload(dir)?,
        };
        let fail = |why| Error::Manifest(format!("{id} ({}): {why}", path.display()));
        Package::new(id.clone(), dir.into(), manifest, &files, target).map_err(fail)
    }

    /// Loads every `<mods_dir>/*/mod.toml`.
    pub fn load_all(mods_dir: &Path, target: &Target) -> Result<BTreeMap<String, Package>> {
        let mut dirs: Vec<PathBuf> = fs::read_dir(mods_dir)
            .map_err(io(mods_dir))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.join(FILE_NAME).is_file())
            .collect();
        dirs.sort();
        dirs.iter()
            .map(|d| Package::load(d, target).map(|p| (p.id.clone(), p)))
            .collect()
    }
}

/// Every index key must already be a normalized relative path (no `..`, no absolute path, no
/// `\` or `.` parts): an edited `index.json` must not point outside the mod folder.
fn checked_index(id: &str, files: TreeManifest) -> Result<TreeManifest> {
    for key in files.keys() {
        if paths::normalize(key).ok().as_deref() != Some(key.as_str()) {
            return Err(Error::Manifest(format!(
                "{id}: {INDEX} key {key:?} is not a normalized relative path"
            )));
        }
    }
    Ok(files)
}

/// Hashes every payload file of the package folder `dir` (all but `mod.toml` and `index.json`).
pub fn hash_payload(dir: &Path) -> Result<TreeManifest> {
    let mut files = TreeManifest::new();
    let mut visit = |rel: &str, p: &Path, _: &fs::Metadata| {
        if rel != FILE_NAME && rel != INDEX {
            let (b3, size) = b3_file(p)?;
            files.insert(rel.to_string(), FileHash { b3, size });
        }
        Ok(())
    };
    walk_files(dir, "", &mut visit, &mut Vec::new())?;
    Ok(files)
}

/// Hashes the payload of `dir` into `index.json` (mods are immutable once imported).
pub fn write_index(dir: &Path) -> Result<TreeManifest> {
    let files = hash_payload(dir)?;
    let json = serde_json::to_vec_pretty(&files).expect("index serializes");
    write_atomic(&dir.join(INDEX), &json)?;
    Ok(files)
}

impl Workspace {
    /// Loads every mod from its `index.json`; a mod imported before indexes existed gets one
    /// built (and saved) once.
    pub fn packages(&self) -> Result<BTreeMap<String, Package>> {
        let mods = self.dir.join("mods");
        for e in fs::read_dir(&mods).map_err(io(&mods))? {
            let d = e.map_err(io(&mods))?.path();
            if d.join(FILE_NAME).is_file() && !d.join(INDEX).exists() {
                write_index(&d)?;
            }
        }
        Package::load_all(&mods, &self.t)
    }

    /// Rehashes every mod payload against its index (read-only). Returns the number of files
    /// hashed and one line per mismatch.
    pub fn check_mods(&self) -> Result<(usize, Vec<String>)> {
        let (mut n, mut bad) = (0, Vec::new());
        for id in &self.state.mods {
            let dir = self.dir.join("mods").join(id);
            let Some(index) = read_json::<TreeManifest>(&dir.join(INDEX))? else {
                bad.push(format!("{id}: no {INDEX}"));
                continue;
            };
            let now = hash_payload(&dir)?;
            n += now.len();
            let paths: BTreeSet<&String> = index.keys().chain(now.keys()).collect();
            for p in paths {
                let why = match (index.get(p), now.get(p)) {
                    (Some(_), None) => "missing",
                    (None, Some(_)) => "not in the index",
                    (Some(a), Some(b)) if a != b => "content changed",
                    _ => continue,
                };
                bad.push(format!("{id}: {p}: {why}"));
            }
        }
        Ok((n, bad))
    }
}
