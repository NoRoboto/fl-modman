//! Build: the tree that the base plus an ordered mod list produces.
//!
//! The list is ascending priority: on any conflict the lower mod in the list wins (grilling
//! Q5), and every conflict is reported. Build is a pure function of its inputs (T5).

use std::collections::{BTreeMap, BTreeSet};

use crate::base::{FileHash, Snapshot, TreeManifest};
use crate::codec::map_teams::{self, MapTeams};
use crate::codec::sider_ini::SiderIni;
use crate::codec::{DocText, Encoding};
use crate::error::{Error, Result};
use crate::hash;
use crate::manifest::{Package, ini_key};
use crate::paths;
use crate::target::Target;

/// One file of the desired tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provided {
    pub hash: FileHash,
    /// The mod providing it; `None` = the base.
    pub by: Option<String>,
}

/// A line a mod adds to, or removes from, a managed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocChange {
    pub added: bool,
    pub line: String,
    pub by: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedDoc {
    pub hash: FileHash,
    pub bytes: Vec<u8>,
    pub changes: Vec<DocChange>,
}

/// `winner` overrides `items` (files, virtual paths or team rows) of `loser`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub winner: String,
    pub loser: String,
    pub items: Vec<String>,
}

impl std::fmt::Display for Conflict {
    /// `X overrides N file(s) of Y`.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (w, n, l) = (&self.winner, self.items.len(), &self.loser);
        write!(f, "{w} overrides {n} file(s) of {l}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Desired {
    pub target: String,
    pub base_snapshot: String,
    /// Mods in effect, ascending priority (exe-guarded mods that do not match are dropped).
    pub mods: Vec<String>,
    pub files: BTreeMap<String, Provided>,
    pub docs: BTreeMap<String, RenderedDoc>,
    pub conflicts: Vec<Conflict>,
    pub warnings: Vec<String>,
}

impl Desired {
    /// The hash manifest of the desired tree (files and rendered documents).
    pub fn manifest(&self) -> TreeManifest {
        let files = self.files.iter().map(|(p, f)| (p.clone(), f.hash.clone()));
        let docs = self.docs.iter().map(|(p, d)| (p.clone(), d.hash.clone()));
        files.chain(docs).collect()
    }
}

/// `(winner rank, loser rank)` → overridden items.
type Conflicts = BTreeMap<(usize, usize), Vec<String>>;

/// Runs Build. Errors are bad input: an unknown or repeated mod, an unencodable document.
pub fn build(
    t: &Target,
    base: &Snapshot,
    packages: &BTreeMap<String, Package>,
    list: &[String],
) -> Result<Desired> {
    if base.target != t.id {
        return Err(Error::Input(format!(
            "base snapshot is for target {:?}, not {:?}",
            base.target, t.id
        )));
    }
    let mut warnings = Vec::new();
    let mut mods: Vec<&Package> = Vec::new();
    for (i, id) in list.iter().enumerate() {
        if list[..i].contains(id) {
            return Err(Error::Input(format!("{id:?} is listed twice")));
        }
        let pkg = packages
            .get(id)
            .ok_or_else(|| Error::Input(format!("unknown mod {id:?}")))?;
        match (&pkg.manifest.exe, &base.exe_sha256) {
            (Some(want), have) if Some(want) != have.as_ref() => warnings.push(format!(
                "{id} disabled: made for exe {}, this game's exe is {}",
                hash::short(want),
                have.as_deref().map_or("missing", hash::short)
            )),
            _ => mods.push(pkg),
        }
    }

    let mut conflicts = Conflicts::new();
    let files = files(t, base, &mods, &mut conflicts);
    let mut docs = BTreeMap::new();
    let mut roots = Vec::new();
    if let Some((text, enc)) = doc_source(
        base,
        &t.sider_ini,
        mods.iter()
            .any(|m| !m.manifest.cpk_root.is_empty() || !m.manifest.lua_module.is_empty()),
    ) {
        let mut ini = SiderIni::parse(text);
        let mut changes = Vec::new();
        // cpk.root lookup is first-wins (F1): the lower a mod, the earlier its roots.
        let new_roots = mods
            .iter()
            .rev()
            .flat_map(|m| m.manifest.cpk_root.iter().map(move |v| (&m.id, v)));
        roots = ordered(&mut ini, "cpk.root", new_roots, true, &mut changes);
        let new_modules = mods
            .iter()
            .flat_map(|m| m.manifest.lua_module.iter().map(move |v| (&m.id, v)));
        ordered(&mut ini, "lua.module", new_modules, false, &mut changes);
        docs.insert(
            t.sider_ini.clone(),
            render(&t.sider_ini, enc, &ini.render(), changes)?,
        );
    }
    let any_rows = mods.iter().any(|m| !m.manifest.map_teams.is_empty());
    if let Some((text, enc)) = doc_source(base, &t.map_teams, any_rows) {
        let mut mt = MapTeams::parse(text);
        let changes = teams(&mut mt, &mods, &mut conflicts);
        docs.insert(
            t.map_teams.clone(),
            render(&t.map_teams, enc, &mt.render(), changes)?,
        );
    }
    shadows(t, &roots, &files, &mods, &mut conflicts);

    Ok(Desired {
        target: t.id.clone(),
        base_snapshot: base.id.clone(),
        mods: mods.iter().map(|m| m.id.clone()).collect(),
        files,
        docs,
        conflicts: conflicts
            .into_iter()
            .map(|((w, l), items)| Conflict {
                winner: mods[w].id.clone(),
                loser: mods[l].id.clone(),
                items,
            })
            .collect(),
        warnings,
    })
}

/// Each path takes the value of its last provider in the list, else the base.
fn files(
    t: &Target,
    base: &Snapshot,
    mods: &[&Package],
    conflicts: &mut Conflicts,
) -> BTreeMap<String, Provided> {
    let mut out: BTreeMap<String, Provided> = base
        .files
        .iter()
        .filter(|(p, _)| !t.is_doc(p))
        .map(|(p, h)| {
            let f = Provided {
                hash: h.clone(),
                by: None,
            };
            (p.clone(), f)
        })
        .collect();
    let mut owner: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, m) in mods.iter().enumerate() {
        for (p, pf) in &m.puts {
            if let Some(prev) = owner.insert(p, i)
                && out[p].hash.b3 != pf.hash.b3
            {
                conflicts.entry((i, prev)).or_default().push(p.clone());
            }
            let f = Provided {
                hash: pf.hash.clone(),
                by: Some(m.id.clone()),
            };
            out.insert(p.clone(), f);
        }
    }
    out
}

