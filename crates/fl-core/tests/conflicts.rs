//! Conflicts resolved by list order (the lower mod wins), the exe guard, and input and
//! manifest errors, on synthetic packages over the fixture tree.

mod common;

use common::*;
use fl_core::build::build;
use fl_core::manifest::Package;

fn put(group: &str, id: &str, to: &str, body: &str) -> Package {
    let toml = format!("[[put]]\nfrom = \"x\"\nto = \"{to}\"\n");
    write_pkg(group, id, &toml, &[("x", body)])
}

#[test]
fn file_conflict_goes_to_the_lower_mod_and_is_reported() {
    let p = pkgs(vec![
        put("file", "a", "@data/dt18_all.cpk", "AAA"),
        put("file", "b", "@data/dt18_all.cpk", "BBB"),
    ]);
    let b = base();
    for (order, winner, loser) in [(["a", "b"], "b", "a"), (["b", "a"], "a", "b")] {
        let d = built(&b, &p, &order);
        assert_eq!(d.files["Data/dt18_all.cpk"].by.as_deref(), Some(winner));
        let c = &d.conflicts[0];
        assert_eq!((c.winner.as_str(), c.loser.as_str()), (winner, loser));
        assert_eq!(c.items, ["Data/dt18_all.cpk"]);
    }
    let text = plan_for(&b, &p, &list(&["a", "b"]), &b.files).render_text();
    assert!(text.contains("! b overrides 1 file(s) of a (e.g. Data/dt18_all.cpk)"));
}

#[test]
fn identical_puts_are_not_a_conflict() {
    let p = pkgs(vec![
        put("same", "a", "@data/x.cpk", "same"),
        put("same", "b", "@data/x.cpk", "same"),
    ]);
    assert!(built(&base(), &p, &["a", "b"]).conflicts.is_empty());
}

#[test]
fn replacing_a_download_cpk_keeps_the_base_original_reported() {
    let p = pkgs(vec![put(
        "dl",
        "faces",
        "@download/01_smkdb_fa1.cpk",
        "new",
    )]);
    let b = base();
    let d = built(&b, &p, &["faces"]);
    assert!(
        d.conflicts.is_empty(),
        "replacing base is not a mod conflict"
    );
    let text = plan_for(&b, &p, &list(&["faces"]), &b.files).render_text();
    assert!(text.contains("~ put   download/01_smkdb_fa1.cpk   3 B   replaces base   faces"));
}

#[test]
fn loose_file_in_the_game_root() {
    let p = pkgs(vec![put("loose", "reshade", "@game/dxgi.dll", "dll")]);
    let d = built(&base(), &p, &["reshade"]);
    assert_eq!(d.files["dxgi.dll"].by.as_deref(), Some("reshade"));
}

#[test]
fn a_team_row_replaces_the_base_rows_of_that_team() {
    let b = base_with_fl26_map_teams();
    let d = built(&b, &fixture_packages(), &["camp-nou-2026"]);
    let text = doc_text(&d, MAP_TEAMS);
    assert!(!text.contains("Spotify Camp Nou"));
    assert!(text.contains("108, 002, Camp Nou 2026, Camp Nou 2026\t# Barcelona"));
    let removed: Vec<_> = d.docs[MAP_TEAMS]
        .changes
        .iter()
        .filter(|c| !c.added)
        .collect();
    assert_eq!(removed.len(), 1);
    assert!(d.conflicts.is_empty(), "the base is not a mod");
}

#[test]
fn a_row_identical_to_the_base_row_leaves_the_file_unchanged() {
    let b = base_with_fl26_map_teams();
    // "0108" == "108" numerically, as stadiums.lua compares
    let toml = "map_teams = [\"0108, 002, Spotify Camp Nou, common\\\\Camp Nou\"]\n";
    let p = pkgs(vec![write_pkg("same-row", "same", toml, &[])]);
    let d = built(&b, &p, &["same"]);
    assert_eq!(d.docs[MAP_TEAMS].hash, b.files[MAP_TEAMS]);
    assert!(d.docs[MAP_TEAMS].changes.is_empty());
}

#[test]
fn two_mods_rows_for_one_team_go_to_the_lower_mod() {
    let row = |id: &str, name: &str| {
        let toml = format!("map_teams = [\"500, 001, {name}, {name}\"]\n");
        write_pkg("rows", id, &toml, &[])
    };
    let p = pkgs(vec![row("s1", "One"), row("s2", "Two")]);
    let d = built(&base(), &p, &["s1", "s2"]);
    let text = doc_text(&d, MAP_TEAMS);
    assert!(text.contains("500, 001, Two, Two") && !text.contains("One"));
    let c = &d.conflicts[0];
    assert_eq!((c.winner.as_str(), c.loser.as_str()), ("s2", "s1"));
    assert_eq!(c.items, ["map_teams team 500"]);
}

