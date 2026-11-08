//! FL27 `GRAPHICS000000`: the in-game rendering options, one byte per field. Bytes 0–2 are
//! unknown and never written. The layout was found by changing one in-game option at a time and
//! diffing the file. No game code or text is included.

use crate::error::{Result, input};

/// The file, relative to `SPGAME/Football Life`.
pub const FILE: &str = "2027/save/GRAPHICS000000";

const OFF_ON: &[&str] = &["off", "on"];
const LOW_HIGH: &[&str] = &["low", "high"];
const LMH: &[&str] = &["low", "medium", "high"];

/// (key, values by byte value) for bytes 3..19; `render_scale` (no values) is a plain percent.
pub const FIELDS: [(&str, &[&str]); 16] = [
    ("preset", &["low", "medium", "high", "custom"]),
    ("depth_of_field", OFF_ON),
    ("render_scale", &[]),
    ("texture_filtering", LMH),
    ("motion_blur", OFF_ON),
    ("bloom", OFF_ON),
    ("ambient_occlusion", LMH),
    ("shadows", LOW_HIGH),
    ("postprocessing", OFF_ON),
    ("lens", LOW_HIGH),
    ("effects", LOW_HIGH),
    ("cloth", OFF_ON),
    ("surroundings", LMH),
    ("grass", LMH),
    ("goal_nets", LOW_HIGH),
    ("spectators", LMH),
];

/// Bytes 3..19 the game writes for the low, medium and high presets.
const PRESETS: [[u8; 16]; 3] = [
    [0, 0, 80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 100, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [2, 1, 100, 2, 1, 1, 2, 1, 1, 1, 1, 1, 2, 2, 1, 2],
];

pub fn is_key(key: &str) -> bool {
    FIELDS.iter().any(|f| f.0 == key)
}

/// The accepted values of a field (`render_scale`: 50..=100 in steps of 5 for pickers).
pub fn options(key: &str) -> Vec<String> {
    match FIELDS.iter().find(|f| f.0 == key) {
        Some((_, [])) => (50..=100).step_by(5).map(|p: u8| p.to_string()).collect(),
        Some((_, vals)) => vals.iter().map(|v| v.to_string()).collect(),
        None => Vec::new(),
    }
}

pub fn check(b: &[u8]) -> Result<()> {
    match b.len() {
        19 => Ok(()),
        n => Err(input(format!("GRAPHICS000000 is {n} bytes, not 19"))),
    }
}

/// Every field as `(key, value)`; a byte outside the known values shows as its number.
pub fn get(b: &[u8]) -> Vec<(&'static str, String)> {
    let text = |(&(k, vals), &v): (&(&'static str, &[&str]), &u8)| {
        let named = vals.get(usize::from(v)).map(|s| s.to_string());
        (k, named.unwrap_or(v.to_string()))
    };
    FIELDS.iter().zip(&b[3..]).map(text).collect()
}

/// Sets one field like the game: a preset writes its whole row, any other field sets custom.
pub fn set(b: &mut [u8], key: &str, val: &str) -> Result<()> {
    let bad = || input(format!("bad value {val:?} for {key}"));
    let i = FIELDS.iter().position(|f| f.0 == key);
    let i = i.ok_or_else(|| input(format!("unknown setting {key:?}")))?;
    let v = match FIELDS[i].1 {
        [] => val.parse().ok().filter(|p| (50..=100).contains(p)),
        vals => vals.iter().position(|v| *v == val).map(|p| p as u8),
    };
    match (i, v.ok_or_else(bad)?) {
        (0, p @ 0..=2) => b[3..19].copy_from_slice(&PRESETS[usize::from(p)]),
        (_, v) => (b[i + 3], b[3]) = (v, 3),
    }
    Ok(())
}
