//! Port of `modules/consts/runtime.go`.

/// `consts.IsWindows` — whether this build targets Windows.
pub const IS_WINDOWS: bool = cfg!(windows);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_build_target() {
        assert_eq!(IS_WINDOWS, cfg!(target_os = "windows"));
    }
}
