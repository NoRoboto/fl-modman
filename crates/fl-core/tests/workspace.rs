//! Apply, history, undo, crash recovery and drift on a copy of the fixture game.

mod common;

use std::fs;
use std::os::unix::fs::MetadataExt;

use common::*;
use fl_core::import::{Options, import};
use fl_core::workspace::{Generation, Status, Workspace};

const FACE: &str = "SiderAddons/livecpk/Faces Pack/common/character0/face.bin";
const CPK: &str = "download/01_smkdb_fa1.cpk";

fn apply(ws: &mut Workspace) -> fl_core::Result<Option<u32>> {
    let list = ws.enabled();
    ws.apply(&list, "apply", true, true).map(|r| r.1)
}

fn setup(name: &str) -> (std::path::PathBuf, Workspace) {
    let (game, mut ws) = lab(name);
    let (faces, commentary) = sources(&fresh(&format!("{name}-src")));
    import(&mut ws, &faces, Options::default()).unwrap();
    import(&mut ws, &commentary, Options::default()).unwrap();
    (game, ws)
}

#[test]
fn apply_disable_all_and_rollback_round_trip() {
    let (game, mut ws) = setup("ws-round-trip");
    let day0 = tree(&game);
    let (plan, n) = ws.apply(&ws.enabled(), "apply", false, true).unwrap();
    assert_eq!(
        (plan.ops.len(), n),
        (3, None),
        "dry run: two puts and sider.ini"
    );
    assert_eq!(tree(&game), day0);

    assert_eq!(apply(&mut ws).unwrap(), Some(1));
    let gen1 = tree(&game);
    assert_eq!(gen1[CPK], b"new cpk");
    assert_eq!(gen1[FACE], b"face");
    assert!(
        String::from_utf8_lossy(&gen1[SIDER_INI]).contains(r#"cpk.root = ".\livecpk\Faces Pack""#)
    );
    let kept = ws.dir.join("originals").join(CPK);
    assert_eq!(
        fs::read(kept).unwrap(),
        day0[CPK],
        "the base cpk is kept for undo"
    );

    ws.state.disabled = ws.state.mods.iter().cloned().collect();
    assert_eq!(apply(&mut ws).unwrap(), Some(2));
    assert_eq!(
        tree(&game),
        day0,
        "disable all returns to day one, byte for byte"
    );
    assert!(
        !game.join("SiderAddons/livecpk/Faces Pack").exists(),
        "empty folders are pruned"
    );

    ws.rollback(1, true).unwrap();
    assert_eq!(tree(&game), gen1);
    assert!(ws.state.disabled.is_empty());
    let hist = ws.history().unwrap();
    assert_eq!(hist.len(), 3);
    assert!(hist.iter().all(|g| g.1.status == Status::Complete));
    assert_eq!(apply(&mut ws).unwrap(), Some(4), "nothing drifted");
}

#[test]
fn heavy_assets_are_hardlinked_and_text_is_copied() {
    let (game, mut ws) = setup("ws-links");
    let lua = fresh("ws-links-lua");
    write_files(
        &lua,
        &[
            ("Cam/modules/Cam.lua", "return {}"),
            ("Cam/modules/Cam.ini", "a=1"),
        ],
    );
    import(&mut ws, &lua, Options::default()).unwrap();
    apply(&mut ws).unwrap();
    let ino = |p: &std::path::Path| fs::metadata(p).unwrap().ino();
    let mods = ws.dir.join("mods");
    assert_eq!(
        ino(&game.join(CPK)),
        ino(&mods.join("commentary").join(CPK))
    );
    assert_ne!(
        ino(&game.join(FACE)),
        ino(&mods.join("faces/Faces Pack/common/character0/face.bin"))
    );
    assert_ne!(
        ino(&game.join("SiderAddons/modules/Cam.lua")),
        ino(&mods.join("ws-links-lua/modules/Cam.lua"))
    );
    assert!(
        !fs::symlink_metadata(game.join(FACE))
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[test]
fn the_running_game_is_its_process_name_not_a_mention() {
    use std::os::unix::process::CommandExt;
    use std::process::Command;
    let exe = "fl-test-game.exe";
    let mut game = Command::new("sleep")
        .arg0(format!(r"Z:\games\{exe}"))
        .arg("5")
        .spawn()
        .unwrap();
    let mut tool = Command::new("sh")
        .args(["-c", "sleep 5; true", "fl-test-tool.exe"])
        .spawn()
        .unwrap();
    assert!(fl_core::HOST.game_running(exe));
    assert!(!fl_core::HOST.game_running("fl-test-tool.exe"));
    for p in [&mut game, &mut tool] {
        p.kill().unwrap();
        p.wait().unwrap();
    }
}

fn interrupt(ws: &Workspace, n: u32, mods: &[&str]) {
    let mods = mods.iter().map(|m| m.to_string()).collect();
    let g = Generation {
        status: Status::InProgress,
        mods,
        ..Default::default()
    };
    fs::write(
        ws.dir.join(format!("history/{n:04}.toml")),
        toml::to_string(&g).unwrap(),
    )
    .unwrap();
}

#[test]
fn an_interrupted_apply_is_finished_or_rolled_back() {
    let (game, mut ws) = setup("ws-crash");
    ws.state.disabled.insert("commentary".into());
    apply(&mut ws).unwrap();
    let gen1 = tree(&game);

    // Crash halfway through generation 2: the base cpk kept, the new one placed, nothing marked
    // complete.
    interrupt(&ws, 2, &["faces", "commentary"]);
    let kept = ws.dir.join("originals").join(CPK);
    fs::create_dir_all(kept.parent().unwrap()).unwrap();
    fs::copy(game.join(CPK), kept).unwrap();
    fs::write(game.join(CPK), "new cpk").unwrap();
    let err = apply(&mut ws).unwrap_err().to_string();
    assert!(err.contains("generation 2 was interrupted"), "{err}");
    assert_eq!(ws.recover(false).unwrap(), 2);
    assert_eq!(tree(&game), gen1);
    assert_eq!(ws.history().unwrap()[1].1.status, Status::RolledBack);

    interrupt(&ws, 3, &["faces", "commentary"]);
    assert_eq!(ws.recover(true).unwrap(), 3);
    assert_eq!(fs::read(game.join(CPK)).unwrap(), b"new cpk");
    ws.state.disabled.clear();
    let (plan, _) = ws.apply(&ws.enabled(), "apply", false, false).unwrap();
    assert!(
        plan.ops.is_empty(),
        "the disk is the finished generation: {:?}",
        plan.ops
    );
}

#[test]
fn plans_read_known_paths_only_and_keep_an_unchanged_cache() {
    let (game, mut ws) = setup("ws-known");
    fs::write(game.join("stray.bin"), "x").unwrap();
    let cache = ws.dir.join("hashes.json");
    let mtime = || fs::metadata(&cache).unwrap().modified().unwrap();
    ws.apply(&ws.enabled(), "apply", false, true).unwrap();
    let before = mtime();
    assert!(!fs::read_to_string(&cache).unwrap().contains("stray.bin"));
    ws.apply(&ws.enabled(), "apply", false, true).unwrap();
    assert_eq!(mtime(), before, "an unchanged cache is not rewritten");
}

#[test]
fn drift_is_refused_then_restored_or_accepted() {
    let (game, mut ws) = setup("ws-drift");
    apply(&mut ws).unwrap();

    // A file the tool placed is edited by hand: refuse, then restore.
    fs::write(game.join(FACE), "hacked").unwrap();
    let err = apply(&mut ws).unwrap_err().to_string();
    assert!(err.contains(FACE) && err.contains("accept-drift"), "{err}");
    ws.restore().unwrap();
    assert_eq!(fs::read(game.join(FACE)).unwrap(), b"face");

    // A game update rewrites a base file and the exe; an exe-guarded mod is in the list.
    let exe = ws.base().unwrap().exe_sha256.unwrap();
    write_files(
        &ws.dir.join("mods/guarded"),
        &[("mod.toml", &format!("exe = \"{exe}\"\n"))],
    );
    ws.state.mods.push("guarded".into());
    fs::write(game.join("Data/dt41_eng_all.cpk"), "patched").unwrap();
    fs::write(game.join("FL_2027.exe"), "new exe").unwrap();
    let err = apply(&mut ws).unwrap_err().to_string();
    assert!(
        err.contains("2 file(s) changed") && err.contains("FL_2027.exe"),
        "{err}"
    );
    let err = ws.restore().unwrap_err().to_string();
    assert!(err.contains("no copy kept"), "{err}");

    let accepted = ws.accept_drift().unwrap();
    assert_eq!(accepted, ["Data/dt41_eng_all.cpk", "FL_2027.exe"]);
    assert_ne!(ws.base().unwrap().exe_sha256.unwrap(), exe);
    let (plan, _) = ws.apply(&ws.enabled(), "apply", true, true).unwrap();
    assert!(
        plan.warnings
            .iter()
            .any(|w| w.starts_with("guarded disabled")),
        "{:?}",
        plan.warnings
    );
    assert_eq!(
        fs::read(game.join("Data/dt41_eng_all.cpk")).unwrap(),
        b"patched"
    );

    // Files Sider modules rewrite at runtime are not drift and keep their edits.
    fs::write(game.join("SiderAddons/modules/camera.ini"), "runtime").unwrap();
    apply(&mut ws).unwrap();
    assert_eq!(
        fs::read(game.join("SiderAddons/modules/camera.ini")).unwrap(),
        b"runtime"
    );
}

const PAYLOAD: &str = "mods/faces/Faces Pack/common/character0/face.bin";

#[test]
fn plan_reads_the_index_and_check_rehashes() {
    let (game, mut ws) = setup("ws-index");
    let index = ws.dir.join("mods/faces/index.json");
    let text = fs::read_to_string(&index).unwrap();
    assert!(
        text.contains("Faces Pack/common/character0/face.bin"),
        "{text}"
    );
    assert!(!text.contains("mod.toml"));
    assert_eq!(ws.check_mods().unwrap(), (2, vec![]));

    // Same size, other content: the plan never reads the payload, `check` sees it.
    let before = ws.apply(&ws.enabled(), "apply", false, true).unwrap().0;
    fs::write(ws.dir.join(PAYLOAD), "FACE").unwrap();
    let after = ws.apply(&ws.enabled(), "apply", false, true).unwrap().0;
    assert_eq!(before, after);
    let (_, bad) = ws.check_mods().unwrap();
    let face = "faces: Faces Pack/common/character0/face.bin: content changed";
    assert_eq!(bad, [face]);

    // Apply guards what it places by size only.
    fs::write(ws.dir.join(PAYLOAD), "longer").unwrap();
    let err = apply(&mut ws).unwrap_err().to_string();
    assert!(err.contains("fl-modman check"), "{err}");
    assert!(!game.join(FACE).exists());
}

#[test]
fn a_workspace_without_indexes_gets_them_once() {
    let (_, ws) = setup("ws-migrate");
    let index = ws.dir.join("mods/faces/index.json");
    let want = fs::read(&index).unwrap();
    fs::remove_file(&index).unwrap();
    fs::remove_file(ws.dir.join("mods/commentary/index.json")).unwrap();
    let (_, bad) = ws.check_mods().unwrap();
    assert_eq!(bad, ["faces: no index.json", "commentary: no index.json"]);

    let pkgs = ws.packages().unwrap();
    assert_eq!(pkgs["faces"].puts[FACE].hash.size, 4);
    assert_eq!(fs::read(&index).unwrap(), want);
    assert!(ws.dir.join("mods/commentary/index.json").is_file());
    assert_eq!(ws.check_mods().unwrap().1, Vec::<String>::new());
}
