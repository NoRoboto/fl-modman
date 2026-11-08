//! Game discovery (grilling Q10): a bounded scan of the usual Linux places (~/Games for
//! Lutris/Heroic/umu, Steam libraries and their compatdata prefixes) for the game exe and for
//! Wine prefixes holding `SPGAME/Football Life`. The user always picks.

use std::fs;
use std::path::{Path, PathBuf};

use crate::settings::fl_docs;

/// The Proton launch options FL27 needs (grilling Q18).
pub const OVERRIDES: &str = "lsteamclient=d;steamclient64=n;steam_api64=n,b;winmm=n,b";

/// The Steam config (`shortcuts.vdf` or `localconfig.vdf`) that holds [`OVERRIDES`], if any.
/// Read only: fl-modman never edits Steam files.
pub fn overrides_set(home: &Path) -> Option<PathBuf> {
    let users = fs::read_dir(home.join(".local/share/Steam/userdata"));
    let config = |u: fs::DirEntry| {
        ["shortcuts.vdf", "localconfig.vdf"].map(|f| u.path().join("config").join(f))
    };
    let set = |f: &PathBuf| {
        fs::read(f).is_ok_and(|b| {
            b.windows(OVERRIDES.len())
                .any(|w| w == OVERRIDES.as_bytes())
        })
    };
    users
        .into_iter()
        .flatten()
        .flatten()
        .flat_map(config)
        .find(set)
}

/// Never descended: Windows system folders, prefix links, nested Steam libraries.
const SKIP: [&str; 4] = ["windows", "dosdevices", "shadercache", "steamapps"];

#[derive(Default)]
pub struct Found {
    /// Game folders, with the prefix they live in when they are inside one.
    pub games: Vec<(PathBuf, Option<PathBuf>)>,
    /// Prefixes holding `Documents/SPGAME/Football Life`.
    pub prefixes: Vec<PathBuf>,
}

pub fn detect(home: &Path, exe: &str) -> Found {
    let steam = [
        ".local/share/Steam",
        ".steam/steam",
        ".var/app/com.valvesoftware.Steam/.local/share/Steam",
    ];
    let mut libs: Vec<PathBuf> = steam
        .iter()
        .map(|s| home.join(s).join("steamapps"))
        .collect();
    for s in steam {
        let vdf = fs::read_to_string(home.join(s).join("steamapps/libraryfolders.vdf"));
        for line in vdf.unwrap_or_default().lines() {
            let f: Vec<&str> = line.split('"').collect();
            if f.len() >= 4 && f[1] == "path" {
                libs.push(Path::new(f[3]).join("steamapps"));
            }
        }
    }
    let mut libs: Vec<PathBuf> = libs
        .iter()
        .filter_map(|l| fs::canonicalize(l).ok())
        .collect();
    libs.sort();
    libs.dedup();
    let mut found = Found::default();
    walk(&home.join("Games"), exe, 7, &mut found);
    for l in &libs {
        walk(&l.join("common"), exe, 2, &mut found);
        walk(&l.join("compatdata"), exe, 7, &mut found);
    }
    found.games.sort();
    found.prefixes.sort();
    for (g, pfx) in &mut found.games {
        *pfx = g
            .ancestors()
            .find(|a| a.join("drive_c").is_dir())
            .map(Path::to_path_buf);
    }
    found
}

/// Visits `dir` and its subfolders down to `depth`, collecting game folders and prefixes.
fn walk(dir: &Path, exe: &str, depth: u32, found: &mut Found) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let (mut subdirs, mut game) = (Vec::new(), false);
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_lowercase();
        game |= name.eq_ignore_ascii_case(exe);
        if e.file_type().is_ok_and(|t| t.is_dir()) && !SKIP.contains(&name.as_str()) {
            subdirs.push(e.path());
        }
    }
    if game {
        found.games.push((dir.into(), None));
    }
    if fl_docs(dir).is_some() {
        found.prefixes.push(dir.into());
    }
    if depth > 0 {
        subdirs.iter().for_each(|d| walk(d, exe, depth - 1, found));
    }
}
