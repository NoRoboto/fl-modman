//! Disk usage and bulk remove on a copy of the fixture game: hardlinked vs copied payloads,
//! reclaimable bytes, staging leftovers, and `remove_disabled` keeping enabled or applied mods.

mod common;

use std::fs;

use common::*;
use fl_core::import::{Options, import};
use fl_core::workspace::Workspace;

fn apply(ws: &mut Workspace) {
    let list = ws.enabled();
    ws.apply(&list, "apply", true, true).unwrap();
}

fn mod_bytes(ws: &Workspace, id: &str) -> u64 {
    let dir = ws.dir.join("mods").join(id);
    let files = tree(&dir);
    files.values().map(|b| b.len() as u64).sum()
}

#[test]
fn usage_splits_shared_and_workspace_only_bytes() {
    let (_, mut ws) = lab("usage-split");
    let (faces, commentary) = sources(&fresh("usage-split-src"));
    import(&mut ws, &faces, Options::default()).unwrap();
    import(&mut ws, &commentary, Options::default()).unwrap();
    apply(&mut ws);

    let u = ws.usage().unwrap();
    assert!(u.same_fs);
    let [f, c] = &u.mods[..] else { panic!("{u:?}") };
    // `download/*.cpk` is hardlinked into the game ("new cpk", 7 bytes); face.bin is copied.
    assert_eq!(
        (c.id.as_str(), c.enabled, c.applied, c.shared),
        ("commentary", true, true, 7)
    );
    assert_eq!((f.id.as_str(), f.shared), ("faces", 0));
    for m in [f, c] {
        assert_eq!(
            m.bytes,
            mod_bytes(&ws, &m.id),
            "{} counts mod.toml and index.json",
            m.id
        );
        assert_eq!(m.files, 3, "{}: payload, mod.toml, index.json", m.id);
    }
    assert_eq!(c.only_here(), c.bytes - 7);
    assert_eq!(u.mods_dir, f.bytes + c.bytes);
    assert!(u.originals > 0, "the replaced base cpk is kept");
    assert!(u.history > 0);
    assert_eq!((u.staging, u.reclaimable()), (0, 0));
    assert!(
        u.total > u.mods_dir + u.originals + u.history,
        "plus state.toml and the base"
    );
}

#[test]
fn reclaimable_waits_for_the_apply_and_counts_staging() {
    let (_, mut ws) = lab("usage-reclaim");
    let (faces, commentary) = sources(&fresh("usage-reclaim-src"));
    import(&mut ws, &faces, Options::default()).unwrap();
    import(&mut ws, &commentary, Options::default()).unwrap();
    apply(&mut ws);

    ws.state.disabled.insert("faces".into());
    let u = ws.usage().unwrap();
    assert!(!u.mods[0].enabled && u.mods[0].applied);
    assert_eq!(u.reclaimable(), 0, "still in the game until the next apply");

    apply(&mut ws);
    write_files(&ws.dir, &[("staging/archive/left.bin", "0123456789")]);
    let u = ws.usage().unwrap();
    assert!(u.mods[0].removable());
    assert_eq!(u.staging, 10);
    assert_eq!(u.reclaimable(), mod_bytes(&ws, "faces") + 10);
}

#[test]
fn copied_payloads_share_nothing() {
    let (game, mut ws) = lab("usage-copied");
    let (_, commentary) = sources(&fresh("usage-copied-src"));
    import(&mut ws, &commentary, Options::default()).unwrap();
    apply(&mut ws);
    // What apply does across filesystems: the placed file is a copy, not the payload's inode.
    let placed = game.join("download/01_smkdb_fa1.cpk");
    let bytes = fs::read(&placed).unwrap();
    fs::remove_file(&placed).unwrap();
    fs::write(&placed, bytes).unwrap();
    let u = ws.usage().unwrap();
    assert_eq!(u.mods[0].shared, 0);
    assert_eq!(u.mods[0].only_here(), u.mods[0].bytes);
}

#[test]
fn remove_disabled_keeps_enabled_and_applied_mods() {
    let (_, mut ws) = lab("usage-remove");
    let src = fresh("usage-remove-src");
    let (faces, commentary) = sources(&src);
    write_files(&src.join("cams"), &[("Cams/modules/Cam.lua", "return {}")]);
    for s in [&faces, &commentary, &src.join("cams")] {
        import(&mut ws, s, Options::default()).unwrap();
    }
    apply(&mut ws);
    // faces: disabled and applied away; cams: disabled but still applied; commentary: enabled.
    ws.state.disabled.insert("faces".into());
    apply(&mut ws);
    ws.state.disabled.insert("cams".into());
    ws.save().unwrap();
    write_files(&ws.dir, &[("staging/x", "x")]);

    assert_eq!(ws.removable().unwrap(), ["faces"]);
    assert_eq!(ws.remove_disabled().unwrap(), ["faces"]);
    assert_eq!(ws.state.mods, ["commentary", "cams"]);
    assert!(!ws.dir.join("mods/faces").exists() && !ws.dir.join("staging").exists());
    assert!(ws.dir.join("mods/cams").is_dir() && ws.dir.join("mods/commentary").is_dir());
    let reopened = Workspace::open(&ws.dir).unwrap();
    assert_eq!(reopened.state.mods, ["commentary", "cams"]);
    assert!(
        ws.remove_disabled().unwrap().is_empty(),
        "nothing left to remove"
    );
}
