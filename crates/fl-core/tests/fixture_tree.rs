//! Integration tests over the synthetic FL27 tree and the four fixture packages of
//! design.md §10 (Camp Nou row, Closs livecpk, Machete cursors livecpk, EPL cameras).

mod common;

use common::*;
use fl_core::plan::{Op, Replaces};

#[test]
fn scan_excludes_user_paths_and_reads_identity() {
    let b = base();
    assert!(b.files.contains_key("FL_2027.exe"));
    assert!(!b.files.contains_key("SiderAddons/sider.log"));
    assert!(!b.files.contains_key("SiderAddons/camera.cache"));
    assert!(b.files.contains_key("SiderAddons/modules/camera.ini"));
    assert_eq!(b.version.as_deref(), Some("27.1.0"));
    assert_eq!(b.exe_sha256.as_ref().map(String::len), Some(64));
    assert_eq!(b.docs.len(), 2);
    // stable id for an unchanged tree
    assert_eq!(b.id, base().id);
}

#[test]
fn empty_list_reproduces_the_base_byte_for_byte() {
    let b = base();
    let d = built(&b, &fixture_packages(), &[]);
    assert_eq!(d.manifest(), b.files);
    assert_eq!(doc_text(&d, SIDER_INI), base_text(SIDER_INI));
    let p = plan_for(&b, &fixture_packages(), &[], &b.files);
    assert!(p.ops.is_empty(), "{:#?}", p.ops);
}

#[test]
fn fixture_list_renders_roots_and_row() {
    let b = base();
    let pkgs = fixture_packages();
    let mods = list(&["mariano-closs-v5", "machete-cursor", "camp-nou-2026"]);
    let d = fl_core::build::build(&target(), &b, &pkgs, &mods).unwrap();

    // New roots go above the base roots; the lower mod in the list comes first (F1).
    let expected = base_text(SIDER_INI).replace(
        "\tcpk.root = \".\\livecpk\\root\"\r\n",
        "\tcpk.root = \".\\livecpk\\Machete Cursors\"\r\n\
         \tcpk.root = \".\\livecpk\\Mariano Closs V5\"\r\n\
         \tcpk.root = \".\\livecpk\\root\"\r\n",
    );
    assert_eq!(doc_text(&d, SIDER_INI), expected);

    // The row goes after the last row, before the trailing comment; CRLF kept.
    let expected = base_text(MAP_TEAMS).replace(
        "#\u{20}Ajax\r\n",
        "#\u{20}Ajax\r\n108, 002, Camp Nou 2026, Camp Nou 2026\t# Barcelona\r\n",
    );
    assert_eq!(doc_text(&d, MAP_TEAMS), expected);
    assert!(d.conflicts.is_empty());

    let p = plan_for(&b, &pkgs, &mods, &b.files);
    let text = p.render_text();
    assert!(
        text.contains("Plan: 11 put (108 B), 0 del, 2 docs, 0 conflicts."),
        "{text}"
    );
    assert!(text.contains("+ put   7 files (e.g. SiderAddons/livecpk/Mariano Closs V5/eng/"));
    assert!(text.contains("+ cpk.root = \".\\livecpk\\Machete Cursors\"   machete-cursor"));
    // docs come after the content they activate
    let first_doc = p
        .ops
        .iter()
        .position(|o| matches!(o, Op::Doc { .. }))
        .unwrap();
    assert!(
        p.ops[first_doc..]
            .iter()
            .all(|o| matches!(o, Op::Doc { .. }))
    );
}

#[test]
fn camera_modules_append_after_the_base_modules() {
    let b = base();
    let d = built(&b, &fixture_packages(), &["camera-epl"]);
    let lua = ini_values(&d, "lua.module");
    assert_eq!(
        &lua[lua.len() - 5..],
        [
            "movieintro.lua",
            "BroadCastCam.lua",
            "FanViewCam.lua",
            "PenaltyCam.lua",
            "StadiumCam.lua"
        ]
    );
    let text = doc_text(&d, SIDER_INI);
    assert!(text.contains("\"StadiumCam.lua\"\r\n\r\n\r\n\toverlay.enabled"));
    assert!(d.files.contains_key("SiderAddons/modules/BroadCastCam.ini"));
}

#[test]
fn disabling_everything_plans_back_to_base() {
    let b = base();
    let pkgs = fixture_packages();
    let all = list(&[
        "mariano-closs-v5",
        "machete-cursor",
        "camp-nou-2026",
        "camera-epl",
    ]);
    let deployed = apply(&b.files, &plan_for(&b, &pkgs, &all, &b.files));
    assert_ne!(deployed, b.files);
    let back = plan_for(&b, &pkgs, &[], &deployed);
    let docs = back.ops.iter().filter(|o| matches!(o, Op::Doc { .. }));
    assert_eq!(docs.count(), 2);
    assert!(back.ops.iter().any(|o| matches!(o, Op::Del { .. })));
    assert_eq!(apply(&deployed, &back), b.files);
}

#[test]
fn untracked_files_are_never_deleted() {
    let b = base();
    let mut current = b.files.clone();
    let stray = "SiderAddons/livecpk/HandInstalled/x.bin".to_string();
    current.insert(stray, b.files["FL_2027.exe"].clone());
    let p = plan_for(&b, &fixture_packages(), &[], &current);
    assert!(p.ops.is_empty());
}

#[test]
fn restoring_a_missing_base_file_reports_replacement_kind() {
    let b = base();
    let mut current = b.files.clone();
    current.remove("Data/dt41_eng_all.cpk");
    let p = plan_for(&b, &fixture_packages(), &[], &current);
    assert!(matches!(
        &p.ops[..],
        [Op::Put {
            replaces: Replaces::Nothing,
            by: None,
            ..
        }]
    ));
}
