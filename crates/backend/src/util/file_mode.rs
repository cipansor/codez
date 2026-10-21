//! Port of `modules/util/file_unix.go` and `modules/util/file_windows.go`.
//!
//! Upstream splits this across two build-tagged files: Unix reads the process
//! umask once and masks new modes with it; Windows does nothing because it has
//! no umask. Both behaviours live here behind `cfg`.

#[cfg(unix)]
mod imp {
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::sync::OnceLock;

    /// The process umask, read once.
    ///
    /// Go obtains this by calling `unix.Umask` twice (set a temporary value,
    /// then restore). Without a libc dependency the only way to *read* it is
    /// `/proc/self/status`, which exists on Linux. On other Unixes the
    /// conventional `0o022` is assumed.
    fn default_umask() -> u32 {
        static UMASK: OnceLock<u32> = OnceLock::new();
        *UMASK.get_or_init(|| {
            std::fs::read_to_string("/proc/self/status")
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find_map(|l| l.strip_prefix("Umask:"))
                        .and_then(|v| u32::from_str_radix(v.trim(), 8).ok())
                })
                .unwrap_or(0o022)
        })
    }

    /// `util.ApplyUmask` — chmods `f` to `new_mode & !umask`.
    pub fn apply_umask(f: impl AsRef<Path>, new_mode: u32) -> std::io::Result<()> {
        apply_umask_with(f, new_mode, default_umask())
    }

    /// [`apply_umask`] with an explicit umask, for callers that know it.
    pub fn apply_umask_with(f: impl AsRef<Path>, new_mode: u32, umask: u32) -> std::io::Result<()> {
        let mode = new_mode & !umask;
        std::fs::set_permissions(f, std::fs::Permissions::from_mode(mode))
    }

    /// The umask this process started with, for callers that want to display or
    /// apply it themselves.
    pub fn process_umask() -> u32 {
        default_umask()
    }
}

#[cfg(not(unix))]
mod imp {
    use std::path::Path;

    /// `util.ApplyUmask` on Windows — a no-op, because Windows has no umask.
    pub fn apply_umask(_f: impl AsRef<Path>, _new_mode: u32) -> std::io::Result<()> {
        Ok(())
    }

    /// See [`apply_umask`]; the explicit-umask variant is also a no-op.
    pub fn apply_umask_with(
        _f: impl AsRef<Path>,
        _new_mode: u32,
        _umask: u32,
    ) -> std::io::Result<()> {
        Ok(())
    }

    /// Windows has no umask; report 0 so masking is a no-op.
    pub fn process_umask() -> u32 {
        0
    }
}

pub use imp::{apply_umask, apply_umask_with, process_umask};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_umask_does_not_fail_on_a_real_file() {
        let path = std::env::temp_dir().join(format!("codez-umask-{}", std::process::id()));
        std::fs::write(&path, b"x").unwrap();
        apply_umask(&path, 0o644).unwrap();
        let _ = std::fs::remove_file(&path);
    }

    #[cfg(unix)]
    #[test]
    fn apply_umask_masks_the_mode() {
        use std::os::unix::fs::PermissionsExt;
        let path = std::env::temp_dir().join(format!("codez-umask2-{}", std::process::id()));
        std::fs::write(&path, b"x").unwrap();

        // 0o777 with a 0o077 umask leaves 0o700.
        apply_umask_with(&path, 0o777, 0o077).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);

        let _ = std::fs::remove_file(&path);
    }

    #[cfg(unix)]
    #[test]
    fn process_umask_is_a_plausible_value() {
        // Masking with it must not be able to clear all permission bits.
        let umask = process_umask();
        assert!(umask <= 0o777, "umask {umask:o} out of range");
    }
}
