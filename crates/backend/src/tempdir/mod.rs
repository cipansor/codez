//! Port of `gitea.dev/modules/tempdir` — TempDir helpers.
//!
//! | upstream | here |
//! | --- | --- |
//! | `tempdir.go` | this module |

use crate::util::file_retry::{remove_all_with_retry, remove_with_retry};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// `tempdir.TempDir` — a base directory plus a subdirectory beneath it.
pub struct TempDir {
    /// `base` must exist; it is never created automatically.
    base: PathBuf,
    /// `sub` may be multi-level and is created on demand.
    sub: PathBuf,
}

impl TempDir {
    /// `tempdir.New`.
    pub fn new(base: impl Into<PathBuf>, sub: impl Into<PathBuf>) -> Self {
        Self {
            base: base.into(),
            sub: sub.into(),
        }
    }

    /// `tempdir.OsTempDir` — a `TempDir` rooted at the system temp directory.
    pub fn os_temp_dir(sub: impl Into<PathBuf>) -> Self {
        Self::new(std::env::temp_dir(), sub)
    }

    /// `TempDir.JoinPath` — joins `elems` under `base/sub`.
    ///
    /// Upstream joins `elems` first and then appends that single component, so
    /// an absolute `elem` does not escape `base/sub`; the same order is kept.
    pub fn join_path(&self, elems: &[&str]) -> PathBuf {
        let mut out = self.base.join(&self.sub);
        for e in elems.iter().filter(|e| !e.is_empty()) {
            out.push(e);
        }
        out
    }

    /// `TempDir.MkdirAllSub` — like `mkdir -p`, but `base` must already exist.
    pub fn mkdir_all_sub(&self, dir: &str) -> std::io::Result<PathBuf> {
        if !self.base.exists() {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
        }
        let full = self.base.join(&self.sub).join(dir);
        std::fs::create_dir_all(&full)?;
        Ok(full)
    }

    /// `TempDir.prepareDirWithPattern` — resolves the parent directory and the
    /// trailing name prefix used as a temp-name pattern.
    fn prepare_dir_with_pattern(&self, elems: &[&str]) -> std::io::Result<(PathBuf, String)> {
        if !self.base.exists() {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
        }
        let mut full = self.base.join(&self.sub);
        for e in elems {
            full.push(e);
        }
        let pattern = full
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let dir = full
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.base.join(&self.sub));
        std::fs::create_dir_all(&dir)?;
        Ok((dir, pattern))
    }

    /// `TempDir.MkdirTempRandom` — like `mkdtemp`, with the last path element as
    /// the name pattern. Returns the directory and a cleanup closure.
    pub fn mkdir_temp_random(&self, elems: &[&str]) -> std::io::Result<(PathBuf, Box<dyn Fn()>)> {
        let (dir, pattern) = self.prepare_dir_with_pattern(elems)?;
        let created = create_temp_dir(&dir, &pattern)?;
        let cleanup_path = created.clone();
        Ok((
            created,
            Box::new(move || {
                if let Err(e) = remove_all_with_retry(&cleanup_path) {
                    eprintln!(
                        "Failed to remove temp directory {}: {e}",
                        cleanup_path.display()
                    );
                }
            }),
        ))
    }

    /// `TempDir.CreateTempFileRandom` — like `mkstemp`, with the last path
    /// element as the name pattern. Returns the file and a cleanup closure.
    pub fn create_temp_file_random(
        &self,
        elems: &[&str],
    ) -> std::io::Result<(TempFile, Box<dyn Fn()>)> {
        let (dir, pattern) = self.prepare_dir_with_pattern(elems)?;
        let (file, path) = create_temp_file(&dir, &pattern)?;
        let cleanup_path = path.clone();
        let cleanup = Box::new(move || {
            if let Err(e) = remove_with_retry(&cleanup_path) {
                eprintln!(
                    "Unable to remove temporary file: {}: Error: {e}",
                    cleanup_path.display()
                );
            }
        });
        Ok((TempFile { file, path }, cleanup))
    }

    /// `TempDir.RemoveOutdated` — removes files older than `d`, then prunes the
    /// now-empty directories, deepest first.
    pub fn remove_outdated(&self, d: Duration) {
        let root = self.join_path(&[]);
        remove_outdated_inner(&root, d);
    }
}

/// The handle returned by [`TempDir::create_temp_file_random`].
///
/// Go's `*os.File` exposes `Name()`; Rust's `File` does not, so the path is
/// carried alongside the handle.
pub struct TempFile {
    file: std::fs::File,
    path: PathBuf,
}

