//! Hashing the game tree (strictly read-only): the base snapshot, and the cached rescans that
//! apply and drift use. `hashes.json` keeps path → size, mtime, BLAKE3.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::adapters::fs::{b3_file, read_json, sha256_file, walk_files, write_atomic};
use crate::base::{FileHash, SCHEMA, Snapshot, TreeManifest, manifest_id, parse_version_file};
use crate::codec::DocText;
use crate::error::{Error, Result, input, io};
use crate::target::Target;

/// Path → (size, mtime in ns, BLAKE3): a file is rehashed only when its size or mtime moved.
pub type HashCache = BTreeMap<String, (u64, u64, String)>;

/// Hashes the file `rel` with metadata `m` unless `cache` holds it with the same size and
/// mtime, and records it in `fresh`.
fn hash_cached(
    rel: &str,
    path: &Path,
    m: &fs::Metadata,
    cache: &HashCache,
    fresh: &mut HashCache,
) -> Result<FileHash> {
    let since = m.modified().map(|t| t.duration_since(UNIX_EPOCH));
    let mtime = since.map_or(0, |d| d.map_or(0, |d| d.as_nanos() as u64));
    let b3 = match cache.get(rel) {
        Some((s, t, h)) if (*s, *t) == (m.len(), mtime) && mtime != 0 => h.clone(),
        _ => b3_file(path)?.0,
    };
    fresh.insert(rel.into(), (m.len(), mtime, b3.clone()));
    Ok(FileHash { b3, size: m.len() })
}

/// Hashes every file under `root` outside the user paths (read-only), reusing `cache`.
/// Returns the files and the cache entries of this scan.
pub fn scan_files(
    root: &Path,
    target: &Target,
    cache: &HashCache,
    skipped: &mut Vec<String>,
) -> Result<(TreeManifest, HashCache)> {
    let (mut files, mut fresh) = (TreeManifest::new(), HashCache::new());
    let mut visit = |rel: &str, path: &Path, m: &fs::Metadata| {
        if !target.is_user_path(rel) {
            files.insert(rel.into(), hash_cached(rel, path, m, cache, &mut fresh)?);
        }
        Ok(())
    };
    walk_files(root, "", &mut visit, skipped)?;
    Ok((files, fresh))
}

/// Like [`scan_files`] for the given `paths` only, without listing folders: a path that is
/// missing or not a regular file is absent from the result.
pub fn scan_paths<'a>(
    root: &Path,
    paths: impl IntoIterator<Item = &'a str>,
    cache: &HashCache,
) -> Result<(TreeManifest, HashCache)> {
    use std::io::ErrorKind::{NotADirectory, NotFound};
    let (mut files, mut fresh) = (TreeManifest::new(), HashCache::new());
    for rel in paths {
        let path = root.join(rel);
        match path.symlink_metadata() {
            Ok(m) if m.is_file() => {
                files.insert(rel.into(), hash_cached(rel, &path, &m, cache, &mut fresh)?);
            }
            Ok(_) => {}
            Err(e) if matches!(e.kind(), NotFound | NotADirectory) => {}
            Err(e) => return Err(io(&path)(e)),
        }
    }
    Ok((files, fresh))
}

/// Hashes the game tree (or only `paths`), reusing `<ws_dir>/hashes.json` for files whose size
/// and mtime did not move. The cache is rewritten only when it changed.
pub(super) fn cached(
    ws_dir: &Path,
    game: &Path,
    t: &Target,
    paths: Option<&BTreeSet<&str>>,
    skipped: &mut Vec<String>,
) -> Result<TreeManifest> {
    let path = ws_dir.join("hashes.json");
    let cache: HashCache = read_json(&path).ok().flatten().unwrap_or_default();
    let (files, fresh) = match paths {
        Some(paths) => scan_paths(game, paths.iter().copied(), &cache)?,
        None => scan_files(game, t, &cache, skipped)?,
    };
    if fresh != cache {
        let json = serde_json::to_vec(&fresh).expect("cache serializes");
        write_atomic(&path, &json)?;
    }
    Ok(files)
}

impl Snapshot {
    /// Scans `root` read-only: hashes every file outside the user paths and reads the
    /// managed documents and the identity files.
    pub fn scan(root: &Path, target: &Target) -> Result<Self> {
        let mut skipped = Vec::new();
        let files = scan_files(root, target, &HashCache::new(), &mut skipped)?.0;
        Self::from_files(root, target, files, skipped)
    }

    /// The snapshot of `root` given its hashed `files`.
    pub fn from_files(
        root: &Path,
        t: &Target,
        files: TreeManifest,
        skipped: Vec<String>,
    ) -> Result<Self> {
        let (schema, target) = (SCHEMA, t.id.clone());
        let mut s = Snapshot {
            schema,
            target,
            files,
            skipped,
            ..Default::default()
        };
        s.refresh(root, t, |_| true)?;
        Ok(s)
    }

    /// Re-reads from `root` the documents and identity files for which `only` holds (as
    /// listed in `files`) and recomputes the id.
    pub fn refresh(&mut self, root: &Path, t: &Target, only: impl Fn(&str) -> bool) -> Result<()> {
        let read = |rel: &str| {
            let p = root.join(rel);
            fs::read(&p).map_err(io(&p))
        };
        let has = |rel: &String| only(rel) && self.files.contains_key(rel);
        for doc in [&t.sider_ini, &t.map_teams].into_iter().filter(|d| only(d)) {
            match self.files.contains_key(doc) {
                true => self.docs.insert(doc.clone(), DocText::decode(&read(doc)?)),
                false => self.docs.remove(doc),
            };
        }
        if only(&t.exe) {
            let exe = has(&t.exe).then(|| sha256_file(&root.join(&t.exe)));
            self.exe_sha256 = exe.transpose()?;
        }
        if only(&t.version_file) {
            let v = has(&t.version_file)
                .then(|| read(&t.version_file))
                .transpose()?;
            self.version = v.and_then(|v| parse_version_file(&String::from_utf8_lossy(&v)));
        }
        self.id = manifest_id(&self.files);
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(io(path))?;
        let snap: Snapshot = serde_json::from_str(&text).map_err(|source| Error::Json {
            path: path.into(),
            source,
        })?;
        if snap.schema != SCHEMA {
            return Err(input(format!(
                "{}: unsupported snapshot schema {}",
                path.display(),
                snap.schema
            )));
        }
        Ok(snap)
    }
}
