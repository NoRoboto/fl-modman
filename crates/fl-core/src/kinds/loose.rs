//! Loose files in the game root (e.g. ReShade's `dxgi.dll`) → `@game/`, verbatim.

use super::*;

pub struct Loose;

impl ModKind for Loose {
    /// A `.dll`/`.asi`, or a file named like one in the game root.
    fn detect(&self, top: &Listing) -> u8 {
        let hook = |f: &String| matches!(ext(f).as_str(), "dll" | "asi");
        let root_file = |f: &String| base_name(top.base, "", f).is_some();
        if top.files.iter().any(|f| hook(f) || root_file(f)) { 10 } else { 0 }
    }

    fn contribute(&self, root: &Root) -> Contribution {
        folder(root, "game", "@game/", |_| Ok(()))
    }

    fn describe(&self) -> &'static str {
        "Loose"
    }
}
