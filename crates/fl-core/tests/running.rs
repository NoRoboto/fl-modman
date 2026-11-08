//! Removing is refused while the game runs. Its own test binary: a fake running FL_2027.exe
//! would make every apply in a shared binary fail.

mod common;

use std::os::unix::process::CommandExt;
use std::process::Command;

use common::*;

#[test]
fn nothing_is_removed_while_the_game_runs() {
    let (_, mut ws) = lab("running");
    write_files(&ws.dir, &[("mods/stray/a.bin", "a")]);
    let mut game = Command::new("sleep")
        .arg0(r"Z:\games\FL_2027.exe")
        .arg("10")
        .spawn()
        .unwrap();
    let freed = ws.remove_disabled(&["stray".into()], true);
    let removed = ws.remove("stray");
    game.kill().unwrap();
    game.wait().unwrap();
    let err = freed.unwrap_err().to_string();
    assert!(err.contains("is running"), "{err}");
    assert!(removed.unwrap_err().to_string().contains("is running"));
    assert!(ws.dir.join("mods/stray").is_dir());
}
