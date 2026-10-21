//! Port of `modules/util/runtime.go`.
//!
//! Go can walk the call stack and name the calling function. Rust cannot do
//! that portably, but a macro expanded inside a function body can, because a
//! nested item's `type_name` includes the path of the scope that encloses it.
//! [`caller_func_name`] uses that, so it reports the enclosing function rather
//! than the caller's caller — which is what upstream's helper is used for.

/// `util.CallerFuncName` — the current function's name, trimmed to its last two
/// path segments so it reads like Go's `pkg.Func`.
///
/// Must be used inside a function body:
///
/// ```
/// # use codez_backend::caller_func_name;
/// fn my_fn() -> String {
///     caller_func_name!()
/// }
/// assert!(my_fn().ends_with("my_fn"));
/// ```
#[macro_export]
macro_rules! caller_func_name {
    () => {{
        fn __codez_type_of<T>(_: T) -> &'static str {
            ::std::any::type_name::<T>()
        }
        fn __codez_marker() {}
        let full = __codez_type_of(__codez_marker);
        // `full` is "<scope path>::__codez_marker"; drop the marker.
        let scope = full.strip_suffix("::__codez_marker").unwrap_or(full);
        // Keep only the last two segments, mirroring upstream's
        // "only keep the last package name and func name".
        let parts: ::std::vec::Vec<&str> = scope.split("::").collect();
        let keep = parts.len().saturating_sub(2);
        parts[keep..].join("::")
    }};
}

/// `util.CallerFuncName`'s path-trimming rule, exposed for callers that already
/// have a full path (for example from [`std::any::type_name`]).
pub fn trim_func_name(full_path: &str) -> String {
    let parts: Vec<&str> = full_path.split("::").collect();
    let keep = parts.len().saturating_sub(2);
    parts[keep..].join("::")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestCallerFuncName` in modules/util/runtime_test.go,
    // adapted: Rust reports the enclosing module path rather than Go's
    // "util.TestCallerFuncName", so assert the shape and the function name.
    #[test]
    fn caller_func_name_names_the_enclosing_function() {
        let name = caller_func_name!();
        assert!(
            name.ends_with("caller_func_name_names_the_enclosing_function"),
            "got {name}"
        );
        // Only the last two segments are kept.
        assert!(name.split("::").count() <= 2, "got {name}");
    }

    #[test]
    fn trim_keeps_the_last_two_segments() {
        assert_eq!(
            trim_func_name("codez_backend::util::runtime::tests::my_test"),
            "tests::my_test"
        );
        assert_eq!(trim_func_name("my_fn"), "my_fn");
        assert_eq!(trim_func_name(""), "");
    }
}
