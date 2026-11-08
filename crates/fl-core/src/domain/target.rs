//! The game target: roots, the two managed documents and user paths. FL27 only (grilling Q2);
//! other games later as more `targets/*.toml` files.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::paths;

const FL27: &str = include_str!("../../targets/fl27.toml");

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub id: String,
    pub exe: String,
    pub version_file: String,
    pub sider_dir: String,
    pub sider_ini: String,
    pub map_teams: String,
    /// Globs (relative to the game dir) the game or the user rewrite.
    pub user_paths: Vec<String>,
    /// Globs of placed files the game or Sider modules rewrite at runtime: never drift.
    pub drift_ignore: Vec<String>,
    /// Extensions of heavy assets never written by the game: hardlinked, not copied.
    pub hardlink_ext: Vec<String>,
    /// Top-level folders of livecpk content, for import root detection.
    pub livecpk_dirs: Vec<String>,
    pub roots: BTreeMap<String, String>,
}

impl Target {
    pub fn fl27() -> Self {
        toml::from_str(FL27).expect("built-in target parses")
    }

    /// Resolves `@root/rest` (or `@root`) to a normalized path relative to the game dir.
    pub fn resolve(&self, at: &str) -> Result<String> {
        let body = at
            .strip_prefix('@')
            .ok_or_else(|| Error::Path(at.into(), "must start with @<root>"))?;
        let (root, rest) = body.split_once('/').unwrap_or((body, ""));
        let base = self
            .roots
            .get(root)
            .ok_or_else(|| Error::Path(at.into(), "unknown root"))?;
        Ok(paths::join(
            &paths::normalize(base)?,
            &paths::normalize(rest)?,
        ))
    }

    /// The managed documents; mods edit them through `cpk_root`, `lua_module`, `map_teams`.
    pub fn is_doc(&self, rel: &str) -> bool {
        rel == self.sider_ini || rel == self.map_teams
    }

    pub fn is_user_path(&self, rel: &str) -> bool {
        self.user_paths.iter().any(|g| paths::glob_match(g, rel))
    }

    pub fn drift_ignored(&self, rel: &str) -> bool {
        self.drift_ignore.iter().any(|g| paths::glob_match(g, rel))
    }

    pub fn hardlink(&self, rel: &str) -> bool {
        let ext = rel.rsplit_once('.').map_or("", |(_, e)| e);
        self.hardlink_ext
            .iter()
            .any(|e| e.eq_ignore_ascii_case(ext))
    }

    /// Resolves a `cpk.root` value (`.\livecpk\X`) to a tree path under the Sider folder.
    pub fn cpk_root_dir(&self, value: &str) -> Result<String> {
        Ok(paths::join(
            &paths::normalize(&self.sider_dir)?,
            &paths::normalize(value)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fl27_resolves_roots_and_paths() {
        let t = Target::fl27();
        assert_eq!(
            t.resolve("@stadiums/Camp Nou 2026/").unwrap(),
            "SiderAddons/content/stadiums/Camp Nou 2026"
        );
        assert_eq!(t.resolve("@game/dxgi.dll").unwrap(), "dxgi.dll");
        assert!(t.resolve("@nope/x").is_err());
        assert!(t.resolve("@game/../x").is_err());
        assert!(t.is_user_path("SiderAddons/sider.log"));
        assert!(!t.is_user_path("SiderAddons/modules/BroadCastCam.ini"));
        assert_eq!(
            t.cpk_root_dir(r".\livecpk\Mariano Closs V5").unwrap(),
            "SiderAddons/livecpk/Mariano Closs V5"
        );
    }
}
