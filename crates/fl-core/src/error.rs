use std::path::PathBuf;

/// Input, manifest or I/O errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{path}: invalid TOML: {source}")]
    Toml {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("{path}: invalid JSON: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("invalid path {0:?}: {1}")]
    Path(String, &'static str),
    #[error("{0}")]
    Manifest(String),
    #[error("{0}")]
    Input(String),
    /// Files changed outside fl-modman since the last apply (grilling Q9).
    #[error(
        "{} file(s) changed outside fl-modman:\n  {}\nrun `fl-modman accept-drift` \
         (keep them as the new base) or `fl-modman restore`",
        .0.len(), .0[..(.0.len().min(50))].join("\n  ")
    )]
    Drift(Vec<String>),
    /// Import could not tell the mod root: the candidate roots and their types.
    #[error(
        "cannot tell the mod root (candidates: {}); re-run with --root <path in the archive>",
        .0.iter().map(|(r, k)| format!("{r} ({k:?})")).collect::<Vec<_>>().join(", ")
    )]
    AmbiguousRoot(Vec<(String, crate::kinds::Kind)>),
    /// Import of a stadium without a team.
    #[error("a stadium needs --team <id>[,<slot>]")]
    NeedsTeam,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub fn input(msg: impl Into<String>) -> Error {
    Error::Input(msg.into())
}

pub(crate) fn io(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> Error {
    let path = path.into();
    move |source| Error::Io { path, source }
}
