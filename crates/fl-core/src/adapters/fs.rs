//! Filesystem edge: TOML/JSON files, atomic writes, hardlink-or-copy placement, the sorted
//! no-symlink walk and file hashing. `std::fs` already is the cross-platform layer, so this is
//! plain functions, not a port.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

use crate::error::{Error, Result, io};

pub fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).map_err(io(path))?;
    toml::from_str(&text).map_err(|source| Error::Toml {
        path: path.into(),
        source,
    })
}

pub fn write_toml<T: Serialize>(path: &Path, v: &T) -> Result<()> {
    write_atomic(path, toml::to_string(v).expect("serializes").as_bytes())
}

/// The JSON file at `path`; `None` if it does not exist.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    match fs::read(path) {
        Ok(b) => serde_json::from_slice(&b)
            .map(Some)
            .map_err(|source| Error::Json {
                path: path.into(),
                source,
            }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io(path)(e)),
    }
}

/// Creates `dst`'s folder and returns the temp path that is renamed over `dst`: files are never
/// written in place (the LAB instance shares hardlinked files with the stable install).
fn tmp(dst: &Path) -> Result<PathBuf> {
    let dir = dst.parent().expect("file path");
    fs::create_dir_all(dir).map_err(io(dir))?;
    let mut s = dst.as_os_str().to_owned();
    s.push(".flmm-tmp");
    Ok(s.into())
}

pub fn write_atomic(dst: &Path, bytes: &[u8]) -> Result<()> {
    let t = tmp(dst)?;
    fs::write(&t, bytes)
        .and_then(|()| fs::rename(&t, dst))
        .map_err(io(dst))
}

/// Hardlinks (if `link` and on one filesystem) or copies `src` to `dst`; never a symlink.
pub fn place(src: &Path, dst: &Path, link: bool) -> Result<()> {
    let t = tmp(dst)?;
    let _ = fs::remove_file(&t);
    if !(link && fs::hard_link(src, &t).is_ok()) {
        fs::copy(src, &t).map_err(io(src))?;
    }
    fs::rename(&t, dst).map_err(io(dst))
}

/// (device, inode) of a file, from metadata read without following symlinks: two paths with the
/// same id are hardlinks to one file.
pub fn file_id(m: &fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (m.dev(), m.ino())
}

/// Visits every regular file under `root/rel` in sorted order without following symlinks.
/// Non-regular entries (symlinks, devices) are recorded in `skipped`.
pub fn walk_files(
    root: &Path,
    rel: &str,
    visit: &mut dyn FnMut(&str, &Path, &fs::Metadata) -> Result<()>,
    skipped: &mut Vec<String>,
) -> Result<()> {
    let dir = root.join(rel);
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .map_err(io(&dir))?
        .collect::<io::Result<_>>()
        .map_err(io(&dir))?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| Error::Input(format!("non-UTF-8 file name in {}", dir.display())))?;
        let child = crate::paths::join(rel, name);
        let path = entry.path();
        let meta = path.symlink_metadata().map_err(io(&path))?;
        if meta.is_dir() {
            walk_files(root, &child, visit, skipped)?;
        } else if meta.is_file() {
            visit(&child, &path, &meta)?;
        } else {
            skipped.push(child);
        }
    }
    Ok(())
}

/// Hashes a file with BLAKE3, streaming. Returns `(hex, size)`.
pub fn b3_file(path: &Path) -> Result<(String, u64)> {
    let mut file = File::open(path).map_err(io(path))?;
    let mut hasher = blake3::Hasher::new();
    let size = io::copy(&mut file, &mut hasher).map_err(io(path))?;
    Ok((hasher.finalize().to_hex().to_string(), size))
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(io(path))?;
    let (mut hasher, mut buf) = (Sha256::new(), vec![0u8; 1 << 16]);
    loop {
        match file.read(&mut buf).map_err(io(path))? {
            0 => break,
            n => hasher.update(&buf[..n]),
        }
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
