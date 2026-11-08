//! The pure model: paths, the target, tree manifests and the base snapshot, `mod.toml`
//! packages, build and plan. Nothing here reads the disk or looks at other processes.

pub mod base;
pub mod build;
pub mod hash;
pub mod manifest;
pub mod paths;
pub mod plan;
pub mod target;