impl TempFile {
    /// `(*os.File).Name` — the file's path.
    pub fn name(&self) -> &Path {
        &self.path
    }

    /// The open handle, for reading and writing.
    pub fn file(&self) -> &std::fs::File {
        &self.file
    }
}

/// Creates a uniquely-named directory under `dir` whose name starts with `pattern`.
fn create_temp_dir(dir: &Path, pattern: &str) -> std::io::Result<PathBuf> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    for _ in 0..1000 {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate = dir.join(format!("{pattern}{n}"));
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::other(
        "unable to create a unique temp directory",
    ))
}

/// Creates a uniquely-named file under `dir` whose name starts with `pattern`.
fn create_temp_file(dir: &Path, pattern: &str) -> std::io::Result<(std::fs::File, PathBuf)> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    for _ in 0..1000 {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate = dir.join(format!("{pattern}{n}"));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(f) => return Ok((f, candidate)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::other("unable to create a unique temp file"))
}

/// Recursive worker for [`TempDir::remove_outdated`].
fn remove_outdated_inner(path: &Path, d: Duration) {
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let full = entry.path();
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            remove_outdated_inner(&full, d);
            // The directory is empty once its outdated children are gone.
            let _ = std::fs::remove_dir(&full);
            continue;
        }
        let old = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|mtime| SystemTime::now().duration_since(mtime).ok())
            .map(|age| age > d)
            .unwrap_or(false);
        if old {
            let _ = std::fs::remove_file(&full);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mirrors Go's `t.TempDir()`: a fresh directory removed when the guard drops.
    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "codez-tempdir-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            Self(p)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    // Transcribed from `TestTempDir` in modules/tempdir/tempdir_test.go.
    #[test]
    fn temp_dir_matches_upstream() {
        let base = TestDir::new();
        let base_str = base.path().to_string_lossy().to_string();

        // Create: the sub directory may contain "/".
        let td = TempDir::new(&base_str, "sub1/sub2");
        assert_eq!(td.join_path(&[]), base.path().join("sub1/sub2"));
        assert_eq!(td.join_path(&["test"]), base.path().join("sub1/sub2/test"));

        // MkdirTempRandom.
        let (s, cleanup) = td.mkdir_temp_random(&["foo"]).unwrap();
        // Upstream uses strings.HasPrefix, so compare text rather than path
        // components: "foo0" is a prefix of the name but not a path component.
        let want = base
            .path()
            .join("sub1/sub2/foo")
            .to_string_lossy()
            .to_string();
        assert!(
            s.to_string_lossy().starts_with(&want),
            "got {}",
            s.display()
        );
        assert!(s.exists());
        cleanup();
        assert!(!s.exists(), "cleanup must remove the directory");

        // CreateTempFileRandom.
        let (f, cleanup) = td.create_temp_file_random(&["foo", "bar"]).unwrap();
        let filename = f.name().to_path_buf();
        let want = base
            .path()
            .join("sub1/sub2/foo/bar")
            .to_string_lossy()
            .to_string();
        assert!(
            filename.to_string_lossy().starts_with(&want),
            "got {}",
            filename.display()
        );
        assert!(filename.exists());
        cleanup();
        assert!(!filename.exists(), "cleanup must remove the file");

        // RemoveOutdated: f2 and dir-b's file are aged an hour, f1 is fresh.
        let (fa1, _ca1) = td.create_temp_file_random(&["dir-a", "f1"]).unwrap();
        let (fa2, _ca2) = td.create_temp_file_random(&["dir-a", "f2"]).unwrap();
        let (fb1, _cb1) = td.create_temp_file_random(&["dir-b", "f1"]).unwrap();

        let aged = SystemTime::now() - Duration::from_secs(3600);
        for path in [fa2.name(), fb1.name()] {
            let f = std::fs::File::options().write(true).open(path).unwrap();
            f.set_times(std::fs::FileTimes::new().set_modified(aged))
                .unwrap();
        }

        td.remove_outdated(Duration::from_secs(60));

        assert!(fa1.name().exists(), "the fresh file must survive");
        assert!(!fa2.name().exists(), "the aged file must be removed");
        assert!(
            !fb1.name().exists(),
            "the aged file in dir-b must be removed"
        );
    }

    // Transcribed from the `BaseNotExist` subtest.
    #[test]
    fn a_missing_base_directory_is_an_error() {
        let base = TestDir::new();
        let td = TempDir::new(base.path().join("not-exist"), "sub");
        let Err(err) = td.mkdir_temp_random(&["foo"]) else {
            panic!("a missing base directory must fail");
        };
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}
