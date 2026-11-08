//! `fl-modman` command line: the engine behind the GUI, and a dev/test tool (grilling Q3).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use fl_core::detect::OVERRIDES;
use fl_core::error::input;
use fl_core::import::{Options, import};
use fl_core::kinds::Kind;
use fl_core::plan::{Plan, human_size};
use fl_core::usage::{ModUsage, Usage};
use fl_core::workspace::Workspace;
use fl_core::{Result, detect, graphics, hash, settings};

#[derive(Parser)]
#[command(
    name = "fl-modman",
    version,
    about = "Mod manager for SP Football Life 2027 (Linux)"
)]
struct Cli {
    /// Workspace folder [default: $FL_MODMAN; for init: <game-dir>/../fl-modman]
    #[arg(short, long, global = true)]
    workspace: Option<PathBuf>,
    /// Write the changes (apply, rollback, remove --disabled); without it only the plan is shown.
    #[arg(long, global = true)]
    yes: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List FL27 installs and Wine prefixes found in the usual places.
    Detect,
    /// Create the workspace and take the base snapshot of the game as it is.
    Init {
        game_dir: PathBuf,
        #[arg(long)]
        prefix: Option<PathBuf>,
    },
    /// Import a mod archive (zip, 7z, rar) or folder, enabled at the bottom (it wins).
    Import {
        src: PathBuf,
        #[arg(long = "type")]
        kind: Option<Kind>,
        /// Stadium team: <id>[,<slot>]
        #[arg(long)]
        team: Option<String>,
        /// Mod root, relative to the archive.
        #[arg(long)]
        root: Option<String>,
        /// Use this mod.toml (e.g. catalog/<id>/mod.toml) instead of generating one.
        #[arg(long)]
        manifest: Option<PathBuf>,
    },
    /// Show the mod list (the lower mod wins).
    List,
    Enable {
        id: String,
    },
    Disable {
        id: String,
    },
    /// Move a mod to a position in the list.
    Move {
        id: String,
        index: usize,
    },
    /// Delete a disabled, unapplied mod; --disabled: all of them and staging/ leftovers (--yes).
    Remove {
        #[arg(required_unless_present = "disabled", conflicts_with = "disabled")]
        id: Option<String>,
        /// Every disabled mod no longer applied, plus staging/ leftovers; lists them without --yes.
        #[arg(long)]
        disabled: bool,
    },
    /// Show the disk usage of the workspace per mod, and what `remove --disabled` frees.
    Usage,
    /// Show the plan; with --yes apply it as a new generation.
    Apply,
    History,
    /// Re-apply generation N (with --yes) and make its mods the list.
    Rollback {
        n: u32,
    },
    /// Finish or roll back an interrupted apply.
    Recover {
        #[arg(
            long,
            required_unless_present = "rollback",
            conflicts_with = "rollback"
        )]
        finish: bool,
        #[arg(long)]
        rollback: bool,
    },
    /// Take files changed outside fl-modman as the new base (apply re-applies the mods).
    AcceptDrift,
    /// Put back what fl-modman placed (the last complete generation).
    Restore,
    /// `settings get` or `settings set key=value ...` (settings.dat and the rendering options).
    Settings {
        #[arg(value_parser = ["get", "set"])]
        action: String,
        pairs: Vec<String>,
    },
    /// Rehash every mod payload and report files that differ from its index (read-only).
    Check,
    /// Check the Proton launch options FL27 needs.
    Doctor,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    fl_core::lap("cli");
    let dir = cli
        .workspace
        .or_else(|| std::env::var_os("FL_MODMAN").map(PathBuf::from));
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let (cmd, mut ws) = match (cli.cmd, dir) {
        (Cmd::Detect, _) => return detect_cmd(&home),
        (Cmd::Doctor, _) => return doctor(&home),
        (Cmd::Init { game_dir, prefix }, dir) => return init(&game_dir, prefix, dir),
        (_, None) => return Err(input("pass --workspace <dir> or set FL_MODMAN")),
        (cmd, Some(dir)) => (cmd, Workspace::open(&dir)?),
    };
    if let Cmd::Recover { finish, .. } = cmd {
        let n = ws.recover(finish)?;
        println!(
            "generation {n} {}",
            if finish { "finished" } else { "rolled back" }
        );
        return Ok(());
    }
    ws.refuse_pending()?;
    match cmd {
        Cmd::Import {
            src,
            kind,
            team,
            root,
            manifest,
        } => {
            let (team, root, manifest) = (team.as_deref(), root.as_deref(), manifest.as_deref());
            let opt = Options {
                kind,
                team,
                root,
                manifest,
            };
            let m = import(&mut ws, &src, opt)?;
            let (id, kind, root, n) = (&m.id, m.kind, &m.root, m.files.len());
            println!("imported {id} ({kind:?}, root {root}): {n} file(s), enabled last");
            m.files.iter().take(10).for_each(|f| println!("  + {f}"));
            for c in &m.conflicts {
                println!("  ! {c} (e.g. {})", c.items[0]);
            }
        }
        Cmd::List => {
            for (i, m) in ws.state.mods.iter().enumerate() {
                let on = if ws.state.disabled.contains(m) {
                    " "
                } else {
                    "x"
                };
                println!("{i:3}  [{on}] {m}");
            }
        }
        Cmd::Enable { id } => {
            ws.position(&id)?;
            ws.state.disabled.remove(&id);
            ws.save()?;
        }
        Cmd::Disable { id } => {
            ws.position(&id)?;
            ws.state.disabled.insert(id);
            ws.save()?;
        }
        Cmd::Move { id, index } => {
            let m = ws.state.mods.remove(ws.position(&id)?);
            ws.state.mods.insert(index.min(ws.state.mods.len()), m);
            ws.save()?;
        }
        Cmd::Remove { id: Some(id), .. } => ws.remove(&id)?,
        Cmd::Remove { id: None, .. } => remove_disabled(&mut ws, cli.yes)?,
        Cmd::Usage => print!("{}", usage_text(&ws.usage()?)),
        Cmd::Apply => report(ws.apply(&ws.enabled(), "apply", cli.yes, true)?),
        Cmd::Rollback { n } => report(ws.rollback(n, cli.yes)?),
        Cmd::Restore => report(ws.restore()?),
        Cmd::AcceptDrift => {
            let drift = ws.accept_drift()?;
            drift.iter().for_each(|p| println!("  accepted {p}"));
            println!(
                "{} file(s) taken into the base; apply re-applies the mods",
                drift.len()
            );
        }
        Cmd::Check => {
            let (n, bad) = ws.check_mods()?;
            bad.iter().for_each(|b| println!("  ! {b}"));
            let m = ws.state.mods.len();
            println!("{m} mod(s), {n} file(s) hashed, {} mismatch(es)", bad.len());
            if !bad.is_empty() {
                return Err(input(
                    "mod payloads differ from their index; re-import them",
                ));
            }
        }
        Cmd::History => {
            for (n, g) in ws.history()? {
                println!(
                    "{n:04}  {:?}  {}  [{}]",
                    g.status,
                    g.note,
                    g.mods.join(", ")
                );
            }
        }
        Cmd::Settings { action, pairs } => {
            let prefix = ws
                .state
                .prefix
                .as_deref()
                .ok_or_else(|| input("init had no --prefix"))?;
            let docs = settings::fl_docs(prefix)
                .ok_or_else(|| input("no SPGAME/Football Life in the prefix"))?;
            if action == "get" {
                let b = read(&docs.join("settings.dat"))?;
                settings::check(&b)?;
                let mut kv = settings::get(&b);
                let gfx = docs.join(graphics::FILE);
                match gfx.exists() {
                    true => {
                        let g = read(&gfx)?;
                        graphics::check(&g)?;
                        kv.extend(graphics::get(&g));
                    }
                    false => eprintln!("no {}: {}", gfx.display(), settings::GFX_HINT),
                }
                kv.iter().for_each(|(k, v)| println!("{k}={v}"));
            } else {
                ws.refuse_running()?;
                for bak in settings::update(&docs, &ws.dir, &pairs)? {
                    println!("updated (backup {})", bak.display());
                }
            }
        }
        Cmd::Detect | Cmd::Doctor | Cmd::Init { .. } | Cmd::Recover { .. } => unreachable!(),
    }
    Ok(())
}