#[test]
fn shadowed_virtual_paths_go_to_the_lower_mod() {
    let root = |id: &str| {
        let toml = format!(
            "cpk_root = ['.\\livecpk\\{id}']\n[[put]]\nfrom = \"r/\"\nto = \"@livecpk/{id}/\"\n"
        );
        write_pkg("shadow", id, &toml, &[("r/mex/string/all.str", id)])
    };
    let p = pkgs(vec![root("closs"), root("menu")]);
    let d = built(&base(), &p, &["closs", "menu"]);
    let roots = ini_values(&d, "cpk.root");
    assert_eq!(&roots[..2], [r".\livecpk\menu", r".\livecpk\closs"]);
    let c = &d.conflicts[0];
    assert_eq!((c.winner.as_str(), c.loser.as_str()), ("menu", "closs"));
    assert_eq!(c.items, ["<virtual> mex/string/all.str"]);
}

#[test]
fn values_already_in_the_base_are_idempotent() {
    let toml = "cpk_root = ['.\\LiveCpk/Ultimate_ATM']\nlua_module = ['LIB\\nesalib.lua']\n";
    let p = pkgs(vec![write_pkg("idem", "idem", toml, &[])]);
    let b = base();
    let d = built(&b, &p, &["idem"]);
    assert_eq!(doc_text(&d, SIDER_INI), base_text(SIDER_INI));
}

#[test]
fn exe_guard_disables_the_mod_with_a_warning() {
    let b = base();
    let exe = b.exe_sha256.clone().unwrap().to_uppercase();
    let p = pkgs(vec![
        write_pkg("exe", "pinned", "exe = \"00ff\"\n", &[]),
        write_pkg("exe", "matching", &format!("exe = \"{exe}\"\n"), &[]),
    ]);
    let d = built(&b, &p, &["pinned", "matching"]);
    assert_eq!(d.mods, ["matching"]);
    assert!(d.warnings[0].starts_with("pinned disabled: made for exe 00ff"));
}

#[test]
fn input_errors() {
    let (t, b, p) = (target(), base(), fixture_packages());
    assert!(build(&t, &b, &p, &list(&["nope"])).is_err());
    assert!(build(&t, &b, &p, &list(&["camp-nou-2026", "camp-nou-2026"])).is_err());
}

#[test]
fn manifest_errors() {
    let bad = [
        ("absolute root", "cpk_root = ['Z:\\mods\\x']\n"),
        ("quoted value", "lua_module = ['a\"b.lua']\n"),
        ("short row", "map_teams = [\"1, 2\"]\n"),
        (
            "two rows",
            "map_teams = [\"1, 2, a, a\", \"01, 3, b, b\"]\n",
        ),
        (
            "doc put",
            "[[put]]\nfrom = \"f\"\nto = \"@sider/sider.ini\"\n",
        ),
        (
            "user put",
            "[[put]]\nfrom = \"f\"\nto = \"@sider/sider.log\"\n",
        ),
        ("unknown root", "[[put]]\nfrom = \"f\"\nto = \"@save/x\"\n"),
        ("escape", "[[put]]\nfrom = \"f\"\nto = \"@game/../x\"\n"),
        (
            "missing source",
            "[[put]]\nfrom = \"nope/\"\nto = \"@livecpk/x/\"\n",
        ),
        (
            "folder to file",
            "[[put]]\nfrom = \"d/\"\nto = \"@livecpk/x\"\n",
        ),
        (
            "unknown field",
            "[[put]]\nfrom = \"f\"\nto = \"@game/\"\nwhen = \"x\"\n",
        ),
        (
            "removed field",
            "[[put]]\nfrom = \"f\"\nto = \"@game/\"\n[[seed]]\nfrom = \"f\"\nto = \"@game/\"\n",
        ),
    ];
    for (what, body) in bad {
        let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("bad")
            .join(what.replace(' ', "-"));
        std::fs::create_dir_all(dir.join("d")).unwrap();
        std::fs::write(dir.join("f"), "x").unwrap();
        std::fs::write(dir.join("d/g"), "x").unwrap();
        std::fs::write(dir.join("mod.toml"), body).unwrap();
        assert!(
            Package::load(&dir, &target()).is_err(),
            "{what} should be rejected"
        );
    }
}

#[test]
fn index_keys_must_be_normalized_paths() {
    let toml = "[[put]]\nfrom = \"f\"\nto = \"@game/\"\n";
    let pkg = write_pkg("bad-index", "ok", toml, &[("f", "x")]);
    for key in ["a/../../x", "/etc/passwd", "./f", "d\\f"] {
        let json = format!(
            "{{\"{}\": {{\"b3\": \"00\", \"size\": 1}}}}",
            key.replace('\\', "\\\\")
        );
        std::fs::write(pkg.dir.join("index.json"), json).unwrap();
        let err = Package::load(&pkg.dir, &target()).err();
        let err = err
            .unwrap_or_else(|| panic!("{key} should be rejected"))
            .to_string();
        assert!(
            err.contains("not a normalized relative path"),
            "{key}: {err}"
        );
    }
}
