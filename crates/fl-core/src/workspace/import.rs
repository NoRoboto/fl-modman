//! Import (grilling Q4): archive or folder → staging → mod root (single wrapper folders are
//! descended, readme/images ignored, FL layout matched case-insensitively) → a kind (`kinds`) →
//! `mods/<id>/` with its `mod.toml`, generated or given, enabled at the bottom of the list.

use std::fs;
use std::path::{Path, PathBuf};

use super::Workspace;
use super::index::write_index;
use crate::HOST;
use crate::adapters::fs::walk_files;
use crate::base::Snapshot;
use crate::build::{Conflict, build};
use crate::error::{Error, Result, input, io};
use crate::kinds::{self, JUNK, Kind, Listing, Root};
use crate::manifest::{FILE_NAME, Package};
use crate::paths::{self, ext};
use crate::target::Target;

/// What to import with; every field is optional.
#[derive(Clone, Copy, Default)]
pub struct Options<'a> {
    /// `--type`: the kind, instead of detecting it.
    pub kind: Option<Kind>,
    /// `--team <id>[,<slot>]`: the team of a stadium.
    pub team: Option<&'a str>,
    /// `--root`: the mod root, relative to the archive.
    pub root: Option<&'a str>,
    /// `--manifest`: a `mod.toml` (from `catalog/`) to use instead of a generated one.
    pub manifest: Option<&'a Path>,
}

pub struct Imported {
    pub id: String,
    /// The mod root, relative to the archive.
    pub root: String,
    pub kind: Kind,
    /// Tree paths the mod puts.
    pub files: Vec<String>,
    /// Conflicts with the enabled mods; the new mod is the lowest, so it wins.
    pub conflicts: Vec<Conflict>,
}

/// The entries of `dir` that matter for detection.
fn listing<'a>(dir: &Path, base: &'a Snapshot, t: &'a Target) -> Result<Listing<'a>> {
    let (mut files, mut dirs) = (Vec::new(), Vec::new());
    for e in fs::read_dir(dir).map_err(io(dir))? {
        let (e, name) = e.map(|e| (e.path(), e.file_name())).map_err(io(dir))?;
        let name = name.to_string_lossy().into_owned();
        if e.is_dir() && name != "__MACOSX" {
            dirs.push(name);
        } else if e.is_file() && !JUNK.contains(&ext(&name).as_str()) {
            files.push(name);
        }
    }
    files.sort();
    dirs.sort();
    Ok(Listing {
        files,
        dirs,
        base,
        t,
    })
}

