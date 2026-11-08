//! Every `catalog/<id>/mod.toml` loads through the real manifest code. The catalog ships no
//! payload (mods are not redistributed), so each one gets a synthetic payload with one file per
//! put source, then `Package::load` runs every check (schema, roots, user paths, sider.ini values,
//! map_teams rows).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use fl_core::base::Snapshot;
use fl_core::build::build;
use fl_core::manifest::{FILE_NAME, Manifest, Package};
use fl_core::target::Target;

#[test]
fn every_catalog_manifest_loads() {
    let catalog = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog");
    let tmp = Path::new(env!("CARGO_TARGET_TMPDIR")).join("catalog");
    let _ = fs::remove_dir_all(&tmp);
    let target = Target::fl27();
    let mut pkgs = BTreeMap::new();
    for entry in fs::read_dir(&catalog).unwrap() {
        let src = entry.unwrap().path();
        if !src.join(FILE_NAME).is_file() {
            continue;
        }
        let id = src.file_name().unwrap().to_string_lossy().into_owned();
        assert!(src.join("SOURCE.md").is_file(), "{id}: SOURCE.md missing");
        let text = fs::read_to_string(src.join(FILE_NAME)).unwrap();
        let manifest: Manifest = toml::from_str(&text).unwrap_or_else(|e| panic!("{id}: {e}"));
        let dir = tmp.join(&id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(FILE_NAME), &text).unwrap();
        for put in &manifest.put {
            let file = if put.from.ends_with('/') {
                dir.join(&put.from).join("payload.bin")
            } else {
                dir.join(&put.from)
            };
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, id.as_bytes()).unwrap();
        }
        let pkg = Package::load(&dir, &target).unwrap_or_else(|e| panic!("{id}: {e}"));
        assert!(!pkg.puts.is_empty(), "{id}: no puts");
        pkgs.insert(id, pkg);
    }
    assert_eq!(pkgs.len(), 21, "catalog size: {:?}", pkgs.keys());
    // All twenty-one enabled at once still build on the synthetic FL27 tree (conflicts are reported).
    let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/fl27-game");
    let base = Snapshot::scan(&game, &target).unwrap();
    let list: Vec<String> = pkgs.keys().cloned().collect();
    build(&target, &base, &pkgs, &list).unwrap();
}