/// The base text of a document, or an empty one if a mod needs it and it does not exist.
fn doc_source<'a>(base: &'a Snapshot, path: &str, needed: bool) -> Option<(&'a str, Encoding)> {
    match base.docs.get(path) {
        Some(d) => Some((&d.text, d.encoding)),
        None => needed.then_some(("", Encoding::Utf8)),
    }
}

fn render(path: &str, enc: Encoding, text: &str, changes: Vec<DocChange>) -> Result<RenderedDoc> {
    let bytes = DocText::encode(enc, text)
        .map_err(|c| Error::Input(format!("{path}: {c:?} cannot be written in a Latin-1 file")))?;
    Ok(RenderedDoc {
        hash: FileHash {
            b3: hash::b3_bytes(&bytes),
            size: bytes.len() as u64,
        },
        bytes,
        changes,
    })
}

/// Adds the values of `key` not already present (compared by [`ini_key`]) before the base
/// entries (`front`) or after them. Returns the final values in file order.
fn ordered<'a>(
    ini: &mut SiderIni,
    key: &str,
    new: impl Iterator<Item = (&'a String, &'a String)>,
    front: bool,
    changes: &mut Vec<DocChange>,
) -> Vec<String> {
    let base: Vec<String> = ini.entries(key).into_iter().map(|e| e.value).collect();
    let mut seen: BTreeSet<String> = base.iter().map(|v| ini_key(v)).collect();
    let mut added: Vec<&str> = Vec::new();
    for (by, v) in new {
        if seen.insert(ini_key(v)) {
            added.push(v);
            changes.push(DocChange {
                added: true,
                line: format!("{key} = \"{v}\""),
                by: by.clone(),
            });
        }
    }
    if added.is_empty() {
        return base;
    }
    ini.insert(key, &added, front);
    ini.entries(key).into_iter().map(|e| e.value).collect()
}

