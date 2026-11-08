//! Import: root detection for each of the five types, wrapper folders, ambiguity, archives.

mod common;

use std::fs;
use std::io::Write;
use std::path::Path;

use common::*;
use fl_core::import::{Imported, Options, import};
use fl_core::workspace::Workspace;

fn team_opt(team: Option<&str>) -> Options<'_> {
    Options {
        team,
        ..Default::default()
    }
}

fn with(manifest: &Path) -> Options<'_> {
    Options {
        manifest: Some(manifest),
        ..Default::default()
    }
}

fn import_dir(ws: &mut Workspace, src: &Path, team: Option<&str>) -> Imported {
    import(ws, src, team_opt(team)).unwrap_or_else(|e| panic!("{}: {e}", src.display()))
}

#[test]
fn each_type_is_detected_through_wrapper_folders() {
    let (_, mut ws) = lab("imp-types");
    let src = fresh("imp-types-src");
    let (faces, commentary) = sources(&src);
    write_files(
        &src.join("cams"),
        &[
            ("Cams v2/modules/Cam.lua", "x"),
            ("Cams v2/modules/Cam.ini", "y"),
        ],
    );
    write_files(
        &src.join("nou"),
        &[
            ("Camp Nou 2026/stadium.fpk", "s"),
            ("Camp Nou 2026/shot.jpg", ""),
        ],
    );
    write_files(
        &src.join("reshade"),
        &[
            ("ReShade/dxgi.dll", "d"),
            ("ReShade/reshade-shaders/a.fx", "f"),
        ],
    );

    let m = import_dir(&mut ws, &faces, None);
    assert_eq!(
        (m.kind.describe(), m.root.as_str()),
        ("Livecpk", "Faces Pack")
    );
    assert_eq!(
        m.files,
        ["SiderAddons/livecpk/Faces Pack/common/character0/face.bin"],
        "readme skipped"
    );
    let m = import_dir(&mut ws, &commentary, None);
    assert_eq!(
        (m.kind.describe(), m.files.as_slice()),
        (
            "Download",
            ["download/01_smkdb_fa1.cpk".to_string()].as_slice()
        )
    );
    let m = import_dir(&mut ws, &src.join("cams"), None);
    assert_eq!(
        (m.kind.describe(), m.root.as_str()),
        ("Lua", "Cams v2/modules")
    );
    let m = import_dir(&mut ws, &src.join("nou"), Some("108,002"));
    assert_eq!(m.kind.describe(), "Stadium");
    assert_eq!(
        m.files,
        ["SiderAddons/content/stadiums/Camp Nou 2026/stadium.fpk"]
    );
    let m = import_dir(&mut ws, &src.join("reshade"), None);
    assert_eq!(m.kind.describe(), "Loose");
    assert!(
        m.files.contains(&"reshade-shaders/a.fx".to_string())
            && m.files.contains(&"dxgi.dll".to_string())
    );

    let pkgs = ws.packages().unwrap();
    assert_eq!(pkgs["faces"].manifest.cpk_root, [r".\livecpk\Faces Pack"]);
    assert_eq!(pkgs["cams"].manifest.lua_module, ["Cam.lua"]);
    assert_eq!(
        pkgs["nou"].manifest.map_teams,
        ["108, 002, Camp Nou 2026, Camp Nou 2026"]
    );
    assert_eq!(
        ws.enabled(),
        ["faces", "commentary", "cams", "nou", "reshade"]
    );
}

#[test]
fn stadium_slot_comes_from_the_base_row_or_must_be_given() {
    let (_, mut ws) = lab("imp-stadium");
    let src = fresh("imp-stadium-src");
    write_files(&src.join("anfield"), &[("Anfield 2026/stadium.fpk", "s")]);
    let err = import(&mut ws, &src.join("anfield"), Options::default())
        .err()
        .unwrap();
    assert!(err.to_string().contains("--team"), "{err}");
    let err = import(&mut ws, &src.join("anfield"), team_opt(Some("999")))
        .err()
        .unwrap();
    assert!(err.to_string().contains("--team 999,<slot>"), "{err}");
    import_dir(&mut ws, &src.join("anfield"), Some("103"));
    let rows = &ws.packages().unwrap()["anfield"].manifest.map_teams;
    assert_eq!(rows, &["103, 004, Anfield 2026, Anfield 2026"]);
}

