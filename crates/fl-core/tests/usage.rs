//! Disk usage and bulk remove on a copy of the fixture game: hardlinked vs copied payloads,
//! reclaimable bytes, staging leftovers, and `remove_disabled` keeping enabled or applied mods.

mod common;

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use common::*;
use fl_core::import::{Options, import};
use fl_core::workspace::{Generation, Status, Workspace};

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

/// Sets the mtime of `p` and everything under it (not following symlinks) to an hour ago, so
/// the staging quiet period has passed.
fn age(p: &Path) {
    let old = SystemTime::now() - Duration::from_secs(3600);
    let m = fs::symlink_metadata(p).unwrap();
    if m.is_dir() {
        for e in fs::read_dir(p).unwrap() {
            age(&e.unwrap().path());
        }
    }
    if !m.file_type().is_symlink() {
        fs::File::open(p).unwrap().set_modified(old).unwrap();
    }
}

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn interrupt(ws: &Workspace, n: u32) {
    let g = Generation {
        status: Status::InProgress,
        ..Default::default()
    };
    let path = ws.dir.join(format!("history/{n:04}.toml"));
    fs::write(path, toml::to_string(&g).unwrap()).unwrap();
}

/// faces, commentary and cams imported and applied.
fn three(name: &str) -> (PathBuf, Workspace) {
    let (game, mut ws) = lab(name);
    let src = fresh(&format!("{name}-src"));
    let (faces, commentary) = sources(&src);
    write_files(&src.join("cams"), &[("Cams/modules/Cam.lua", "return {}")]);
    for s in [&faces, &commentary, &src.join("cams")] {
        import(&mut ws, s, Options::default()).unwrap();
    }
    apply(&mut ws);
    (game, ws)
}

#[test]
fn remove_disabled_deletes_only_the_listed_removable_mods() {
    let (_, mut ws) = three("usage-remove");
    // faces: disabled and applied away; cams: disabled but still applied; commentary: enabled.
    ws.state.disabled.insert("faces".into());
    apply(&mut ws);
    ws.state.disabled.insert("cams".into());
    ws.save().unwrap();
    write_files(&ws.dir, &[("staging/x", "x")]);
    age(&ws.dir.join("staging"));

    assert_eq!(ws.removable().unwrap(), ["faces"]);
    assert_eq!(ws.usage().unwrap().reclaimable_ids(), ["faces"]);
    let faces = mod_bytes(&ws, "faces");
    let f = ws
        .remove_disabled(&ids(&["faces", "cams", "commentary"]), true)
        .unwrap();
    assert_eq!(f.removed, ["faces", "staging/"]);
    assert_eq!(f.bytes, faces + 1);
    assert_eq!(f.kept.len(), 2, "{:?}", f.kept);
    assert!(f.kept.iter().all(|k| k.contains("no longer removable")));
    assert_eq!(ws.state.mods, ["commentary", "cams"]);
    assert!(!ws.dir.join("mods/faces").exists() && !ws.dir.join("staging").exists());
    assert!(ws.dir.join("mods/cams").is_dir() && ws.dir.join("mods/commentary").is_dir());
    let reopened = Workspace::open(&ws.dir).unwrap();
    assert_eq!(reopened.state.mods, ["commentary", "cams"]);
}

#[test]
fn removing_a_once_linked_mod_leaves_the_kept_mods_files_untouched() {
    let (game, mut ws) = lab("usage-linked");
    let src = fresh("usage-linked-src");
    let (_, commentary) = sources(&src);
    write_files(
        &src.join("reshade"),
        &[("dxgi.dll", "dll"), ("reshade-shaders/a.fx", "fx")],
    );
    import(&mut ws, &src.join("reshade"), Options::default()).unwrap();
    import(&mut ws, &commentary, Options::default()).unwrap();
    apply(&mut ws);
    // Both payloads were hardlinked into the game (.dll and .cpk).
    let u = ws.usage().unwrap();
    assert!(u.mods.iter().all(|m| m.shared > 0), "{u:?}");

    ws.state.disabled.insert("commentary".into());
    apply(&mut ws);
    let before = tree(&game);
    let f = ws.remove_disabled(&ids(&["commentary"]), false).unwrap();
    assert_eq!(f.removed, ["commentary"]);
    assert_eq!(tree(&game), before, "the game is byte for byte the same");
    assert_eq!(fs::read(game.join("dxgi.dll")).unwrap(), b"dll");
    let u = ws.usage().unwrap();
    assert_eq!((u.mods[0].id.as_str(), u.mods[0].shared), ("reshade", 3));
}

