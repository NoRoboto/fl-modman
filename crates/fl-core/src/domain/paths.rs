//! Relative game-tree paths and a small glob matcher (for user paths).
//!
//! Paths inside the model are `String`s relative to the game root, `/`-separated, with no `.`
//! or `..` components. This keeps the model OS-agnostic; conversion to native paths happens
//! only at the filesystem edge.

use crate::error::{Error, Result};

/// Normalizes a relative path: `\` becomes `/`, empty and `.` components are dropped.
/// Rejects absolute paths, drive letters and `..`.
pub fn normalize(raw: &str) -> Result<String> {
    let unified = raw.replace('\\', "/");
    if unified.starts_with('/') || unified.as_bytes().get(1) == Some(&b':') {
        return Err(Error::Path(raw.into(), "must be relative"));
    }
    let mut parts = Vec::new();
    for part in unified.split('/') {
        match part {
            "" | "." => {}
            ".." => return Err(Error::Path(raw.into(), "must not contain '..'")),
            p => parts.push(p),
        }
    }
    Ok(parts.join("/"))
}

/// The lowercase extension of a file name.
pub fn ext(name: &str) -> String {
    name.rsplit_once('.')
        .map_or(String::new(), |(_, e)| e.to_ascii_lowercase())
}

/// Joins two normalized relative paths.
pub fn join(base: &str, rest: &str) -> String {
    match (base.is_empty(), rest.is_empty()) {
        (true, _) => rest.to_string(),
        (_, true) => base.to_string(),
        _ => format!("{base}/{rest}"),
    }
}

/// True if `path` equals `dir` or lies below it. Both are normalized; `""` is the root.
pub fn is_within(path: &str, dir: &str) -> bool {
    dir.is_empty()
        || path == dir
        || (path.len() > dir.len() && path.starts_with(dir) && path.as_bytes()[dir.len()] == b'/')
}

/// Case-insensitive variant of [`is_within`], for lookups the game performs (Windows/Wine).
pub fn is_within_ci(path: &str, dir: &str) -> bool {
    dir.is_empty()
        || (path.len() >= dir.len()
            && path[..dir.len()].eq_ignore_ascii_case(dir)
            && (path.len() == dir.len() || path.as_bytes()[dir.len()] == b'/'))
}

/// Matches a `/`-separated path against a glob whose `*` and `?` match within one segment.
/// Matching is ASCII case-insensitive, like the game's lookups.
pub fn glob_match(pattern: &str, path: &str) -> bool {
    let pat: Vec<&str> = pattern.split('/').collect();
    let segs: Vec<&str> = path.split('/').collect();
    match_segments(&pat, &segs)
}

fn match_segments(pat: &[&str], segs: &[&str]) -> bool {
    match pat.split_first() {
        None => segs.is_empty(),
        Some((p, rest)) => match segs.split_first() {
            Some((s, srest)) => {
                match_segment(p.as_bytes(), s.as_bytes()) && match_segments(rest, srest)
            }
            None => false,
        },
    }
}

fn match_segment(p: &[u8], s: &[u8]) -> bool {
    match p.split_first() {
        None => s.is_empty(),
        Some((b'*', rest)) => (0..=s.len()).any(|i| match_segment(rest, &s[i..])),
        Some((b'?', rest)) => !s.is_empty() && match_segment(rest, &s[1..]),
        Some((c, rest)) => {
            s.first().is_some_and(|x| x.eq_ignore_ascii_case(c)) && match_segment(rest, &s[1..])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_unifies_separators() {
        assert_eq!(normalize(r".\livecpk\Faces/").unwrap(), "livecpk/Faces");
        assert_eq!(normalize("a//b/./c").unwrap(), "a/b/c");
        assert!(normalize("../x").is_err());
        assert!(normalize("/abs").is_err());
        assert!(normalize(r"C:\Games").is_err());
    }

    #[test]
    fn within() {
        assert!(is_within("a/b/c", "a/b"));
        assert!(is_within("a/b", "a/b"));
        assert!(!is_within("a/bc", "a/b"));
        assert!(is_within("x", ""));
        assert!(is_within_ci("SiderAddons/LiveCpk/x", "SiderAddons/livecpk"));
        assert!(!is_within_ci("SiderAddons/livecpkx", "SiderAddons/livecpk"));
    }

    #[test]
    fn globs() {
        assert!(glob_match("SiderAddons/*.log", "SiderAddons/sider.log"));
        assert!(glob_match("SiderAddons/*.log", "SiderAddons/SIDER.LOG"));
        assert!(!glob_match(
            "SiderAddons/*.log",
            "SiderAddons/sub/sider.log"
        ));
        assert!(glob_match("x/?.lua", "x/a.lua"));
    }
}