/// The usage table, totals, reclaimable and the filesystem warning.
fn usage_text(u: &Usage) -> String {
    let mut s = format!(
        "{:<32} {:<9} {:>7} {:>10} {:>10} {:>10}\n",
        "mod", "state", "files", "size", "shared", "only here"
    );
    for m in &u.mods {
        let state = match (m.enabled, m.applied) {
            (true, true) => "applied",
            (true, false) => "pending",
            (false, true) => "disabled*",
            (false, false) => "disabled",
        };
        s += &format!(
            "{:<32} {state:<9} {:>7} {:>10} {:>10} {:>10}\n",
            m.id,
            m.files,
            human_size(m.bytes),
            human_size(m.shared),
            human_size(m.only_here())
        );
    }
    if u.mods.iter().any(|m| !m.enabled && m.applied) {
        s += "disabled* = still in the game until the next apply\n";
    }
    let only: u64 = u.mods.iter().map(ModUsage::only_here).sum();
    s += &format!(
        "\nmods/ {} ({} only in the workspace)   originals/ {}   history/ {}   staging/ {}\n\
         workspace total {}\nreclaimable {} (remove --disabled)\n",
        human_size(u.mods_dir),
        human_size(only),
        human_size(u.originals),
        human_size(u.history),
        human_size(u.staging),
        human_size(u.total),
        human_size(u.reclaimable()),
    );
    if !u.same_fs {
        s += "warning: the workspace and the game are on different filesystems, so apply copies \
              files instead of hardlinking them: every applied byte is stored twice\n";
    }
    s
}