#[test]
fn an_ambiguous_archive_lists_candidates_and_takes_a_root() {
    let (_, mut ws) = lab("imp-ambiguous");
    let src = fresh("imp-ambiguous-src").join("pack");
    write_files(
        &src,
        &[
            ("Pack/A/common/x.dds", "a"),
            ("Pack/B/common/y.dds", "b"),
            ("info.txt", ""),
        ],
    );
    let err = import(&mut ws, &src, Options::default())
        .err()
        .unwrap()
        .to_string();
    assert!(
        err.contains("Pack/A (Livecpk), Pack/B (Livecpk)") && err.contains("--root"),
        "{err}"
    );
    assert!(!ws.dir.join("mods/pack").exists() && ws.state.mods.is_empty());
    let m = import(
        &mut ws,
        &src,
        Options {
            root: Some("Pack/B"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(m.files, ["SiderAddons/livecpk/B/common/y.dds"]);
    let err = import(&mut ws, &src, Options::default()).err().unwrap();
    assert!(err.to_string().contains("already exists"), "{err}");
}

#[test]
fn zip_and_7z_archives_are_extracted() {
    let (_, mut ws) = lab("imp-archives");
    let src = fresh("imp-archives-src");
    let zip_path = src.join("Machete Cursors v1.zip");
    let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path).unwrap());
    for (name, body) in [
        ("Wrapper/Machete/common/render/c.dds", "c"),
        ("Wrapper/readme.txt", "r"),
    ] {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(body.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
    let m = import(&mut ws, &zip_path, Options::default()).unwrap();
    assert_eq!(
        (m.id.as_str(), m.root.as_str()),
        ("machete-cursors-v1", "Wrapper/Machete")
    );
    assert_eq!(m.files, ["SiderAddons/livecpk/Machete/common/render/c.dds"]);
    assert!(!ws.dir.join("staging").exists());

    write_files(&src.join("7z-in"), &[("Cam/modules/Cam.lua", "x")]);
    let seven = src.join("cam.7z");
    sevenz_rust2::compress_to_path(src.join("7z-in"), &seven).unwrap();
    let m = import(&mut ws, &seven, Options::default()).unwrap();
    assert_eq!(
        (m.kind.describe(), m.files.as_slice()),
        (
            "Lua",
            ["SiderAddons/modules/Cam.lua".to_string()].as_slice()
        )
    );

    let err = import(&mut ws, &src.join("broken.zip"), Options::default())
        .err()
        .unwrap();
    assert!(err.to_string().contains("broken.zip"), "{err}");
}

#[test]
fn a_stadium_server_folder_is_a_stadium_and_a_team_is_never_ignored() {
    let (_, mut ws) = lab("imp-stadium-server");
    let src = fresh("imp-stadium-server-src");
    let nou = src.join("spotify-nou");
    write_files(
        &nou,
        &[
            ("Camp Nou/AddOn/Turf/t.ftex", "t"),
            ("Camp Nou/Asset/model/bg/s.fmdl", "s"),
            ("Camp Nou/common/render/c.dds", "c"),
        ],
    );
    let err = import(&mut ws, &nou, Options::default()).err().unwrap();
    assert!(err.to_string().contains("--team"), "{err}");
    let m = import_dir(&mut ws, &nou, Some("108,002"));
    assert_eq!(
        (m.kind.describe(), m.root.as_str()),
        ("Stadium", "Camp Nou")
    );
    assert!(
        m.files
            .contains(&"SiderAddons/content/stadiums/Camp Nou/common/render/c.dds".into())
    );
    let rows = &ws.packages().unwrap()["spotify-nou"].manifest.map_teams;
    assert_eq!(rows, &["108, 002, Camp Nou, Camp Nou"]);

    // A team on anything that is not a stadium fails, detected or forced.
    write_files(&src.join("faces"), &[("Faces/common/f.dds", "f")]);
    let err = import(&mut ws, &src.join("faces"), team_opt(Some("108")));
    let err = err.err().unwrap().to_string();
    assert!(
        err.contains("--team is for stadiums") && err.contains("Livecpk"),
        "{err}"
    );
    let lua: fl_core::kinds::Kind = "LUA".parse().unwrap();
    let opt = Options {
        kind: Some(lua),
        team: Some("108"),
        ..Default::default()
    };
    let err = import(&mut ws, &src.join("faces"), opt)
        .err()
        .unwrap()
        .to_string();
    assert!(err.contains("this mod is Lua"), "{err}");
    assert!(!ws.dir.join("mods/faces").exists());
    assert_eq!(ws.state.mods, ["spotify-nou"]);
}

#[test]
fn a_given_manifest_replaces_the_generated_one_and_is_checked() {
    let (_, mut ws) = lab("imp-manifest");
    let src = fresh("imp-manifest-src");
    let closs = src.join("Closs V5");
    write_files(
        &closs,
        &[
            ("Closs V5/ags/sound/acb/t.acb", "a"),
            ("Closs V5/mex/string/all.str", "m"),
        ],
    );
    let toml = "name = \"Closs (English slot)\"\ncpk_root = ['.\\livecpk\\Closs V5']\n\n\
                [[put]]\nfrom = \"Closs V5/ags/sound/\"\nto = \"@livecpk/Closs V5/eng/sound/\"\n";
    let bad = toml.replace("ags/sound", "nope");
    fs::write(src.join("bad.toml"), &bad).unwrap();
    let good = src.join("mod.toml");
    fs::write(&good, toml).unwrap();

    let err = import(&mut ws, &closs, with(&src.join("bad.toml")));
    let err = err.err().unwrap().to_string();
    assert!(err.contains("matches no file in the package"), "{err}");
    assert!(ws.state.mods.is_empty() && !ws.dir.join("mods/closs-v5").exists());
    let opt = Options {
        team: Some("108"),
        ..with(&good)
    };
    let err = import(&mut ws, &closs, opt).err().unwrap();
    assert!(err.to_string().contains("--team does nothing"), "{err}");

    let m = import(&mut ws, &closs, with(&good)).unwrap();
    assert_eq!(
        (m.kind.describe(), m.root.as_str()),
        ("Livecpk", "Closs V5")
    );
    assert_eq!(
        m.files,
        ["SiderAddons/livecpk/Closs V5/eng/sound/acb/t.acb"]
    );
    let dir = ws.dir.join("mods/closs-v5");
    assert_eq!(fs::read_to_string(dir.join("mod.toml")).unwrap(), toml);
    assert!(dir.join("index.json").is_file() && dir.join("Closs V5/mex/string/all.str").is_file());
    assert_eq!(ws.enabled(), ["closs-v5"]);
}

#[test]
fn an_archive_with_a_symlink_is_refused_and_leaves_nothing() {
    let (_, mut ws) = lab("imp-symlink");
    let src = fresh("imp-symlink-src");
    // An escaping target (refused by the zip reader) and one inside the archive, which the zip
    // reader creates and the post-extract check must refuse.
    for (name, target, why) in [
        ("escape", "/etc/passwd", "escapes"),
        ("inner", "render/c.dds", "is a link"),
    ] {
        let zip_path = src.join(format!("{name}.zip"));
        let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path).unwrap());
        let opts = zip::write::SimpleFileOptions::default();
        zip.start_file("Evil/common/render/c.dds", opts).unwrap();
        zip.write_all(b"c").unwrap();
        zip.add_symlink("Evil/common/link", target, opts).unwrap();
        zip.finish().unwrap();
        let err = import(&mut ws, &zip_path, Options::default())
            .err()
            .expect("a symlink entry must be refused")
            .to_string();
        assert!(err.contains(why), "{name}: {err}");
        assert!(!ws.dir.join("staging/archive").exists(), "{name}");
        assert!(!ws.dir.join("mods").join(name).exists(), "{name}");
    }
}
