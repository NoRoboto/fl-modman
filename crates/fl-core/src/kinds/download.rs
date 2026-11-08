//! A `download/*.cpk` replacement → `@download/`, with the base file's spelling (the base file
//! is the original kept for undo).

use super::*;

pub struct Download;

impl ModKind for Download {
    fn detect(&self, top: &Listing) -> u8 {
        let cpk = |f: &String| ext(f) == "cpk" && base_name(top.base, "download/", f).is_some();
        if top.files.iter().any(cpk) { 50 } else { 0 }
    }

    fn contribute(&self, root: &Root) -> Contribution {
        let mut c = folder(root, "download", "@download/", |_| Ok(()));
        for (file, dest) in &mut c.files {
            if let Some(name) = base_name(root.base, "download/", file) {
                *dest = format!("download/{name}");
            }
        }
        c
    }

    fn describe(&self) -> &'static str {
        "Download"
    }
}
