//! A Sider Lua module → `@modules/`; every top-level `.lua` becomes a `lua.module` line.

use super::*;

pub struct Lua;

impl ModKind for Lua {
    fn detect(&self, top: &Listing) -> u8 {
        if top.files.iter().any(|f| ext(f) == "lua") { 40 } else { 0 }
    }

    fn contribute(&self, root: &Root) -> Contribution {
        let top = root.files.iter().filter(|f| !f.contains('/') && ext(f) == "lua");
        folder(root, "modules", "@modules/", |m| {
            m.lua_module = top.cloned().collect();
            Ok(())
        })
    }

    fn describe(&self) -> &'static str {
        "Lua"
    }
}