#[test]
fn nothing_is_removed_while_an_apply_is_interrupted() {
    let (_, mut ws) = three("usage-pending");
    ws.state.disabled.insert("faces".into());
    apply(&mut ws);
    interrupt(&ws, 3);
    let u = ws.usage().unwrap();
    assert_eq!(u.reclaimable_ids(), ["faces"], "usage still works");
    let err = ws.remove_disabled(&ids(&["faces"]), true).unwrap_err();
    assert!(err.to_string().contains("interrupted"), "{err}");
    let err = ws.remove("faces").unwrap_err();
    assert!(err.to_string().contains("interrupted"), "{err}");
    assert!(ws.dir.join("mods/faces").is_dir());
    assert!(ws.state.mods.contains(&"faces".to_string()));
}

#[test]
fn symlinks_lose_only_the_link() {
    let (_, mut ws) = three("usage-links");
    ws.state.disabled.insert("faces".into());
    apply(&mut ws);
    // mods/faces and staging/ become links to folders outside the workspace.
    let outside = fresh("usage-links-outside");
    let mods_faces = ws.dir.join("mods/faces");
    copy_dir(&mods_faces, &outside.join("faces"));
    fs::remove_dir_all(&mods_faces).unwrap();
    symlink(outside.join("faces"), &mods_faces).unwrap();
    write_files(&outside, &[("staging/archive/big.bin", "0123456789")]);
    symlink(outside.join("staging"), ws.dir.join("staging")).unwrap();

    let u = ws.usage().unwrap();
    let faces = u.mods.iter().find(|m| m.id == "faces").unwrap();
    assert_eq!(
        (faces.files, faces.bytes),
        (0, 0),
        "a symlinked mod folder is not followed"
    );
    assert!(
        u.notes
            .iter()
            .any(|n| n.contains("mods/faces is not a folder")),
        "{u:?}"
    );
    assert_eq!(u.staging, 0, "a symlinked staging is not followed");

    let f = ws.remove_disabled(&ids(&["faces"]), true).unwrap();
    assert_eq!(
        (f.removed.clone(), f.kept.len()),
        (ids(&["faces", "staging/"]), 0)
    );
    assert!(
        fs::symlink_metadata(&mods_faces).is_err()
            && fs::symlink_metadata(ws.dir.join("staging")).is_err()
    );
    assert!(
        outside.join("faces/mod.toml").is_file(),
        "the link target stays"
    );
    assert!(outside.join("staging/archive/big.bin").is_file());
}

#[test]
fn a_recent_staging_is_kept() {
    let (_, mut ws) = lab("usage-recent");
    write_files(&ws.dir, &[("staging/archive/new.bin", "n")]);
    let f = ws.remove_disabled(&[], true).unwrap();
    assert!(
        f.removed.is_empty() && f.kept[0].contains("last 10 minutes"),
        "{f:?}"
    );
    assert!(ws.dir.join("staging/archive/new.bin").is_file());
    age(&ws.dir.join("staging"));
    let f = ws.remove_disabled(&[], true).unwrap();
    assert_eq!((f.removed.clone(), f.bytes), (ids(&["staging/"]), 1));
}

