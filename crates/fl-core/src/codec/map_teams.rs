//! Stadium server `map_teams.txt` codec (F5, F6).
//!
//! A row is `teamId, stadiumSlot, name, folder # comment`. Mirroring `stadiums.lua`: a UTF-8
//! BOM is stripped, everything after the first `#` is a comment, fields are split on `,` and
//! trimmed, and a line is a row only with at least four fields and a non-empty first field.
//! Rows with the same team id form a set (exact duplicates skipped); Sider picks one at random.

use super::{Eol, Line, dominant_eol, fix_terminators, join_lines, split_lines};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapTeams {
    lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub line: usize,
    /// Normalized team id (see [`normalize_key`]).
    pub key: String,
    pub fields: Vec<String>,
}

impl MapTeams {
    pub fn parse(text: &str) -> Self {
        MapTeams {
            lines: split_lines(text),
        }
    }

    pub fn render(&self) -> String {
        join_lines(&self.lines)
    }

    pub fn rows(&self) -> Vec<Row> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(i, l)| {
                let fields = parse_row(&l.text)?;
                Some(Row {
                    line: i,
                    key: normalize_key(&fields[0]),
                    fields,
                })
            })
            .collect()
    }

    /// Removes the row lines in `remove` and appends the raw lines `append` after the last
    /// row line (or at the end when there are no rows), so trailing comments stay last.
    pub fn splice(&mut self, remove: &[usize], append: &[&str]) {
        let dominant = dominant_eol(&self.lines);
        let unterminated = self.lines.last().is_some_and(|l| l.eol == Eol::None);
        let anchor = self.rows().last().map(|r| r.line);
        let mut new: Vec<Line> = append
            .iter()
            .map(|r| Line {
                text: r.to_string(),
                eol: dominant,
            })
            .collect();
        let mut out = Vec::with_capacity(self.lines.len() + new.len());
        for (i, line) in self.lines.iter().enumerate() {
            if !remove.contains(&i) {
                out.push(line.clone());
            }
            if Some(i) == anchor {
                out.append(&mut new);
            }
        }
        out.append(&mut new);
        fix_terminators(&mut out, dominant, unterminated);
        self.lines = out;
    }
}

/// Team ids are compared numerically, as `tonumber` does in `stadiums.lua` ("007" == "7").
pub fn normalize_key(field: &str) -> String {
    let f = field.trim_start_matches('\u{feff}').trim();
    f.parse::<u64>()
        .map_or_else(|_| f.to_string(), |n| n.to_string())
}

/// The part of a row that `stadiums.lua` compares for duplicates: slot, name, folder.
pub fn row_identity(fields: &[String]) -> &[String] {
    &fields[1..fields.len().min(4)]
}

/// The trimmed fields of a row line, or `None` if the line is not a row.
pub fn parse_row(text: &str) -> Option<Vec<String>> {
    let t = text.trim_start_matches('\u{feff}');
    let data = t.split_once('#').map_or(t, |(d, _)| d);
    let fields: Vec<String> = data.split(',').map(|f| f.trim().to_string()).collect();
    (fields.len() >= 4 && !fields[0].is_empty()).then_some(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FL26: &str = include_str!("../../../../tests/fixtures/fl26/map_teams.txt");

    #[test]
    fn fl26_shaped_round_trips() {
        assert_eq!(MapTeams::parse(FL26).render(), FL26);
        let mut m = MapTeams::parse(FL26);
        m.splice(&[], &[]);
        assert_eq!(m.render(), FL26);
    }

    #[test]
    fn fl26_shaped_rows_and_quirks() {
        let rows = MapTeams::parse(FL26).rows();
        // header comments and "#Addons Stadiums:" are not rows
        assert!(rows.iter().all(|r| !r.fields[0].starts_with('#')));
        let camp_nou = rows.iter().find(|r| r.key == "108").unwrap();
        assert_eq!(camp_nou.fields[3], r"common\Camp Nou");
        // tab after the comma on "12,\t006" and a trailing tab before the comment
        let olimpico = rows.iter().find(|r| r.key == "12").unwrap();
        assert_eq!(olimpico.fields[1], "006");
        assert_eq!(olimpico.fields[3], r"common\Stadio Olimpico");
    }

    #[test]
    fn bom_and_numeric_keys() {
        let m = MapTeams::parse("\u{feff}007, 001, A, B\n7, 001, A, B # dup\n");
        let rows = m.rows();
        assert_eq!(rows[0].key, "7");
        assert_eq!(rows[1].key, "7");
        assert_eq!(row_identity(&rows[0].fields), row_identity(&rows[1].fields));
    }

    #[test]
    fn append_after_last_row_before_trailing_comment() {
        let mut m = MapTeams::parse("# head\r\n1, 001, A, A\r\n\r\n# end\r\n");
        m.splice(&[], &["108, 002, Camp Nou, Camp Nou\t# Barcelona"]);
        assert_eq!(
            m.render(),
            "# head\r\n1, 001, A, A\r\n108, 002, Camp Nou, Camp Nou\t# Barcelona\r\n\r\n# end\r\n"
        );
    }

    #[test]
    fn remove_row() {
        let mut m = MapTeams::parse("1, 001, A, A\n2, 002, B, B\n");
        m.splice(&[0], &[]);
        assert_eq!(m.render(), "2, 002, B, B\n");
    }
}
