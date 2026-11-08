//! The mod types (grilling Q7) as compile-time plugins: each file holds one [`ModKind`] that
//! recognises an extracted mod root and lays it out as a package. A new type is one more file
//! here plus its entry in the `registry!` line; nothing is loaded at runtime.

use std::{fmt, str::FromStr};

use crate::base::Snapshot;
use crate::error::{Error, Result, input};
use crate::manifest::{Manifest, Put};
use crate::paths::ext;
use crate::target::Target;

/// A candidate root as detection sees it: its top-level entries, sorted, without readme, image
/// and link files or `__MACOSX`.
pub struct Listing<'a> {
    pub files: Vec<String>,
    pub dirs: Vec<String>,
    pub base: &'a Snapshot,
    pub t: &'a Target,
}

/// The mod root being imported.
pub struct Root<'a> {
    /// Its folder name (the archive stem when the root is the archive's top).
    pub name: &'a str,
    /// Every payload file, relative to the root, in walk order.
    pub files: &'a [String],
    pub base: &'a Snapshot,
    pub t: &'a Target,
    /// `--team <id>[,<slot>]`, for stadiums.
    pub team: Option<&'a str>,
}

/// How a kind lays a root out in `mods/<id>/`.
pub struct Contribution {
    /// (file relative to the root, path in the package).
    pub files: Vec<(String, String)>,
    /// The generated `mod.toml`, or why there is none (a stadium without a team).
    pub manifest: Result<Manifest>,
}

pub trait ModKind: Sync {
    /// How much a candidate root looks like this kind: 0 = not at all. The highest wins; a tie
    /// goes to the earlier kind in the registry.
    fn detect(&self, top: &Listing) -> u8;
    /// The payload layout, the puts and the document entries (`cpk_root`, `lua_module`,
    /// `map_teams`) that build reads.
    fn contribute(&self, root: &Root) -> Contribution;
    /// The label the CLI and GUI show; `--type` takes it in any case.
    fn describe(&self) -> &'static str;
}

pub type Kind = &'static dyn ModKind;

macro_rules! registry {
    ($($file:ident::$kind:ident),*) => {
        $(mod $file;)*
        /// Every kind, in tie-break order.
        pub static KINDS: &[Kind] = &[$(&$file::$kind),*];
    };
}
registry!(
    livecpk::Livecpk,
    lua::Lua,
    stadium::Stadium,
    download::Download,
    loose::Loose
);

/// The kind a candidate root looks most like, if any.
pub fn detect(top: &Listing) -> Option<Kind> {
    let scored = KINDS
        .iter()
        .map(|k| (k.detect(top), *k))
        .filter(|s| s.0 > 0);
    scored.rev().max_by_key(|s| s.0).map(|s| s.1)
}

impl fmt::Debug for dyn ModKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.describe())
    }
}

impl FromStr for Kind {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let names: Vec<String> = KINDS.iter().map(|k| k.describe().to_lowercase()).collect();
        let k = KINDS.iter().find(|k| k.describe().eq_ignore_ascii_case(s));
        k.copied()
            .ok_or_else(|| input(format!("unknown type {s:?} ({})", names.join("|"))))
    }
}

/// Readme, image and link files: ignored at the mod root.
pub const JUNK: [&str; 13] = [
    "txt", "md", "pdf", "rtf", "doc", "nfo", "url", "htm", "html", "jpg", "jpeg", "png", "bmp",
];

/// The real name of the base file `<pre><name>`, matched case-insensitively.
fn base_name<'a>(base: &'a Snapshot, pre: &str, name: &str) -> Option<&'a str> {
    let mut names = base.files.keys().filter_map(|k| k.strip_prefix(pre));
    names.find(|n| n.eq_ignore_ascii_case(name))
}

/// Every root file under `dir/` in the package, put as one folder to `to`; `docs` adds the
/// document entries.
fn folder(
    root: &Root,
    dir: &str,
    to: &str,
    docs: impl FnOnce(&mut Manifest) -> Result<()>,
) -> Contribution {
    let files = root.files.iter().map(|f| (f.clone(), format!("{dir}/{f}")));
    let (name, from, to) = (Some(root.name.into()), format!("{dir}/"), to.into());
    let put = vec![Put { from, to }];
    let mut m = Manifest {
        name,
        put,
        ..Default::default()
    };
    let manifest = docs(&mut m).map(|()| m);
    let files = files.collect();
    Contribution { files, manifest }
}
