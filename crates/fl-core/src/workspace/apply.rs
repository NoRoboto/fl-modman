//! Apply and its relatives: plan a mod list against the disk with the drift check (Q9), realize
//! it as a generation (Q12), and recover, restore, roll back or accept the drift.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use super::{Generation, Status, Workspace, scan};
use crate::adapters::fs::{place, write_atomic, write_toml};
use crate::base::{Snapshot, TreeManifest};
use crate::build::{Desired, build};
use crate::error::{Error, Result, input, io};
use crate::manifest::Package;
use crate::plan::{Op, Plan, plan};

/// A mod list planned against the disk, and the paths that drifted from the expected tree.
struct Check {
    base: Snapshot,
    pkgs: BTreeMap<String, Package>,
    desired: Desired,
    cur: TreeManifest,
    plan: Plan,
    drift: Vec<String>,
}

impl Workspace {
    /// Plans `list` against the disk; drift is every expected path, and every path the plan
    /// touches, whose content differs from the last complete generation. Only the paths fl-modman
    /// knows (base, last generation, every mod's puts, the documents) are read: no folder walk.
    fn check(&self, list: &[String]) -> Result<Check> {
        crate::lap("start");
        let base = self.base()?;
        crate::lap("base");
        let pkgs = self.packages()?;
        crate::lap("packages");
        let desired = build(&self.t, &base, &pkgs, list)?;
        crate::lap("build");
        let mut expected = base.files.clone();
        expected.extend(self.last()?.unwrap_or_default().files);
        crate::lap("history");
        let puts = pkgs.values().flat_map(|p| p.puts.keys());
        let docs = [&self.t.sider_ini, &self.t.map_teams];
        let known = expected.keys().chain(puts).chain(docs);
        let known = known.map(String::as_str).collect();
        let (game, mut skipped) = (&self.state.game, Vec::new());
        let cur = scan::cached(&self.dir, game, &self.t, Some(&known), &mut skipped)?;
        crate::lap("scan");
        let mut plan = plan(&self.t, &base, &pkgs, &desired, &cur);
        crate::lap("plan");
        let (want, t) = (desired.manifest(), &self.t);
        // Runtime-rewritten files keep their edits while we want what we wanted last time.
        plan.ops.retain(|op| {
            let p = op.path();
            !(t.drift_ignored(p) && cur.contains_key(p) && expected.get(p) == want.get(p))
        });
        let touched = plan.ops.iter().map(Op::path);
        let paths: BTreeSet<&str> = expected.keys().map(String::as_str).chain(touched).collect();
        let b3 = |m: &TreeManifest, p: &str| m.get(p).map(|h| h.b3.clone());
        let drift = paths
            .into_iter()
            .filter(|p| !t.drift_ignored(p) && b3(&cur, p) != b3(&expected, p));
        let drift = drift.map(String::from).collect();
        crate::lap("drift");
        Ok(Check {
            base,
            pkgs,
            desired,
            cur,
            plan,
            drift,
        })
    }

    /// Plans `list`; with `write`, records it as an in-progress generation, applies it file by
    /// file and marks it complete. Refuses on drift (if `drift_check`) or while the game runs.
    pub fn apply(
        &mut self,
        list: &[String],
        note: &str,
        write: bool,
        drift_check: bool,
    ) -> Result<(Plan, Option<u32>)> {
        self.refuse_pending()?;
        let c = self.check(list)?;
        if drift_check && !c.drift.is_empty() {
            return Err(Error::Drift(c.drift));
        }
        if !write {
            return Ok((c.plan, None));
        }
        self.refuse_running()?;
        let (n, base, mods) = (
            self.next()?,
            self.state.base.clone(),
            c.desired.mods.clone(),
        );
        let files = c.desired.manifest().into_iter();
        let files = files
            .filter(|(p, h)| c.base.files.get(p) != Some(h))
            .collect();
        let g = Generation {
            status: Status::InProgress,
            note: note.into(),
            base,
            mods,
            files,
        };
        self.realize(&c, Some((n, g)))?;
        Ok((c.plan, Some(n)))
    }

