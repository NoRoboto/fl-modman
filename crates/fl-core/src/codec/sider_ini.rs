//! `sider.ini` codec. Holds the multi-key documents `cpk.root` and `lua.module`; every other
//! line (section header, scalar keys, `;` comments, commented-out entries, blank lines) is
//! trivia and is rendered back unchanged.

use super::{Eol, Line, dominant_eol, fix_terminators, join_lines, split_lines};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiderIni {
    lines: Vec<Line>,
}

/// An active `key = value` line. `value` has its surrounding quotes removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IniEntry {
    pub line: usize,
    pub value: String,
}

impl SiderIni {
    pub fn parse(text: &str) -> Self {
        SiderIni {
            lines: split_lines(text),
        }
    }

    pub fn render(&self) -> String {
        join_lines(&self.lines)
    }

    /// Active entries for `key` (key compared ASCII case-insensitively), in file order.
    pub fn entries(&self, key: &str) -> Vec<IniEntry> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(i, l)| {
                let (k, v) = parse_kv(&l.text)?;
                k.eq_ignore_ascii_case(key).then(|| IniEntry {
                    line: i,
                    value: v.to_string(),
                })
            })
            .collect()
    }

    /// Inserts `key = "value"` lines before the first entry of `key` (`front`) or after the
    /// last one, copying the indentation of the existing entries. With no entries they go at
    /// the end of the file.
    pub fn insert(&mut self, key: &str, values: &[&str], front: bool) {
        let entries = self.entries(key);
        let dominant = dominant_eol(&self.lines);
        let unterminated = self.lines.last().is_some_and(|l| l.eol == Eol::None);
        let indent = entries
            .first()
            .map(|e| e.line)
            .or_else(|| self.lines.iter().position(|l| parse_kv(&l.text).is_some()))
            .map(|i| leading_ws(&self.lines[i].text).to_string())
            .unwrap_or_default();
        let at = match (entries.first(), entries.last()) {
            (Some(first), _) if front => first.line,
            (_, Some(last)) => last.line + 1,
            _ => self.lines.len(),
        };
        let new = values.iter().map(|v| Line {
            text: format!("{indent}{key} = \"{v}\""),
            eol: dominant,
        });
        self.lines.splice(at..at, new);
        fix_terminators(&mut self.lines, dominant, unterminated);
    }
}

fn leading_ws(s: &str) -> &str {
    &s[..s.len() - s.trim_start().len()]
}

fn parse_kv(text: &str) -> Option<(&str, &str)> {
    let t = text.trim();
    if t.is_empty() || t.starts_with([';', '#', '[']) {
        return None;
    }
    let (k, v) = t.split_once('=')?;
    let v = v.trim();
    let v = v
        .strip_prefix('"')
        .and_then(|x| x.strip_suffix('"'))
        .unwrap_or(v);
    Some((k.trim(), v))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FL26: &str = include_str!("../../../../tests/fixtures/fl26/sider.ini");

    #[test]
    fn fl26_shaped_round_trips() {
        assert_eq!(SiderIni::parse(FL26).render(), FL26);
        let mut ini = SiderIni::parse(FL26);
        ini.insert("cpk.root", &[], true);
        ini.insert("lua.module", &[], false);
        assert_eq!(ini.render(), FL26);
    }

    #[test]
    fn fl26_shaped_entries() {
        let ini = SiderIni::parse(FL26);
        let roots: Vec<_> = ini
            .entries("cpk.root")
            .into_iter()
            .map(|e| e.value)
            .collect();
        assert_eq!(roots.len(), 5);
        assert_eq!(roots[0], r".\livecpk\root");
        // space-indented line among tab-indented ones
        assert_eq!(roots[1], r".\livecpk\Next Gen Graphics");
        assert_eq!(ini.entries("lua.module").len(), 18);
        // duplicated scalar is trivia, still present twice
        assert_eq!(ini.entries("match-stats.enabled").len(), 2);
    }

    #[test]
    fn insert_front_and_back() {
        let src =
            "[sider]\r\n\tcpk.root = \"a\"\r\n\tcpk.root = \"b\"\r\n; cpk.root = \"old\"\r\nx = 1";
        let mut ini = SiderIni::parse(src);
        ini.insert("cpk.root", &["top"], true);
        ini.insert("cpk.root", &["end"], false);
        assert_eq!(
            ini.render(),
            "[sider]\r\n\tcpk.root = \"top\"\r\n\tcpk.root = \"a\"\r\n\tcpk.root = \"b\"\r\n\tcpk.root = \"end\"\r\n; cpk.root = \"old\"\r\nx = 1"
        );
    }

    #[test]
    fn insert_after_unterminated_last_line() {
        let mut ini = SiderIni::parse("lua.module = \"a\"");
        ini.insert("lua.module", &["b"], false);
        assert_eq!(ini.render(), "lua.module = \"a\"\nlua.module = \"b\"");
    }

    #[test]
    fn insert_without_entries_appends() {
        let mut ini = SiderIni::parse("x = 1\n");
        ini.insert("lua.module", &["b"], true);
        assert_eq!(ini.render(), "x = 1\nlua.module = \"b\"\n");
    }
}
