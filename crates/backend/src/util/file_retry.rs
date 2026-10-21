//! Port of `modules/util/file_retry.go`.
//!
//! On Windows an open file cannot be removed or renamed, so Gitea retries a few
//! times when a `git cat-batch` process is still exiting. The same retry is
//! harmless elsewhere and keeps behaviour identical across platforms.
//!
//! Upstream gates the Windows error code behind `consts.IsWindows`; here the
//! platform check is a `cfg!` so the constant is only consulted on Windows.

use std::io;
use std::path::Path;
use std::time::Duration;

/// `errWindowsSharingViolationError` — Windows error 32, "in use by another process".
#[cfg(windows)]
const ERR_WINDOWS_SHARING_VIOLATION: i32 = 32;

/// Whether an error means "the file is busy, try again".
fn is_err_busy(err: &io::Error) -> bool {
    // EBUSY happens on Unix when a directory is still a process's cwd.
    if err.raw_os_error() == Some(libc_ebusy()) {
        return true;
    }
    #[cfg(windows)]
    if err.raw_os_error() == Some(ERR_WINDOWS_SHARING_VIOLATION) {
        return true;
    }
    false
}

/// `syscall.EBUSY`.
const fn libc_ebusy() -> i32 {
    // 16 on Linux, 16 on macOS, 16 on the BSDs; Gitea only targets these.
    16
}

/// `util.retryWhenFileBusyInternal` — retries `f` while it reports a busy error.
pub fn retry_when_file_busy_internal(
    count: usize,
    delay: Duration,
    f: impl Fn() -> io::Result<()>,
) -> io::Result<()> {
    let mut err = None;
    for _ in 0..count {
        match f() {
            Ok(()) => return Ok(()),
            Err(e) => {
                if !is_err_busy(&e) {
                    return Err(e);
                }
                err = Some(e);
            }
        }
        std::thread::sleep(delay);
    }
    Err(err.unwrap_or_else(|| io::Error::other("retry exhausted")))
}

/// `util.retryWhenFileBusy` — the upstream defaults: 5 attempts, 100ms apart.
fn retry_when_file_busy(f: impl Fn() -> io::Result<()>) -> io::Result<()> {
    retry_when_file_busy_internal(5, Duration::from_millis(100), f)
}

/// `util.RemoveWithRetry`.
pub fn remove_with_retry(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref().to_path_buf();
    retry_when_file_busy(|| std::fs::remove_file(&path))
}

/// `util.RemoveAllWithRetry`.
pub fn remove_all_with_retry(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref().to_path_buf();
    retry_when_file_busy(|| std::fs::remove_dir_all(&path))
}

/// `util.RenameWithRetry`.
pub fn rename_with_retry(old: impl AsRef<Path>, new: impl AsRef<Path>) -> io::Result<()> {
    let old = old.as_ref().to_path_buf();
    let new = new.as_ref().to_path_buf();
    retry_when_file_busy(|| std::fs::rename(&old, &new))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn succeeds_without_retrying_when_the_first_attempt_works() {
        let calls = Cell::new(0);
        let r = retry_when_file_busy_internal(5, Duration::from_millis(1), || {
            calls.set(calls.get() + 1);
            Ok(())
        });
        assert!(r.is_ok());
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn gives_up_immediately_on_a_non_busy_error() {
        let calls = Cell::new(0);
        let r = retry_when_file_busy_internal(5, Duration::from_millis(1), || {
            calls.set(calls.get() + 1);
            Err(io::Error::new(io::ErrorKind::NotFound, "nope"))
        });
        assert!(r.is_err());
        assert_eq!(calls.get(), 1, "a non-busy error must not be retried");
    }

    #[test]
    fn retries_a_busy_error_then_reports_it() {
        let calls = Cell::new(0);
        let r = retry_when_file_busy_internal(3, Duration::from_millis(1), || {
            calls.set(calls.get() + 1);
            Err(io::Error::from_raw_os_error(libc_ebusy()))
        });
        assert!(r.is_err());
        assert_eq!(calls.get(), 3, "a busy error is retried the full count");
    }

    #[test]
    fn retries_a_busy_error_then_succeeds() {
        let calls = Cell::new(0);
        let r = retry_when_file_busy_internal(5, Duration::from_millis(1), || {
            calls.set(calls.get() + 1);
            if calls.get() < 3 {
                Err(io::Error::from_raw_os_error(libc_ebusy()))
            } else {
                Ok(())
            }
        });
        assert!(r.is_ok());
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn remove_all_with_retry_removes_a_directory_tree() {
        let dir = std::env::temp_dir().join(format!("codez-retry-{}", std::process::id()));
        let nested = dir.join("a/b");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("f"), b"x").unwrap();
        remove_all_with_retry(&dir).unwrap();
        assert!(!dir.exists());
    }
}