    /// Executes a plan, recorded as generation `g` (in progress, then complete) once every
    /// source is known to exist. A base file is kept in `originals/` (once) before it is replaced.
    fn realize(&self, c: &Check, mut g: Option<(u32, Generation)>) -> Result<()> {
        let (game, orig) = (&self.state.game, self.dir.join("originals"));
        let source = |op: &Op| match op {
            Op::Put {
                path, by: Some(m), ..
            } => Some(c.pkgs[m].dir.join(&c.pkgs[m].puts[path].source)),
            Op::Put { path, .. } => Some(orig.join(path)),
            _ => None,
        };
        // A cheap guard on the indexes: every source exists with the size the plan expects.
        let bad = |op: &Op| match (op, source(op)) {
            (Op::Put { size, .. }, Some(s)) => {
                // symlink_metadata: a source that is a link is not followed, it fails the check.
                let ok = fs::symlink_metadata(&s).is_ok_and(|m| m.is_file() && m.len() == *size);
                (!ok).then_some(s)
            }
            _ => None,
        };
        if let Some(s) = c.plan.ops.iter().find_map(bad) {
            let s = s.display();
            return Err(input(format!(
                "{s} is missing or changed (no copy kept): run `fl-modman check` (mods) or `accept-drift`"
            )));
        }
        if let Some((n, g)) = &g {
            write_toml(&self.gen_path(*n), g)?;
        }
        for op in &c.plan.ops {
            let (path, link) = (op.path(), self.t.hardlink(op.path()));
            let (dst, kept) = (game.join(path), orig.join(path));
            if c.base.files.contains_key(path) && dst.is_file() && !kept.exists() {
                place(&dst, &kept, link)?;
            }
            match op {
                Op::Del { .. } => {
                    fs::remove_file(&dst).map_err(io(&dst))?;
                    let mut dir = dst.parent();
                    while let Some(d) = dir.filter(|d| d != game && fs::remove_dir(d).is_ok()) {
                        dir = d.parent();
                    }
                }
                Op::Put { .. } => place(&source(op).expect("put"), &dst, link)?,
                Op::Doc { .. } => write_atomic(&dst, &c.desired.docs[path].bytes)?,
            }
        }
        if let Some((n, g)) = &mut g {
            g.status = Status::Complete;
            write_toml(&self.gen_path(*n), g)?;
        }
        Ok(())
    }

    /// Finishes an interrupted generation, or rolls back to the last complete one.
    pub fn recover(&mut self, finish: bool) -> Result<u32> {
        self.refuse_running()?;
        let mut pending = self
            .history()?
            .into_iter()
            .filter(|g| g.1.status == Status::InProgress);
        let (n, mut g) = pending
            .next_back()
            .ok_or_else(|| input("nothing to recover"))?;
        let list = if finish {
            g.mods.clone()
        } else {
            self.last()?.unwrap_or_default().mods
        };
        self.realize(&self.check(&list)?, None)?;
        g.status = if finish {
            Status::Complete
        } else {
            Status::RolledBack
        };
        write_toml(&self.gen_path(n), &g)?;
        Ok(n)
    }

    /// Re-applies the last complete generation over the drift.
    pub fn restore(&mut self) -> Result<(Plan, Option<u32>)> {
        let list = self.last()?.unwrap_or_default().mods;
        self.apply(&list, "restore", true, false)
    }

    /// Re-applies generation `n` through apply and makes its mods the list (others disabled).
    pub fn rollback(&mut self, n: u32, write: bool) -> Result<(Plan, Option<u32>)> {
        let mut hist = self.history()?.into_iter();
        let g = hist.find(|g| g.0 == n && g.1.status == Status::Complete);
        let g = g
            .ok_or_else(|| input(format!("no complete generation {n}")))?
            .1;
        if g.base != self.state.base {
            return Err(input(format!("generation {n} predates an accepted drift")));
        }
        let out = self.apply(&g.mods, &format!("rollback to {n}"), write, true)?;
        if write {
            let rest = self.state.mods.iter().filter(|m| !g.mods.contains(m));
            self.state.disabled = rest.cloned().collect();
            self.state.mods = g.mods.iter().chain(&self.state.disabled).cloned().collect();
            self.save()?;
        }
        Ok(out)
    }

    /// Takes the drifted files as they are on disk into a new base and records the new expected
    /// tree; the next apply re-applies the mods. Returns the drifted paths.
    pub fn accept_drift(&mut self) -> Result<Vec<String>> {
        self.refuse_pending()?;
        let mut c = self.check(&self.enabled())?;
        for p in &c.drift {
            match c.cur.get(p) {
                Some(h) => c.base.files.insert(p.clone(), h.clone()),
                None => c.base.files.remove(p),
            };
            let _ = fs::remove_file(self.dir.join("originals").join(p));
        }
        let changed = |p: &str| c.drift.iter().any(|d| d == p);
        c.base.refresh(&self.state.game, &self.t, changed)?;
        self.save_base(&c.base)?;
        let mut g = self.last()?.unwrap_or_default();
        g.files.retain(|p, _| !changed(p));
        (g.note, g.base) = ("accept-drift".into(), self.state.base.clone());
        write_toml(&self.gen_path(self.next()?), &g)?;
        Ok(c.drift)
    }
}