/// Descends single wrapper folders until a level classifies; else lists the candidates.
fn find_root(top: &Path, base: &Snapshot, t: &Target) -> Result<PathBuf> {
    let mut dir = top.to_path_buf();
    loop {
        let l = listing(&dir, base, t)?;
        if kinds::detect(&l).is_some() {
            return Ok(dir);
        }
        match (l.files.is_empty(), l.dirs.as_slice()) {
            (true, [only]) => dir.push(only),
            _ => break,
        }
    }
    let (mut found, mut todo) = (Vec::new(), vec![(top.to_path_buf(), 0)]);
    while let Some((dir, depth)) = todo.pop() {
        let l = listing(&dir, base, t)?;
        match kinds::detect(&l) {
            Some(k) => found.push((rel(top, &dir), k)),
            None if depth < 4 => todo.extend(l.dirs.iter().map(|d| (dir.join(d), depth + 1))),
            None => {}
        }
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    Err(Error::AmbiguousRoot(found))
}

fn rel(top: &Path, dir: &Path) -> String {
    let r = dir.strip_prefix(top).unwrap_or(dir).to_string_lossy();
    if r.is_empty() {
        ".".into()
    } else {
        r.into_owned()
    }
}

/// Imports an archive or folder as `mods/<id>/` (id from its name) and enables it at the
/// bottom of the list. The mod is built in `staging/` and moved into `mods/` only once it loads;
/// a given manifest must find every `put.from` in the payload.
pub fn import(ws: &mut Workspace, src: &Path, opt: Options) -> Result<Imported> {
    let stem = if src.is_dir() {
        src.file_name()
    } else {
        src.file_stem()
    };
    let stem = stem.map_or(String::new(), |s| s.to_string_lossy().into_owned());
    let words = stem.to_lowercase();
    let words: Vec<&str> = words.split(|c: char| !c.is_alphanumeric()).collect();
    let id = words
        .into_iter()
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let (staging, dest) = (ws.dir.join("staging"), ws.dir.join("mods").join(&id));
    if id.is_empty() || dest.exists() || ws.state.mods.contains(&id) {
        return Err(input(format!("mod {id:?} already exists or has no name")));
    }
    let _ = fs::remove_dir_all(&staging);
    let (top, out) = (staging.join("archive"), staging.join("mod").join(&id));
    let top = if src.is_dir() {
        src.to_path_buf()
    } else {
        HOST.extract(src, &top).map(|()| top)?
    };
    let base = ws.base()?;
    let dir = match opt.root {
        Some(r) => top.join(paths::normalize(r)?),
        None => find_root(&top, &base, &ws.t)?,
    };
    let kind = opt.kind.or(kinds::detect(&listing(&dir, &base, &ws.t)?));
    let kind = kind.ok_or_else(|| input("cannot tell the mod type; pass --type"))?;
    let name = dir.file_name().filter(|_| dir != top);
    let name = name.map_or(stem.clone(), |n| n.to_string_lossy().into_owned());

    let mut files = Vec::new();
    let mut add = |rel: &str, _: &Path, _: &fs::Metadata| {
        let junk = !rel.contains('/') && JUNK.contains(&ext(rel).as_str());
        if !junk && !rel.starts_with("__MACOSX/") {
            files.push(rel.to_string());
        }
        Ok(())
    };
    let mut skipped = Vec::new();
    walk_files(&dir, "", &mut add, &mut skipped)?;
    if let Some(s) = skipped.first() {
        return Err(input(format!(
            "the archive holds a link or special file: {s}"
        )));
    }
    let (t, team) = (&ws.t, opt.team);
    let (name, files, base) = (name.as_str(), files.as_slice(), &base);
    let c = kind.contribute(&Root {
        name,
        files,
        base,
        t,
        team,
    });
    let toml = match (opt.manifest, team) {
        (Some(_), Some(_)) => {
            return Err(input(
                "--team does nothing with --manifest (its rows are used)",
            ));
        }
        (Some(m), None) => fs::read_to_string(m).map_err(io(m))?,
        (None, _) => {
            let m = c.manifest?;
            if team.is_some() && m.map_teams.is_empty() {
                return Err(input(format!(
                    "--team is for stadiums, and this mod is {kind:?}: pass --type stadium if it is one"
                )));
            }
            toml::to_string(&m).expect("manifest serializes")
        }
    };
    for (from, to) in &c.files {
        let (from, to) = (dir.join(from), out.join(to));
        fs::create_dir_all(to.parent().expect("file")).map_err(io(&out))?;
        fs::copy(&from, &to).map_err(io(&from))?;
    }
    fs::write(out.join(FILE_NAME), toml).map_err(io(&out))?;
    write_index(&out)?;
    let mut pkgs = ws.packages()?;
    pkgs.insert(id.clone(), Package::load(&out, &ws.t)?);
    let list: Vec<String> = ws.enabled().into_iter().chain([id.clone()]).collect();
    let conflicts = build(&ws.t, base, &pkgs, &list)?.conflicts.into_iter();
    let conflicts = conflicts
        .filter(|c| c.winner == id || c.loser == id)
        .collect();
    fs::rename(&out, &dest).map_err(io(&dest))?;
    let _ = fs::remove_dir_all(&staging);
    ws.state.mods.push(id.clone());
    ws.save()?;
    let (files, root) = (pkgs[&id].puts.keys().cloned().collect(), rel(&top, &dir));
    Ok(Imported {
        id,
        root,
        kind,
        files,
        conflicts,
    })
}
