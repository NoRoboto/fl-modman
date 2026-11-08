//! Plan: the ordered operations that turn a current tree into the desired one, with the mod
//! responsible for each, printable as text or serializable as JSON.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::base::{Snapshot, TreeManifest};
use crate::build::{Conflict, Desired, DocChange};
use crate::hash;
use crate::manifest::Package;
use crate::target::Target;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Replaces {
    /// The path does not exist yet.
    Nothing,
    /// The current file is the base file (kept as an original for undo).
    Base,
    /// The current file is neither base nor desired (another mod's, or a hand edit).
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Del {
        path: String,
        size: u64,
    },
    Put {
        path: String,
        b3: String,
        size: u64,
        by: Option<String>,
        replaces: Replaces,
    },
    Doc {
        path: String,
        b3: String,
        size: u64,
        changes: Vec<DocChange>,
    },
}

impl Op {
    pub fn path(&self) -> &str {
        match self {
            Op::Del { path, .. } | Op::Put { path, .. } | Op::Doc { path, .. } => path,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub target: String,
    pub base_snapshot: String,
    pub exe_sha256: Option<String>,
    pub version: Option<String>,
    pub mods: Vec<String>,
    /// Deletions (deepest first), then puts, then document edits: content is in place
    /// before the documents that activate it.
    pub ops: Vec<Op>,
    pub conflicts: Vec<Conflict>,
    pub warnings: Vec<String>,
}

/// Diffs `desired` against `current` (a scan of the tree as it is now). Only base paths and
/// paths some known package (enabled or not) puts may be deleted; anything else is left as is.
pub fn plan(
    target: &Target,
    base: &Snapshot,
    packages: &BTreeMap<String, Package>,
    desired: &Desired,
    current: &TreeManifest,
) -> Plan {
    let known: BTreeSet<&String> = base
        .files
        .keys()
        .chain(packages.values().flat_map(|p| p.puts.keys()))
        .chain([&target.sider_ini, &target.map_teams])
        .collect();
    let want = desired.manifest();
    let (mut dels, mut puts, mut docs) = (vec![], vec![], vec![]);
    for (p, h) in current {
        if !want.contains_key(p) && !target.is_user_path(p) && known.contains(p) {
            dels.push(Op::Del {
                path: p.clone(),
                size: h.size,
            });
        }
    }
    dels.reverse();

    for (p, w) in &want {
        let cur = current.get(p);
        if cur.is_some_and(|c| c.b3 == w.b3) {
            continue;
        }
        let (path, b3, size) = (p.clone(), w.b3.clone(), w.size);
        if let Some(d) = desired.docs.get(p) {
            let changes = d.changes.clone();
            docs.push(Op::Doc {
                path,
                b3,
                size,
                changes,
            });
            continue;
        }
        let replaces = match cur {
            None => Replaces::Nothing,
            Some(c) if base.files.get(p).is_some_and(|b| b.b3 == c.b3) => Replaces::Base,
            Some(_) => Replaces::Other,
        };
        let by = desired.files[p].by.clone();
        puts.push(Op::Put {
            path,
            b3,
            size,
            by,
            replaces,
        });
    }

    Plan {
        target: desired.target.clone(),
        base_snapshot: desired.base_snapshot.clone(),
        exe_sha256: base.exe_sha256.clone(),
        version: base.version.clone(),
        mods: desired.mods.clone(),
        ops: dels.into_iter().chain(puts).chain(docs).collect(),
        conflicts: desired.conflicts.clone(),
        warnings: desired.warnings.clone(),
    }
}

pub fn human_size(n: u64) -> String {
    const UNITS: [&str; 4] = ["KiB", "MiB", "GiB", "TiB"];
    if n < 1024 {
        return format!("{n} B");
    }
    let mut v = n as f64 / 1024.0;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    format!("{v:.1} {}", UNITS[u])
}

impl Plan {
    /// One line per deletion, per document change and per mod's group of puts.
    pub fn render_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(
            s,
            "target {}  base snap {}  exe sha256 {}  version {}",
            self.target,
            self.base_snapshot,
            self.exe_sha256.as_deref().map_or("missing", hash::short),
            self.version.as_deref().unwrap_or("unknown"),
        );
        let _ = writeln!(s, "mods, the lower wins: {}", self.mods.join(", "));
        let mut groups: BTreeMap<(&str, Replaces), (usize, u64, &str)> = BTreeMap::new();
        let (mut n_put, mut n_del, mut n_doc, mut bytes) = (0, 0, 0, 0);
        for op in &self.ops {
            match op {
                Op::Del { path, size } => {
                    n_del += 1;
                    let _ = writeln!(s, "  - del   {path}   {}", human_size(*size));
                }
                Op::Put {
                    path,
                    size,
                    by,
                    replaces,
                    ..
                } => {
                    (n_put, bytes) = (n_put + 1, bytes + size);
                    let by = by.as_deref().unwrap_or("base");
                    let g = groups.entry((by, *replaces)).or_insert((0, 0, path));
                    (g.0, g.1) = (g.0 + 1, g.1 + size);
                }
                Op::Doc { path, changes, .. } => {
                    n_doc += 1;
                    if changes.is_empty() {
                        let _ = writeln!(s, "  ~ doc   {path}   restored to base");
                    }
                    for c in changes {
                        let sign = if c.added { "+" } else { "-" };
                        let _ = writeln!(s, "  ~ doc   {path}   {sign} {}   {}", c.line, c.by);
                    }
                }
            }
        }
        for ((by, replaces), (n, size, first)) in groups {
            let (sign, note) = match replaces {
                Replaces::Nothing => ("+", ""),
                Replaces::Base => ("~", "   replaces base"),
                Replaces::Other => ("~", "   replaces current"),
            };
            let what = match n {
                1 => first.to_string(),
                _ => format!("{n} files (e.g. {first})"),
            };
            let _ = writeln!(
                s,
                "  {sign} put   {what}   {}{note}   {by}",
                human_size(size)
            );
        }
        for c in &self.conflicts {
            let _ = writeln!(s, "  ! {c} (e.g. {})", c.items[0]);
        }
        for w in &self.warnings {
            let _ = writeln!(s, "  ? {w}");
        }
        let (size, n_conf) = (human_size(bytes), self.conflicts.len());
        let _ = writeln!(
            s,
            "Plan: {n_put} put ({size}), {n_del} del, {n_doc} docs, {n_conf} conflicts."
        );
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(human_size(12), "12 B");
        assert_eq!(human_size(1536), "1.5 KiB");
        assert_eq!(human_size(3 * 1024 * 1024 * 1024), "3.0 GiB");
    }
}
