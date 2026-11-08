//! The host OS behind `ports`: the filesystem edge, game discovery and the Linux [`Host`].

pub mod detect;
pub mod fs;
mod linux;

use crate::ports::Host;

/// The host this build runs on.
pub static HOST: &dyn Host = &linux::Linux;
