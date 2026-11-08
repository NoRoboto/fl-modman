//! A Stadium Server stadium → `@stadiums/<name>/` + one `map_teams` row for `--team`.

use super::*;
use crate::codec::map_teams::{MapTeams, normalize_key};

pub struct Stadium;

impl ModKind for Stadium {
    /// A `stadium*` entry, or livecpk content next to the Stadium Server `AddOn/` folder (above
    /// livecpk, which would otherwise take the stadium's `common/`).
    fn detect(&self, top: &Listing) -> u8 {
        let named = |n: &String| n.to_ascii_lowercase().starts_with("stadium");
        let addon = top.dirs.iter().any(|d| d.eq_ignore_ascii_case("AddOn"));
        let server = addon && livecpk::Livecpk.detect(top) > 0;
        let hit = top.files.iter().chain(&top.dirs).any(named) || server;
        if hit { 30 } else { 0 }
    }

    fn contribute(&self, root: &Root) -> Contribution {
        let to = format!("@stadiums/{}/", root.name);
        folder(root, root.name, &to, |m| {
            m.map_teams = vec![row(root)?];
            Ok(())
        })
    }

    fn describe(&self) -> &'static str {
        "Stadium"
    }
}

/// `team, slot, name, folder`; the slot defaults to the team's row in the base.
fn row(root: &Root) -> Result<String> {
    let team = root.team.ok_or(Error::NeedsTeam)?;
    let (id, slot) = team.split_once(',').unwrap_or((team, ""));
    let rows = root.base.docs.get(&root.t.map_teams);
    let rows = rows.map(|d| MapTeams::parse(&d.text).rows()).unwrap_or_default();
    let slot = match (slot.trim(), rows.into_iter().find(|r| r.key == normalize_key(id))) {
        ("", Some(r)) => r.fields[1].clone(),
        ("", None) => {
            return Err(input(format!(
                "team {id} has no base row: use --team {id},<slot>"
            )));
        }
        (s, _) => s.into(),
    };
    Ok(format!("{}, {slot}, {n}, {n}", id.trim(), n = root.name))
}