/// One stadium per team: the last mod's row replaces every other row of that team, base rows
/// included (Sider would pick among several at random, F5). A base row identical to the
/// winner stays in place.
fn teams(mt: &mut MapTeams, mods: &[&Package], conflicts: &mut Conflicts) -> Vec<DocChange> {
    let mut winners: BTreeMap<String, (usize, &str, Vec<String>)> = BTreeMap::new();
    for (i, m) in mods.iter().enumerate() {
        for line in &m.manifest.map_teams {
            let fields = map_teams::parse_row(line).expect("validated on load");
            let key = map_teams::normalize_key(&fields[0]);
            let identity = map_teams::row_identity(&fields).to_vec();
            if let Some((prev, _, f)) = winners.insert(key.clone(), (i, line, fields))
                && map_teams::row_identity(&f) != identity
            {
                conflicts
                    .entry((i, prev))
                    .or_default()
                    .push(format!("map_teams team {key}"));
            }
        }
    }
    let rows = mt.rows();
    let mut changes = Vec::new();
    let mut remove = Vec::new();
    let mut append = Vec::new();
    for (key, (i, line, fields)) in &winners {
        let by = &mods[*i].id;
        let same = |r: &map_teams::Row| {
            map_teams::row_identity(&r.fields) == map_teams::row_identity(fields)
        };
        let team: Vec<_> = rows.iter().filter(|r| &r.key == key).collect();
        let keep = team.iter().find(|r| same(r)).map(|r| r.line);
        for r in team.iter().filter(|r| Some(r.line) != keep) {
            remove.push(r.line);
            changes.push(DocChange {
                added: false,
                line: r.fields.join(", "),
                by: by.clone(),
            });
        }
        if keep.is_none() {
            append.push(*line);
            changes.push(DocChange {
                added: true,
                line: fields.join(", "),
                by: by.clone(),
            });
        }
    }
    mt.splice(&remove, &append);
    changes
}

/// A virtual path served by the cpk roots of two mods: the earliest root wins (F1). Files in
/// one virtual path are not the same tree path, so `files` cannot see these.
fn shadows(
    t: &Target,
    roots: &[String],
    files: &BTreeMap<String, Provided>,
    mods: &[&Package],
    conflicts: &mut Conflicts,
) {
    let dirs: Vec<String> = roots
        .iter()
        .filter_map(|r| t.cpk_root_dir(r).ok())
        .collect();
    let mut layers: BTreeMap<String, Vec<(usize, usize)>> = BTreeMap::new();
    for (p, f) in files {
        let Some(m) =
            f.by.as_ref()
                .and_then(|b| mods.iter().position(|m| &m.id == b))
        else {
            continue;
        };
        for (ri, dir) in dirs.iter().enumerate() {
            if p.len() > dir.len() && paths::is_within_ci(p, dir) {
                let q = p[dir.len() + 1..].to_ascii_lowercase();
                layers.entry(q).or_default().push((ri, m));
            }
        }
    }
    for (q, mut layer) in layers {
        layer.sort();
        let winner = layer[0].1;
        let losers: BTreeSet<usize> = layer.iter().map(|l| l.1).filter(|&m| m != winner).collect();
        for l in losers {
            conflicts
                .entry((winner, l))
                .or_default()
                .push(format!("<virtual> {q}"));
        }
    }
}
