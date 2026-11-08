//! `mod.toml` and mod packages (schema: docs/mod-toml.md).
//!
//! A package is a folder `<mods>/<id>/` holding `mod.toml` and the payload; the folder name is
//! the mod id and `from` paths are relative to the folder. [`Package::new`] validates the
//! manifest against the target and resolves every put to concrete tree paths; loading it from
//! disk is `workspace::index`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::base::{FileHash, TreeManifest};
use crate::codec::map_teams;
use crate::error::{Error, Result};
use crate::paths;
use crate::target::Target;

pub const FILE_NAME: &str = "mod.toml";
/// Payload index of an imported mod: path → BLAKE3 + size, written once at import.
pub const INDEX: &str = "index.json";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub name: Option<String>,
    pub version: Option<String>,
    /// SHA-256 of the only game exe this mod works with; on any other exe it is skipped.
    pub exe: Option<String>,
    /// `sider.ini` `cpk.root` values (`.\livecpk\Name`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cpk_root: Vec<String>,
    /// `sider.ini` `lua.module` values (`Name.lua`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lua_module: Vec<String>,
    /// `map_teams.txt` rows (`teamId, slot, name, folder # comment`), at most one per team.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub map_teams: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub put: Vec<Put>,
}

/// Copies a payload file, or a folder when `from` ends in `/`, to `@root/path`. A `to` ending
/// in `/` is a folder that receives the file (or the folder's contents).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Put {
    pub from: String,
    pub to: String,
}

/// A file a package puts into the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PutFile {
    pub hash: FileHash,
    /// Package-relative source path.
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub id: String,
    pub dir: PathBuf,
    pub manifest: Manifest,
    /// Tree path → file. Later puts overlay earlier ones.
    pub puts: BTreeMap<String, PutFile>,
}

impl Package {
    /// Validates `manifest` against the target and resolves its puts over the payload `files`
    /// (package-relative). The error is the reason only; the caller names the mod.
    pub fn new(
        id: String,
        dir: PathBuf,
        mut manifest: Manifest,
        files: &TreeManifest,
        target: &Target,
    ) -> Result<Self, String> {
        let puts = resolve_puts(&manifest.put, files, target).map_err(|e| e.to_string())?;
        if let Some(p) = puts
            .keys()
            .find(|p| target.is_doc(p) || target.is_user_path(p))
        {
            return Err(format!(
                "put on {p}: managed document or user file (use cpk_root / lua_module / map_teams)"
            ));
        }
        for v in manifest.cpk_root.iter().chain(&manifest.lua_module) {
            if v.contains(['"', '\r', '\n']) || v.trim().is_empty() {
                return Err(format!("{v:?} is not a valid sider.ini value"));
            }
        }
        if let Some(v) = manifest.cpk_root.iter().find(|v| !v.starts_with(".\\")) {
            return Err(format!(
                "cpk_root {v:?} must be relative (.\\…); absolute and Z:\\ paths break under Wine"
            ));
        }
        let mut teams = Vec::new();
        for row in &manifest.map_teams {
            match map_teams::parse_row(row) {
                Some(f) if !row.contains(['\r', '\n']) => {
                    let key = map_teams::normalize_key(&f[0]);
                    if teams.contains(&key) {
                        return Err(format!("team {key} has two map_teams rows"));
                    }
                    teams.push(key);
                }
                _ => return Err(format!("map_teams row {row:?} needs >= 4 fields")),
            }
        }
        if let Some(exe) = &mut manifest.exe {
            *exe = exe.to_ascii_lowercase();
        }
        Ok(Package {
            id,
            dir,
            manifest,
            puts,
        })
    }
}

fn resolve_puts(
    ops: &[Put],
    files: &TreeManifest,
    target: &Target,
) -> Result<BTreeMap<String, PutFile>> {
    let mut out = BTreeMap::new();
    for op in ops {
        let folder = op.from.ends_with('/') || op.from.ends_with('\\');
        let into_dir = op.to.ends_with('/');
        let from = paths::normalize(&op.from)?;
        let to = target.resolve(&op.to)?;
        let sources: Vec<(&String, String)> = if folder {
            if !into_dir {
                return Err(Error::Input(format!(
                    "put {:?}: a folder needs `to` ending in '/'",
                    op.from
                )));
            }
            files
                .keys()
                .filter(|k| k.len() > from.len() && paths::is_within(k, &from))
                .map(|k| (k, paths::join(&to, k[from.len()..].trim_start_matches('/'))))
                .collect()
        } else {
            let name = from.rsplit('/').next().unwrap_or(&from);
            let dest = if into_dir {
                paths::join(&to, name)
            } else {
                to.clone()
            };
            files
                .get_key_value(&from)
                .map(|(k, _)| (k, dest))
                .into_iter()
                .collect()
        };
        if sources.is_empty() {
            return Err(Error::Input(format!(
                "from {:?} matches no file in the package",
                op.from
            )));
        }
        for (src, dest) in sources {
            let pf = PutFile {
                hash: files[src].clone(),
                source: src.clone(),
            };
            out.insert(dest, pf);
        }
    }
    Ok(out)
}

/// Normalizes a `sider.ini` value for comparison: Windows paths, case-insensitive, either
/// separator.
pub fn ini_key(value: &str) -> String {
    value.trim().replace('/', "\\").to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn fixture(id: &str) -> Package {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/mods")
            .join(id);
        Package::load(&dir, &Target::fl27()).unwrap()
    }

    #[test]
    fn folder_and_file_puts_resolve() {
        let p = fixture("mariano-closs-v5");
        assert!(
            p.puts
                .contains_key("SiderAddons/livecpk/Mariano Closs V5/eng/sound/acb/00_TEAM.acb")
        );
        assert_eq!(p.puts.len(), 7);
        let cam = fixture("camera-epl");
        assert_eq!(cam.puts.len(), 8);
        assert!(
            cam.puts
                .contains_key("SiderAddons/modules/BroadCastCam.ini")
        );
        assert_eq!(ini_key(r".\LiveCpk/X"), r".\livecpk\x");
    }
}
