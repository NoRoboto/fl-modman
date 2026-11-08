//! A livecpk folder (`common/`, `Asset/`, language folders) → `@livecpk/<name>/` + `cpk.root`.

use super::*;

pub struct Livecpk;

impl ModKind for Livecpk {
    fn detect(&self, top: &Listing) -> u8 {
        let livecpk = |d: &String| top.t.livecpk_dirs.iter().any(|l| l.eq_ignore_ascii_case(d));
        if top.dirs.iter().any(livecpk) { 20 } else { 0 }
    }

    fn contribute(&self, root: &Root) -> Contribution {
        let name = root.name;
        folder(root, name, &format!("@livecpk/{name}/"), |m| {
            m.cpk_root = vec![format!(".\\livecpk\\{name}")];
            Ok(())
        })
    }

    fn describe(&self) -> &'static str {
        "Livecpk"
    }
}
