//! `settings.dat` codec on a synthetic 612-byte file built from the codec's field layout, and game discovery on
//! a synthetic home folder.

mod common;

use std::fs;

use common::*;
use fl_core::{detect, graphics, settings};

/// Magic, 107, flags 0x87e (vsync on2, 60/60 fps, XInput, P2P, auto UDP), 1280x720, sound
/// buffer 2, UDP port 5739, and a non-zero key mapping block that must survive edits.
fn synthetic() -> Vec<u8> {
    let mut b = vec![0u8; 612];
    b[..4].copy_from_slice(b"WECF");
    for (o, v) in [
        (4, 107u32),
        (8, 0x87e),
        (12, 1280),
        (16, 720),
        (24, 2),
        (28, 5739),
    ] {
        b[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }
    (52..612).for_each(|i| b[i] = i as u8);
    b
}

fn flags(b: &[u8]) -> u32 {
    u32::from_le_bytes(b[8..12].try_into().unwrap())
}

#[test]
fn settings_round_trip() {
    let orig = synthetic();
    let got: Vec<String> = settings::get(&orig)
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    assert_eq!(
        got.join(" "),
        "resolution=1280x720 vsync=on2 sound_buffer=2 udp_port=5739 fullscreen=off fps_game=60 \
         fps_replay=60 xinput=on p2p=on auto_udp=on voice_chat=off hdr=off"
    );

    // The edit verified live on the Ally: fullscreen and vsync off turn 0x87e into 0x27f.
    let mut b = orig.clone();
    settings::set(&mut b, "fullscreen", "on").unwrap();
    settings::set(&mut b, "vsync", "off").unwrap();
    assert_eq!(flags(&b), 0x27f);
    for (k, v) in [
        ("resolution", "1920x1080"),
        ("fps_replay", "30"),
        ("hdr", "on"),
        ("udp_port", "6000"),
    ] {
        settings::set(&mut b, k, v).unwrap();
        assert!(settings::get(&b).contains(&(k, v.to_string())), "{k}");
    }
    for (k, v) in [
        ("fullscreen", "off"),
        ("vsync", "on2"),
        ("resolution", "1280x720"),
    ] {
        settings::set(&mut b, k, v).unwrap();
    }
    for (k, v) in [("fps_replay", "60"), ("hdr", "off"), ("udp_port", "5739")] {
        settings::set(&mut b, k, v).unwrap();
    }
    assert_eq!(b, orig, "set back, every byte is the original");

    assert!(settings::set(&mut b, "vsync", "maybe").is_err());
    assert!(settings::set(&mut b, "quality", "high").is_err());
    assert!(settings::check(b"WECX0000000000000000000000000000").is_err());
}

#[test]
fn settings_update_backs_up_first() {
    let dir = fresh("settings-update");
    let file = dir.join("settings.dat");
    fs::write(&file, synthetic()).unwrap();
    let pairs = ["fullscreen=on".to_string(), "vsync = off".to_string()];
    let bak = settings::update(&dir, &dir, &pairs).unwrap();
    assert_eq!(bak.len(), 1);
    assert_eq!(fs::read(&bak[0]).unwrap(), synthetic());
    assert_eq!(flags(&fs::read(&file).unwrap()), 0x27f);
    assert!(settings::update(&dir, &dir, &["nope".to_string()]).is_err());
    // No GRAPHICS file yet: a render key is refused with the hint and nothing is written.
    let err = settings::update(&dir, &dir, &["hdr=on".into(), "grass=low".into()]).unwrap_err();
    assert!(err.to_string().contains("rendering options menu"), "{err}");
    assert_eq!(flags(&fs::read(&file).unwrap()), 0x27f);
}

/// The preset table (codec/graphics.rs), bytes 3..19, for low, medium, high.
const PRESETS: [[u8; 16]; 3] = [
    [0, 0, 80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 100, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [2, 1, 100, 2, 1, 1, 2, 1, 1, 1, 1, 1, 2, 2, 1, 2],
];

/// A custom file seen on the Ally: unknown `50 50 50`, then High with resolution 95 %.
fn gfx() -> Vec<u8> {
    let mut b = vec![50, 50, 50];
    b.extend(PRESETS[2]);
    b[3..6].copy_from_slice(&[3, 1, 95]);
    b
}

#[test]
fn graphics_get_set() {
    let orig = gfx();
    let got: Vec<String> = graphics::get(&orig)
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    assert_eq!(
        got.join(" "),
        "preset=custom depth_of_field=on render_scale=95 texture_filtering=high motion_blur=on \
         bloom=on ambient_occlusion=high shadows=high postprocessing=on lens=high effects=high \
         cloth=on surroundings=high grass=high goal_nets=high spectators=high"
    );
    for (i, p) in ["low", "medium", "high"].iter().enumerate() {
        let mut b = orig.clone();
        graphics::set(&mut b, "preset", p).unwrap();
        assert_eq!(b[..3], [50, 50, 50], "unknown bytes untouched");
        assert_eq!(b[3..], PRESETS[i], "{p} row");
    }
    // Round trip: every value set and read back; any field change makes the preset custom.
    let mut b = PRESETS[1].to_vec();
    b.splice(0..0, [7, 8, 9]);
    for (k, v) in [
        ("spectators", "low"),
        ("cloth", "off"),
        ("render_scale", "50"),
        ("shadows", "low"),
        ("texture_filtering", "high"),
    ] {
        graphics::set(&mut b, k, v).unwrap();
        assert!(graphics::get(&b).contains(&(k, v.to_string())), "{k}");
        assert_eq!(b[3], 3, "{k} sets custom");
    }
    assert_eq!(
        b,
        [7, 8, 9, 3, 1, 50, 2, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0]
    );
    let mut b = orig.clone();
    graphics::set(&mut b, "render_scale", "95").unwrap();
    assert_eq!(b, orig);

    for (k, v) in [
        ("shadows", "medium"),
        ("render_scale", "49"),
        ("render_scale", "101"),
        ("bloom", "1"),
        ("preset", "ultra"),
        ("quality", "high"),
    ] {
        assert!(graphics::set(&mut b, k, v).is_err(), "{k}={v}");
    }
    assert_eq!(b, orig, "refused values write nothing");
    assert!(graphics::check(&[0; 18]).is_err() && graphics::check(&[0; 20]).is_err());
}

#[test]
fn update_writes_both_files() {
    let dir = fresh("settings-update-both");
    let gfx_file = dir.join(graphics::FILE);
    fs::create_dir_all(gfx_file.parent().unwrap()).unwrap();
    fs::write(dir.join("settings.dat"), synthetic()).unwrap();
    fs::write(&gfx_file, gfx()).unwrap();
    let pairs = ["preset=low", "hdr=on", "grass=high"].map(String::from);
    let baks = settings::update(&dir, &dir, &pairs).unwrap();
    assert_eq!(fs::read(&baks[1]).unwrap(), gfx());
    assert!(
        baks[1]
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("GRAPHICS000000.bak-")
    );
    let b = fs::read(&gfx_file).unwrap();
    assert_eq!((b[3], b[5], b[16]), (3, 80, 2));
    // A bad render value refuses the whole set, settings.dat included.
    let pairs = ["hdr=off", "shadows=medium"].map(String::from);
    assert!(settings::update(&dir, &dir, &pairs).is_err());
    assert!(
        settings::get(&fs::read(dir.join("settings.dat")).unwrap()).contains(&("hdr", "on".into()))
    );
}

#[test]
fn detect_finds_games_and_prefixes() {
    let home = fresh("detect-home");
    let spgame = "drive_c/users/steamuser/Documents/SPGAME/Football Life/settings.dat";
    let pfx = ".local/share/Steam/steamapps/compatdata/4242/pfx";
    write_files(
        &home,
        &[
            ("Games/FL27/FL_2027.exe", ""),
            ("Games/umu/default/drive_c/Games/FL27-LAB/fl_2027.exe", ""),
            (&format!("Games/umu/default/{spgame}"), ""),
            (&format!("{pfx}/{spgame}"), ""),
            (".local/share/Steam/steamapps/common/Other/game.exe", ""),
        ],
    );
    let found = detect::detect(&home, "FL_2027.exe");
    let umu = home.join("Games/umu/default");
    assert_eq!(
        found.games,
        [
            (home.join("Games/FL27"), None),
            (umu.join("drive_c/Games/FL27-LAB"), Some(umu.clone()))
        ]
    );
    assert_eq!(found.prefixes, [home.join(pfx), umu]);
}
