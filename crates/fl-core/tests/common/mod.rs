//! Shared helpers for the integration tests: the synthetic FL27 fixture tree, the fixture
//! packages, synthetic packages written under the cargo test tmpdir, and an in-memory
//! "apply" of a plan onto a hash manifest.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use fl_core::base::{FileHash, Snapshot, TreeManifest};
use fl_core::build::{Desired, build};
use fl_core::codec::DocText;
use fl_core::manifest::Package;
use fl_core::plan::{Op, Plan, plan};
use fl_core::target::Target;

pub type Pkgs = BTreeMap<String, Package>;

pub fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

pub fn target() -> Target {
    Target::fl27()
}

pub fn base() -> Snapshot {
    Snapshot::scan(&fixtures().join("fl27-game"), &target()).unwrap()
}

/// The base with the synthetic FL26-shaped `map_teams.txt` (which maps team 108 to Camp Nou).
pub fn base_with_fl26_map_teams() -> Snapshot {
    let mut b = base();
    let bytes = fs::read(fixtures().join("fl26/map_teams.txt")).unwrap();
    let hash = FileHash {
        b3: fl_core::hash::b3_bytes(&bytes),
        size: bytes.len() as u64,
    };
    b.files.insert(MAP_TEAMS.into(), hash);
    b.docs.insert(MAP_TEAMS.into(), DocText::decode(&bytes));
    b
}

pub fn fixture_packages() -> Pkgs {
    Package::load_all(&fixtures().join("mods"), &target()).unwrap()
}

/// Writes a package `<tmp>/<group>/<id>/` with `mod.toml` and payload files.
pub fn write_pkg(group: &str, id: &str, toml: &str, files: &[(&str, &str)]) -> Package {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(group).join(id);
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("mod.toml"), toml).unwrap();
    for (rel, content) in files {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }
    Package::load(&dir, &target()).unwrap()
}

pub fn pkgs(list: Vec<Package>) -> Pkgs {
    list.into_iter().map(|p| (p.id.clone(), p)).collect()
}

pub fn list(mods: &[&str]) -> Vec<String> {
    mods.iter().map(|s| s.to_string()).collect()
}

pub fn built(base: &Snapshot, pkgs: &Pkgs, mods: &[&str]) -> Desired {
    build(&target(), base, pkgs, &list(mods)).unwrap()
}

pub fn plan_for(base: &Snapshot, pkgs: &Pkgs, mods: &[String], current: &TreeManifest) -> Plan {
    let t = target();
    let d = build(&t, base, pkgs, mods).unwrap();
    plan(&t, base, pkgs, &d, current)
}

/// What apply would do to the tree's hash manifest.
pub fn apply(current: &TreeManifest, p: &Plan) -> TreeManifest {
    let mut out = current.clone();
    for op in &p.ops {
        match op {
            Op::Del { path, .. } => {
                out.remove(path);
            }
            Op::Put { path, b3, size, .. } | Op::Doc { path, b3, size, .. } => {
                let h = FileHash {
                    b3: b3.clone(),
                    size: *size,
                };
                out.insert(path.clone(), h);
            }
        }
    }
    out
}

pub fn doc_text(d: &Desired, path: &str) -> String {
    String::from_utf8(d.docs[path].bytes.clone()).unwrap()
}

/// The values of a `sider.ini` key in the rendered document, in file order.
pub fn ini_values(d: &Desired, key: &str) -> Vec<String> {
    let ini = fl_core::codec::sider_ini::SiderIni::parse(&doc_text(d, SIDER_INI));
    ini.entries(key).into_iter().map(|e| e.value).collect()
}

pub fn base_text(path: &str) -> String {
    fs::read_to_string(fixtures().join("fl27-game").join(path)).unwrap()
}

pub const SIDER_INI: &str = "SiderAddons/sider.ini";
pub const MAP_TEAMS: &str = "SiderAddons/content/stadiums/map_teams.txt";

/// A fresh, empty folder under the cargo test tmpdir.
pub fn fresh(name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

pub fn write_files(root: &Path, files: &[(&str, &str)]) {
    for (rel, content) in files {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }
}

pub fn copy_dir(src: &Path, dst: &Path) {
    for (rel, bytes) in tree(src) {
        let p = dst.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, bytes).unwrap();
    }
}

/// Every file under `root` with its bytes.
pub fn tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(root, &p, out);
            } else {
                let rel = p.strip_prefix(root).unwrap().to_str().unwrap().to_string();
                out.insert(rel, fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

/// A copy of the fixture game in `<tmp>/<name>/game` with a workspace in `<tmp>/<name>/fl-modman`.
pub fn lab(name: &str) -> (PathBuf, fl_core::workspace::Workspace) {
    let root = fresh(name);
    let game = root.join("game");
    copy_dir(&fixtures().join("fl27-game"), &game);
    let ws = fl_core::workspace::Workspace::init(&root.join("fl-modman"), &game, None).unwrap();
    (game, ws)
}

/// Mod sources used by several tests: a livecpk pack in a wrapper folder with a readme, and a
/// `download/*.cpk` replacement whose name differs in case from the base file.
pub fn sources(root: &Path) -> (PathBuf, PathBuf) {
    write_files(
        &root.join("faces"),
        &[
            ("Faces Pack/common/character0/face.bin", "face"),
            ("Faces Pack/readme.txt", "read me"),
        ],
    );
    write_files(&root.join("Commentary"), &[("01_SMKDB_FA1.CPK", "new cpk")]);
    (root.join("faces"), root.join("Commentary"))
}
