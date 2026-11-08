//! FL27 `settings.dat`: little-endian, magic `WECF`. Only the documented fields are read or
//! written; every other byte is kept. Implements the on-disk layout of `settings.dat` (plain
//! file-format facts). No game code or text is included.

use crate::error::{Result, input};

/// Bits of the flags word at offset 8: (key, bit, value when set, value when clear).
const BITS: [(&str, u32, &str, &str); 8] = [
    ("fullscreen", 0, "on", "off"),
    ("fps_game", 2, "60", "30"),
    ("fps_replay", 3, "60", "30"),
    ("xinput", 4, "on", "off"),
    ("p2p", 5, "on", "off"),
    ("auto_udp", 6, "on", "off"),
    ("voice_chat", 7, "on", "off"),
    ("hdr", 8, "on", "off"),
];
/// The three exclusive vsync bits.
const VSYNC: [(&str, u32); 3] = [("off", 9), ("on1", 10), ("on2", 11)];
/// u32 fields: (key, offset).
const U32S: [(&str, usize); 2] = [("sound_buffer", 24), ("udp_port", 28)];

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().expect("4 bytes"))
}

fn put(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

pub fn check(b: &[u8]) -> Result<()> {
    match b.len() >= 32 && b.starts_with(b"WECF") {
        true => Ok(()),
        false => Err(input("not a settings.dat (no WECF magic)")),
    }
}

/// Every known field as `(key, value)`.
pub fn get(b: &[u8]) -> Vec<(&'static str, String)> {
    let f = u32_at(b, 8);
    let vsync = VSYNC
        .iter()
        .find(|v| f >> v.1 & 1 == 1)
        .map_or("unset", |v| v.0);
    let mut out = vec![("resolution", format!("{}x{}", u32_at(b, 12), u32_at(b, 16)))];
    out.push(("vsync", vsync.to_string()));
    out.extend(U32S.map(|(k, o)| (k, u32_at(b, o).to_string())));
    out.extend(BITS.map(|(k, n, on, off)| (k, if f >> n & 1 == 1 { on } else { off }.into())));
    out
}

/// Sets one field from its text form (as printed by [`get`]).
pub fn set(b: &mut [u8], key: &str, val: &str) -> Result<()> {
    let bad = || input(format!("bad value {val:?} for {key}"));
    let num = |s: &str| s.parse::<u32>().map_err(|_| bad());
    let mut f = u32_at(b, 8);
    let mut flag = |n: u32, on: bool| f = f & !(1 << n) | u32::from(on) << n;
    if key == "resolution" {
        let (w, h) = val.split_once('x').ok_or_else(bad)?;
        let (w, h) = (num(w)?, num(h)?);
        put(b, 12, w);
        put(b, 16, h);
    } else if key == "vsync" {
        let v = VSYNC.iter().find(|v| v.0 == val).ok_or_else(bad)?.1;
        VSYNC.iter().for_each(|(_, n)| flag(*n, *n == v));
    } else if let Some((_, o)) = U32S.iter().find(|k| k.0 == key) {
        put(b, *o, num(val)?);
    } else if let Some((_, n, on, off)) = BITS.iter().find(|k| k.0 == key) {
        flag(
            *n,
            [*off, *on].iter().position(|v| *v == val).ok_or_else(bad)? == 1,
        );
    } else {
        return Err(input(format!("unknown setting {key:?}")));
    }
    put(b, 8, f);
    Ok(())
}
