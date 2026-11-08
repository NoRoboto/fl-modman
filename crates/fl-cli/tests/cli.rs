//! End-to-end runs of the `fl-modman` binary on a copy of the fixture game.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fresh(name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for e in fs::read_dir(src).unwrap() {
        let p = e.unwrap().path();
        let to = dst.join(p.file_name().unwrap());
        if p.is_dir() {
            copy_dir(&p, &to)
        } else {
            fs::copy(&p, &to).map(drop).unwrap()
        }
    }
}

/// Runs the binary with `HOME` set to `home`; returns (success, stdout + stderr).
fn fl(home: &Path, args: &[&str]) -> (bool, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_fl-modman"))
        .env("HOME", home)
        .env_remove("FL_MODMAN")
        .args(args)
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&o.stdout).into_owned() + &String::from_utf8_lossy(&o.stderr);
    (o.status.success(), out)
}

#[test]
fn init_import_apply_history_settings() {
    let root = fresh("cli-flow");
    let game = root.join("Games/FL27");
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/fl27-game"),
        &game,
    );
    let docs = root.join("pfx/drive_c/users/steamuser/Documents/SPGAME/Football Life");
    fs::create_dir_all(&docs).unwrap();
    let mut dat = vec![0u8; 612];
    dat[..4].copy_from_slice(b"WECF");
    dat[8..12].copy_from_slice(&0x87eu32.to_le_bytes());
    fs::write(docs.join("settings.dat"), &dat).unwrap();
    let src = root.join("src/Machete Cursors");
    fs::create_dir_all(src.join("Machete/common")).unwrap();
    fs::write(src.join("Machete/common/c.dds"), "c").unwrap();
    let (g, s, w) = (
        game.to_str().unwrap(),
        src.to_str().unwrap(),
        root.join("Games/fl-modman"),
    );
    let ws = w.to_str().unwrap();

    let (ok, out) = fl(
        &root,
        &["init", g, "--prefix", root.join("pfx").to_str().unwrap()],
    );
    assert!(ok && out.contains("base "), "{out}");
    let (ok, out) = fl(&root, &["list"]);
    assert!(!ok && out.contains("--workspace"), "{out}");
    let (ok, out) = fl(&root, &["-w", ws, "import", s]);
    assert!(
        ok && out.contains("imported machete-cursors (Livecpk, root Machete)"),
        "{out}"
    );
    let (ok, out) = fl(&root, &["-w", ws, "check"]);
    assert!(
        ok && out.contains("1 mod(s), 1 file(s) hashed, 0 mismatch"),
        "{out}"
    );
    let (ok, out) = fl(&root, &["-w", ws, "apply"]);
    assert!(
        ok && out.contains("re-run with --yes")
            && !game.join("SiderAddons/livecpk/Machete").exists(),
        "{out}"
    );
    let (ok, out) = fl(&root, &["-w", ws, "apply", "--yes"]);
    assert!(ok && out.contains("applied as generation 1"), "{out}");
    assert!(
        game.join("SiderAddons/livecpk/Machete/common/c.dds")
            .is_file()
    );
    let (ok, out) = fl(&root, &["-w", ws, "disable", "machete-cursors"]);
    assert!(ok, "{out}");
    let (_, out) = fl(&root, &["-w", ws, "list"]);
    assert_eq!(out, "  0  [ ] machete-cursors\n");
    let (ok, out) = fl(&root, &["-w", ws, "remove", "machete-cursors"]);
    assert!(!ok && out.contains("apply first"), "{out}");
    assert!(fl(&root, &["-w", ws, "apply", "--yes"]).0);
    let (_, out) = fl(&root, &["-w", ws, "history"]);
    assert!(
        out.contains("0001  Complete  apply  [machete-cursors]")
            && out.contains("0002  Complete  apply  []"),
        "{out}"
    );

    let (ok, out) = fl(
        &root,
        &["-w", ws, "settings", "set", "fullscreen=on", "vsync=off"],
    );
    assert!(ok && out.contains("backup"), "{out}");
    let (_, out) = fl(&root, &["-w", ws, "settings", "get"]);
    assert!(
        out.contains("fullscreen=on\n") && out.contains("vsync=off\n"),
        "{out}"
    );
    assert!(out.contains("rendering options menu once"), "{out}");
    let (ok, out) = fl(&root, &["-w", ws, "settings", "set", "grass=low"]);
    assert!(!ok && out.contains("rendering options menu once"), "{out}");
    fs::create_dir_all(docs.join("2027/save")).unwrap();
    fs::write(docs.join("2027/save/GRAPHICS000000"), [50u8; 19]).unwrap();
    let pairs = ["preset=low", "shadows=high", "hdr=on"];
    let (ok, out) = fl(
        &root,
        &[&["-w", ws, "settings", "set"][..], &pairs].concat(),
    );
    assert!(ok && out.contains("GRAPHICS000000.bak-"), "{out}");
    let (_, out) = fl(&root, &["-w", ws, "settings", "get"]);
    assert!(out.contains("hdr=on\npreset=custom\n"), "{out}");
    assert!(
        out.contains("render_scale=80\n") && out.contains("shadows=high\n"),
        "{out}"
    );

    let (ok, out) = fl(&root, &["doctor"]);
    assert!(ok && out.contains(r#"WINEDLLOVERRIDES="lsteamclient=d;steamclient64=n;steam_api64=n,b;winmm=n,b" %command%"#), "{out}");
    let (ok, out) = fl(&root, &["detect"]);
    assert!(ok && out.contains(&format!("game   {g}")), "{out}");
}

#[test]
fn import_with_a_catalog_manifest_and_a_misplaced_team() {
    let root = fresh("cli-manifest");
    let game = root.join("Games/FL27");
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/fl27-game"),
        &game,
    );
    let src = root.join("src/Boot Root");
    fs::create_dir_all(src.join("boot-root/common")).unwrap();
    fs::write(src.join("boot-root/common/b.dds"), "b").unwrap();
    let manifest = root.join("mod.toml");
    let toml = "name = \"Boot root\"\ncpk_root = ['.\\livecpk\\boot-root']\n\n[[put]]\n\
                from = \"boot-root/\"\nto = \"@livecpk/boot-root/\"\n";
    fs::write(&manifest, toml).unwrap();
    let (g, s, m) = (
        game.to_str().unwrap(),
        src.to_str().unwrap(),
        manifest.to_str(),
    );
    let ws = root.join("Games/fl-modman");
    let ws = ws.to_str().unwrap();
    assert!(fl(&root, &["init", g]).0);

    let (ok, out) = fl(&root, &["-w", ws, "import", s, "--team", "108"]);
    assert!(!ok && out.contains("--team is for stadiums"), "{out}");
    let (ok, out) = fl(&root, &["-w", ws, "import", s, "--manifest", m.unwrap()]);
    assert!(
        ok && out.contains("imported boot-root (Livecpk, root boot-root)"),
        "{out}"
    );
    let dir = root.join("Games/fl-modman/mods/boot-root");
    assert_eq!(fs::read_to_string(dir.join("mod.toml")).unwrap(), toml);
}
