//! Property-style tests for T5 (rebuild is exactly reversible and path independent) and T7
//! (without conflicts, list order does not change what the game observes). A small xorshift
//! generator keeps them deterministic and dependency-free.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use common::*;
use fl_core::base::{Snapshot, TreeManifest};
use fl_core::build::{Desired, build};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            v.swap(i, self.below(i + 1));
        }
    }
}

/// The four fixture mods plus synthetic ones: a lua module, a stadium row, a file conflict
/// pair (replacing a base file) and a cpk root that shadows Closs.
fn pool() -> &'static Pkgs {
    static POOL: OnceLock<Pkgs> = OnceLock::new();
    POOL.get_or_init(|| {
        let g = "pool";
        let mut p = fixture_packages();
        let gameplay = |id: &str, body: &str| {
            let toml = "[[put]]\nfrom = \"g\"\nto = \"@data/dt41_eng_all.cpk\"\n";
            write_pkg(g, id, toml, &[("g", body)])
        };
        p.extend(pkgs(vec![
            write_pkg(
                g,
                "turf",
                "lua_module = [\"TurfLoader.lua\"]\n[[put]]\nfrom = \"TurfLoader.lua\"\nto = \"@modules/\"\n",
                &[("TurfLoader.lua", "-- turf\n")],
            ),
            write_pkg(g, "rows-x", "map_teams = [\"500, 001, X, X\"]\n", &[]),
            gameplay("gameplay-a", "A"),
            gameplay("gameplay-b", "B"),
            write_pkg(
                g,
                "menu",
                "cpk_root = ['.\\livecpk\\Menu']\n[[put]]\nfrom = \"r/\"\nto = \"@livecpk/Menu/\"\n",
                &[("r/mex/string/all.str", "menu strings")],
            ),
        ]));
        p
    })
}

const CONFLICT_FREE: [&str; 6] = [
    "mariano-closs-v5",
    "machete-cursor",
    "camp-nou-2026",
    "camera-epl",
    "turf",
    "rows-x",
];

fn desired(b: &Snapshot, mods: &[String]) -> Desired {
    build(&target(), b, pool(), mods).unwrap()
}

/// T5: after any sequence of enable / disable / reorder actions, applying each plan leaves
/// the tree equal to Build of the current list; disabling everything restores the base.
#[test]
fn t5_random_action_sequences_are_exact_and_reversible() {
    let b = base();
    let p = pool();
    let ids: Vec<String> = p.keys().cloned().collect();
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for _case in 0..300 {
        let mut enabled: Vec<String> = Vec::new();
        let mut tree: TreeManifest = b.files.clone();
        for _step in 0..rng.below(10) + 1 {
            match rng.below(3) {
                0 => {
                    let m = &ids[rng.below(ids.len())];
                    if !enabled.contains(m) {
                        let at = rng.below(enabled.len() + 1);
                        enabled.insert(at, m.clone());
                    }
                }
                1 if !enabled.is_empty() => {
                    enabled.remove(rng.below(enabled.len()));
                }
                _ if enabled.len() > 1 => {
                    let m = enabled.remove(rng.below(enabled.len()));
                    let at = rng.below(enabled.len() + 1);
                    enabled.insert(at, m);
                }
                _ => {}
            }
            tree = apply(&tree, &plan_for(&b, p, &enabled, &tree));
            assert_eq!(tree, desired(&b, &enabled).manifest(), "after {enabled:?}");
            assert!(
                plan_for(&b, p, &enabled, &tree).ops.is_empty(),
                "plan is idempotent"
            );
        }
        let back = plan_for(&b, p, &[], &tree);
        assert_eq!(
            apply(&tree, &back),
            b.files,
            "disable all restores the base"
        );
    }
}

#[test]
fn t5_enable_all_then_disable_all_plans_back_to_base() {
    let b = base();
    let p = pool();
    let on = apply(&b.files, &plan_for(&b, p, &list(&CONFLICT_FREE), &b.files));
    let off = apply(&on, &plan_for(&b, p, &[], &on));
    assert_eq!(off, b.files);
    assert!(plan_for(&b, p, &[], &off).ops.is_empty());
}

/// What the game observes (IR §4.2): files, document entries as sets, map_teams bytes, and
/// the VFS view through the cpk roots (where root order matters).
#[derive(Debug, PartialEq, Eq)]
struct Obs {
    files: BTreeMap<String, String>,
    entries: BTreeSet<String>,
    map_teams: Option<String>,
    vfs: BTreeMap<String, String>,
}

fn observe(d: &Desired) -> Obs {
    let t = target();
    let files = d
        .files
        .iter()
        .map(|(p, f)| (p.clone(), f.hash.b3.clone()))
        .collect();
    let lua = ini_values(d, "lua.module");
    let roots = ini_values(d, "cpk.root");
    let mut vfs = BTreeMap::new();
    for root in &roots {
        let dir = t.cpk_root_dir(root).unwrap();
        for (p, f) in &d.files {
            if p.len() > dir.len() && fl_core::paths::is_within_ci(p, &dir) {
                let q = p[dir.len() + 1..].to_ascii_lowercase();
                vfs.entry(q).or_insert_with(|| f.hash.b3.clone());
            }
        }
    }
    Obs {
        files,
        entries: lua.into_iter().chain(roots).collect(),
        map_teams: d.docs.get(MAP_TEAMS).map(|r| r.hash.b3.clone()),
        vfs,
    }
}

/// T7: for conflict-free sets, every list order gives the same observation.
#[test]
fn t7_order_is_irrelevant_without_conflicts() {
    let b = base();
    let mut rng = Rng(0xdead_beef_cafe_f00d);
    for mask in 1u32..(1 << CONFLICT_FREE.len()) {
        let set: Vec<String> = CONFLICT_FREE
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, m)| m.to_string())
            .collect();
        let reference = desired(&b, &set);
        assert!(
            reference.conflicts.is_empty(),
            "{set:?}: {:?}",
            reference.conflicts
        );
        let obs = observe(&reference);
        let mut orders = vec![set.iter().rev().cloned().collect::<Vec<_>>()];
        for _ in 0..3 {
            let mut s = set.clone();
            rng.shuffle(&mut s);
            orders.push(s);
        }
        for order in orders {
            assert_eq!(observe(&desired(&b, &order)), obs, "{order:?}");
        }
    }
}

/// With a conflict, the list decides, and swapping the pair swaps the winner (T6).
#[test]
fn t6_conflicting_pair_follows_the_list() {
    let b = base();
    let ab = desired(&b, &list(&["gameplay-a", "gameplay-b"]));
    let ba = desired(&b, &list(&["gameplay-b", "gameplay-a"]));
    let by = |d: &Desired| d.files["Data/dt41_eng_all.cpk"].by.clone();
    assert_eq!(by(&ab).as_deref(), Some("gameplay-b"));
    assert_eq!(by(&ba).as_deref(), Some("gameplay-a"));
    let shadow = desired(&b, &list(&["mariano-closs-v5", "menu"]));
    assert_eq!(shadow.conflicts.len(), 1);
    assert_eq!(shadow.conflicts[0].winner, "menu");
}
