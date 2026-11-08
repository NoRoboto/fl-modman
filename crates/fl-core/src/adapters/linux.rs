//! The Linux host: `/proc` for the running game, in-process zip/7z and the system's rar tools.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::error::{Result, input, io};
use crate::paths::ext;
use crate::ports::Host;

pub struct Linux;

impl Host for Linux {
    /// A process is the game if its name, or the file name of its argv[0] (Wine sets it to the
    /// Windows path), is the exe. Tools that only mention the exe in arguments do not count.
    fn game_running(&self, exe: &str) -> bool {
        let mut procs = fs::read_dir("/proc").into_iter().flatten().flatten();
        procs.any(|p| {
            let comm = fs::read_to_string(p.path().join("comm")).unwrap_or_default();
            let cmd = fs::read(p.path().join("cmdline")).unwrap_or_default();
            let argv0 = String::from_utf8_lossy(cmd.split(|b| *b == 0).next().unwrap_or_default());
            let name = argv0.rsplit(['/', '\\']).next().unwrap_or_default();
            comm.trim().eq_ignore_ascii_case(exe) || name.eq_ignore_ascii_case(exe)
        })
    }

    /// Extracts into a fresh `to`, then rejects the result (and deletes it) if it holds a
    /// symlink or an entry that resolves outside `to`, before anything else reads it.
    fn extract(&self, src: &Path, to: &Path) -> Result<()> {
        let _ = fs::remove_dir_all(to);
        fs::create_dir_all(to).map_err(io(to))?;
        let res = unpack(src, to).and_then(|()| check_extracted(to));
        if res.is_err() {
            let _ = fs::remove_dir_all(to);
        }
        res
    }
}

/// zip and 7z in process; anything else (rar) with the first of `7z`, `7zz`, `unrar`, `bsdtar`
/// on the system that succeeds. 7-Zip skips symlink entries (`-snl-`).
fn unpack(src: &Path, to: &Path) -> Result<()> {
    let fail = |e: String| input(format!("{}: {e}", src.display()));
    match ext(&src.to_string_lossy()).as_str() {
        "zip" => {
            let file = fs::File::open(src).map_err(io(src))?;
            let zip = zip::ZipArchive::new(file).and_then(|mut z| z.extract(to));
            return zip.map_err(|e| fail(e.to_string()));
        }
        "7z" => {
            return sevenz_rust2::decompress_file(src, to).map_err(|e| fail(e.to_string()));
        }
        _ => {}
    }
    let tools: [&[&str]; 4] = [
        &["7z", "x", "-y", "-snl-", "-o@D", "--", "@S"],
        &["7zz", "x", "-y", "-snl-", "-o@D", "--", "@S"],
        &["unrar", "x", "-y", "-o+", "--", "@S", "@D/"],
        &["bsdtar", "-xf", "@S", "-C", "@D"],
    ];
    let (s, d) = (src.to_string_lossy(), to.to_string_lossy());
    let mut errors = Vec::new();
    for tool in tools {
        let _ = fs::remove_dir_all(to);
        fs::create_dir_all(to).map_err(io(to))?;
        // One pass, so a path that contains "@D" is never substituted twice.
        let args = tool[1..].iter().map(|a| {
            let mut out = String::new();
            let mut rest = *a;
            while let Some(i) = rest.find('@') {
                out.push_str(&rest[..i]);
                let (sub, n) = match &rest[i..].get(..2) {
                    Some("@S") => (s.as_ref(), 2),
                    Some("@D") => (d.as_ref(), 2),
                    _ => ("@", 1),
                };
                out.push_str(sub);
                rest = &rest[i + n..];
            }
            out + rest
        });
        match Command::new(tool[0]).args(args).output() {
            Ok(o) if o.status.success() => return Ok(()),
            Ok(o) => errors.push(format!(
                "{}: {}",
                tool[0],
                String::from_utf8_lossy(&o.stderr).trim()
            )),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => errors.push(format!("{}: {e}", tool[0])),
        }
    }
    if errors.is_empty() {
        errors.push("no extractor: install 7-Zip with RAR support (7z), unrar or bsdtar".into());
    }
    Err(fail(errors.join("; ")))
}

/// Rejects a tree that holds a symlink or an entry whose canonical path is outside `root`.
fn check_extracted(root: &Path) -> Result<()> {
    let base = root.canonicalize().map_err(io(root))?;
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for e in fs::read_dir(&dir).map_err(io(&dir))? {
            let p = e.map_err(io(&dir))?.path();
            let m = fs::symlink_metadata(&p).map_err(io(&p))?;
            let inside = || p.canonicalize().is_ok_and(|c| c.starts_with(&base));
            if m.file_type().is_symlink() || !inside() {
                let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
                return Err(input(format!(
                    "archive entry {rel} is a link or points outside the archive; refused"
                )));
            }
            if m.is_dir() {
                dirs.push(p);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_post_extract_check_rejects_links_and_accepts_plain_trees() {
        let root = std::env::temp_dir().join(format!("fl-check-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("a")).unwrap();
        fs::write(root.join("a/b.txt"), "x").unwrap();
        check_extracted(&root).unwrap();
        std::os::unix::fs::symlink("../..", root.join("a/up")).unwrap();
        let err = check_extracted(&root).unwrap_err().to_string();
        assert!(err.contains("a/up"), "{err}");
        fs::remove_dir_all(&root).unwrap();
    }
}