fn remove_disabled(ws: &mut Workspace, yes: bool) -> Result<()> {
    let u = ws.usage()?;
    let gone: Vec<&ModUsage> = u.mods.iter().filter(|m| m.removable()).collect();
    for m in &gone {
        println!("  - {} ({})", m.id, human_size(m.bytes));
    }
    if u.staging > 0 {
        println!("  - staging/ leftovers ({})", human_size(u.staging));
    }
    let held = u.mods.iter().filter(|m| !m.enabled && m.applied);
    for m in held {
        println!("  kept {}: still applied, apply first", m.id);
    }
    let size = human_size(u.reclaimable());
    if !yes {
        println!("would free {size}; re-run with --yes to delete");
        return Ok(());
    }
    let n = ws.remove_disabled()?.len();
    println!("removed {n} mod(s), freed {size}");
    Ok(())
}

fn read(file: &Path) -> Result<Vec<u8>> {
    fs::read(file).map_err(|e| input(format!("{}: {e}", file.display())))
}

fn init(game_dir: &Path, prefix: Option<PathBuf>, dir: Option<PathBuf>) -> Result<()> {
    let game =
        fs::canonicalize(game_dir).map_err(|e| input(format!("{}: {e}", game_dir.display())))?;
    let dir = dir.unwrap_or_else(|| game.parent().unwrap_or(&game).join("fl-modman"));
    let ws = Workspace::init(&dir, &game, prefix)?;
    let b = ws.base()?;
    let (n, size) = (
        b.files.len(),
        human_size(b.files.values().map(|f| f.size).sum()),
    );
    let exe = b.exe_sha256.as_deref().map_or("missing", hash::short);
    println!(
        "workspace {}\nbase {}: {n} files, {size}, exe {exe}",
        ws.dir.display(),
        b.id
    );
    Ok(())
}

fn report((plan, n): (Plan, Option<u32>)) {
    print!("{}", plan.render_text());
    match n {
        Some(n) => println!("applied as generation {n}"),
        None => println!("nothing written; re-run with --yes to apply"),
    }
}

fn detect_cmd(home: &Path) -> Result<()> {
    let found = detect::detect(home, "FL_2027.exe");
    for (g, p) in &found.games {
        let p = p.as_ref().map_or("-".into(), |p| p.display().to_string());
        println!("game   {}   (inside prefix: {p})", g.display());
    }
    for p in &found.prefixes {
        println!("prefix {}   (has SPGAME/Football Life)", p.display());
    }
    println!("pick one: fl-modman init <game-dir> --prefix <prefix>");
    Ok(())
}

/// Reports whether a Steam config holds the overrides; never edits Steam files (Q18).
fn doctor(home: &Path) -> Result<()> {
    match detect::overrides_set(home) {
        Some(f) => println!("ok: the overrides are set in {}", f.display()),
        None => println!("the overrides are not in any Steam shortcuts.vdf or localconfig.vdf"),
    }
    println!(
        "\nFL27 under Proton needs these launch options (Steam > the game > Properties):\n\n  \
         WINEDLLOVERRIDES=\"{OVERRIDES}\" %command%\n\n\
         Without them the game stops with \"Unable to create interface ISteamUser\": Proton's\n\
         lsteamclient bridge must be off so the game's own steamclient64/steam_api64 load, and\n\
         winmm=n,b loads Sider. fl-modman never edits Steam files: paste the line yourself."
    );
    Ok(())
}