#[test]
fn orphan_folders_are_listed_and_removed_only_when_asked() {
    let (_, mut ws) = lab("usage-orphan");
    write_files(&ws.dir, &[("mods/stray/a.bin", "abc")]);
    // Just written (an import may be running): kept, with a note.
    let u = ws.usage().unwrap();
    assert!(u.orphans.is_empty() && u.reclaimable() == 0, "{u:?}");
    assert!(
        u.notes
            .iter()
            .any(|n| n.contains("mods/stray: orphan folder changed in the last 10 minutes"))
    );
    let f = ws.remove_disabled(&ids(&["stray"]), false).unwrap();
    assert!(
        f.removed.is_empty() && f.kept[0].contains("no longer removable"),
        "{f:?}"
    );
    assert!(ws.dir.join("mods/stray/a.bin").is_file());

    age(&ws.dir.join("mods/stray"));
    let u = ws.usage().unwrap();
    assert_eq!(u.orphans, [("stray".to_string(), 3)]);
    assert_eq!((u.reclaimable(), u.reclaimable_ids()), (3, ids(&["stray"])));
    assert!(ws.remove_disabled(&[], false).unwrap().removed.is_empty());
    assert!(
        ws.dir.join("mods/stray").is_dir(),
        "not listed, not removed"
    );
    let f = ws.remove_disabled(&ids(&["stray"]), false).unwrap();
    assert_eq!((f.removed.clone(), f.bytes), (ids(&["stray"]), 3));
    assert!(!ws.dir.join("mods/stray").exists());
}

#[test]
fn an_empty_workspace_has_nothing_to_free() {
    let (_, mut ws) = lab("usage-empty");
    let u = ws.usage().unwrap();
    assert!(u.mods.is_empty() && u.orphans.is_empty() && u.notes.is_empty());
    assert_eq!((u.reclaimable(), u.history), (0, 0));
    assert!(u.same_fs && u.total > 0);
    assert!(ws.removable().unwrap().is_empty());
    assert_eq!(ws.remove_disabled(&[], false).unwrap(), Default::default());
}

#[test]
fn missing_folders_and_broken_mods_are_notes_not_errors() {
    let (game, mut ws) = three("usage-missing");
    ws.state.disabled.insert("faces".into());
    apply(&mut ws);
    fs::remove_dir_all(ws.dir.join("mods/faces")).unwrap();
    fs::write(ws.dir.join("mods/commentary/mod.toml"), "not = [toml").unwrap();
    let u = ws.usage().unwrap();
    let faces = u.mods.iter().find(|m| m.id == "faces").unwrap();
    assert_eq!(faces.bytes, 0);
    assert!(
        u.notes.iter().any(|n| n.contains("mods/faces is missing")),
        "{:?}",
        u.notes
    );
    let c = u.mods.iter().find(|m| m.id == "commentary").unwrap();
    assert_eq!(c.shared, 0);
    assert!(
        u.notes
            .iter()
            .any(|n| n.starts_with("commentary: shared bytes unknown"))
    );

    // A missing folder still leaves the list cleanly.
    let f = ws.remove_disabled(&ids(&["faces"]), false).unwrap();
    assert_eq!(f.removed, ["faces"]);
    assert!(!ws.state.mods.contains(&"faces".to_string()));

    // No game folder: usage reports it and assumes one filesystem.
    fs::rename(&game, game.with_extension("moved")).unwrap();
    let u = ws.usage().unwrap();
    assert!(
        u.same_fs && u.notes.iter().any(|n| n.contains("not found")),
        "{:?}",
        u.notes
    );
}

#[test]
fn mod_ids_are_one_plain_path_component() {
    let (_, mut ws) = lab("usage-ids");
    for bad in ["", ".", "..", "../x", "a/b", "a/", "/etc"] {
        assert!(ws.remove(bad).is_err(), "{bad:?}");
        let f = ws.remove_disabled(&ids(&[bad]), false).unwrap();
        assert!(
            f.removed.is_empty() && f.kept[0].contains("bad mod id"),
            "{bad:?}: {f:?}"
        );
    }
    ws.state.mods.push("../escape".into());
    assert!(ws.usage().unwrap_err().to_string().contains("bad mod id"));
    assert!(ws.removable().is_err());
}
