//! The workspace next to the game (grilling Q11): `state.toml`, the base snapshot, `mods/<id>/`
//! (`index`), `originals/` (base files a mod replaced, for undo), `history/NNNN.toml`
//! (generations) and `hashes.json` (`scan`). `apply` records an in-progress generation, applies
//! file by file and marks it complete (Q12), behind a drift check against the last complete one
//! (Q9). `import` adds mods, `settings` edits the game's settings files.

mod apply;
pub mod import;
pub mod index;
mod scan;
pub mod settings;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::HOST;
use crate::adapters::fs::{read_toml, write_atomic, write_toml};
use crate::base::{Snapshot, TreeManifest};
use crate::error::{Result, input, io};
use crate::target::Target;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    pub game: PathBuf,
    pub prefix: Option<PathBuf>,
    /// Base snapshot file, relative to the workspace.
    pub base: String,
    /// The mod list, ascending priority: the lower mod wins.
    pub mods: Vec<String>,
    pub disabled: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    InProgress,
    #[default]
    Complete,
    RolledBack,
}

/// A generation: the mods in effect and the expected tree, as the base snapshot plus `files`,
/// the paths whose content differs from it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Generation {
    pub status: Status,
    pub note: String,
    pub base: String,
    pub mods: Vec<String>,
    pub files: TreeManifest,
}

pub struct Workspace {
    pub dir: PathBuf,
    pub t: Target,
    pub state: State,
}

impl Workspace {
    /// Creates the workspace and takes the base snapshot of the game tree as it is (Q8).
    pub fn init(dir: &Path, game: &Path, prefix: Option<PathBuf>) -> Result<Self> {
        let t = Target::fl27();
        let game = fs::canonicalize(game).map_err(io(game))?;
        if !game.join(&t.exe).is_file() || dir.join("state.toml").exists() {
            return Err(input(format!(
                "no {} in the game dir, or the workspace exists",
                t.exe
            )));
        }
        for d in ["mods", "originals", "history"] {
            fs::create_dir_all(dir.join(d)).map_err(io(dir))?;
        }
        let dir = fs::canonicalize(dir).map_err(io(dir))?;
        if dir.starts_with(&game) {
            return Err(input("the workspace must be outside the game tree"));
        }
        let mut skipped = Vec::new();
        let files = scan::cached(&dir, &game, &t, None, &mut skipped)?;
        let snap = Snapshot::from_files(&game, &t, files, skipped)?;
        let state = State {
            game,
            prefix,
            ..Default::default()
        };
        let mut ws = Workspace { dir, t, state };
        ws.save_base(&snap)?;
        Ok(ws)
    }

    pub fn open(dir: &Path) -> Result<Self> {
        let state = read_toml(&dir.join("state.toml"))?;
        Ok(Workspace {
            dir: dir.into(),
            t: Target::fl27(),
            state,
        })
    }

    pub fn save(&self) -> Result<()> {
        write_toml(&self.dir.join("state.toml"), &self.state)
    }

    fn save_base(&mut self, snap: &Snapshot) -> Result<()> {
        self.state.base = format!("base-{}.json", snap.id);
        let json = serde_json::to_string_pretty(snap).expect("snapshot serializes");
        write_atomic(&self.dir.join(&self.state.base), json.as_bytes())?;
        self.save()
    }

    pub fn base(&self) -> Result<Snapshot> {
        Snapshot::load(&self.dir.join(&self.state.base))
    }

    pub fn enabled(&self) -> Vec<String> {
        let on = self
            .state
            .mods
            .iter()
            .filter(|m| !self.state.disabled.contains(*m));
        on.cloned().collect()
    }

    fn gen_path(&self, n: u32) -> PathBuf {
        self.dir.join(format!("history/{n:04}.toml"))
    }

    /// Every generation, oldest first.
    pub fn history(&self) -> Result<Vec<(u32, Generation)>> {
        let dir = self.dir.join("history");
        let mut out = Vec::new();
        for e in fs::read_dir(&dir).map_err(io(&dir))? {
            let p = e.map_err(io(&dir))?.path();
            let n = p.file_stem().and_then(|s| s.to_str()?.parse().ok());
            if let Some(n) = n.filter(|_| p.extension().is_some_and(|x| x == "toml")) {
                out.push((n, read_toml(&p)?));
            }
        }
        out.sort_by_key(|g| g.0);
        Ok(out)
    }

    /// The last complete generation.
    pub fn last(&self) -> Result<Option<Generation>> {
        let mut gens = self.history()?.into_iter().rev().map(|g| g.1);
        Ok(gens.find(|g| g.status == Status::Complete))
    }

    fn next(&self) -> Result<u32> {
        Ok(self.history()?.last().map_or(1, |g| g.0 + 1))
    }

    /// Fails if an interrupted apply left a generation in progress.
    pub fn refuse_pending(&self) -> Result<()> {
        match self
            .history()?
            .iter()
            .find(|g| g.1.status == Status::InProgress)
        {
            Some((n, _)) => Err(input(format!(
                "generation {n} was interrupted: run `fl-modman recover --finish` or `--rollback`"
            ))),
            None => Ok(()),
        }
    }

    pub fn refuse_running(&self) -> Result<()> {
        match HOST.game_running(&self.t.exe) {
            true => Err(input(format!(
                "{} is running; close the game first",
                self.t.exe
            ))),
            false => Ok(()),
        }
    }

    /// The position of `id` in the mod list.
    pub fn position(&self, id: &str) -> Result<usize> {
        let i = self.state.mods.iter().position(|m| m == id);
        i.ok_or_else(|| input(format!("unknown mod {id:?}")))
    }

    /// Deletes a disabled mod that the last complete generation does not use.
    pub fn remove(&mut self, id: &str) -> Result<()> {
        let i = self.position(id)?;
        if !self.state.disabled.contains(id)
            || self.last()?.is_some_and(|g| g.mods.iter().any(|m| m == id))
        {
            return Err(input(format!("disable {id} and apply first")));
        }
        self.state.mods.remove(i);
        self.state.disabled.remove(id);
        self.save()?;
        let d = self.dir.join("mods").join(id);
        fs::remove_dir_all(&d).map_err(io(&d))
    }
}
