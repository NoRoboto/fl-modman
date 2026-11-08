//! Tree hash manifests and the base snapshot: the first-run scan of the game dir, with any
//! hand-installed mods frozen inside it (grilling Q8). Scanning is `workspace::scan`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::codec::DocText;
use crate::hash;

pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileHash {
    pub b3: String,
    pub size: u64,
}

/// Path (relative, `/`-separated) → content hash. User paths are never included.
pub type TreeManifest = BTreeMap<String, FileHash>;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub schema: u32,
    /// BLAKE3 of the file manifest, shortened; stable for identical trees.
    pub id: String,
    pub target: String,
    /// SHA-256 of the game exe (the `exe` guard of `mod.toml` compares against it).
    pub exe_sha256: Option<String>,
    /// Contents of the target's `version_file`.
    pub version: Option<String>,
    pub files: TreeManifest,
    /// Raw text of the managed documents present in the tree.
    pub docs: BTreeMap<String, DocText>,
    /// Entries that were skipped (symlinks, special files), for the user to review.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped: Vec<String>,
}

pub fn manifest_id(files: &TreeManifest) -> String {
    let canonical = serde_json::to_vec(files).expect("manifest serializes");
    hash::b3_bytes(&canonical)[..16].to_string()
}

/// Reads a version from `version.ini`/`version.txt`: the first meaningful line, either a bare
/// value (`27.1.0`) or `key = value`.
pub fn parse_version_file(text: &str) -> Option<String> {
    text.lines()
        .map(|l| l.trim_start_matches('\u{feff}').trim())
        .find(|l| !l.is_empty() && !l.starts_with(['[', ';', '#']))
        .map(|l| l.split_once('=').map_or(l, |(_, v)| v.trim()))
        .map(|v| v.trim_matches('"').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_file_forms() {
        assert_eq!(parse_version_file("27.1.0\r\n").as_deref(), Some("27.1.0"));
        assert_eq!(
            parse_version_file("[ver]\nversion = \"2.2.0\"\n").as_deref(),
            Some("2.2.0")
        );
        assert_eq!(parse_version_file("\n"), None);
    }
}
