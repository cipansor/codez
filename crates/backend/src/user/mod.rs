//! Port of `gitea.dev/modules/user` — current OS username.
//!
//! | upstream | here |
//! | --- | --- |
//! | `user.go` | this module |

use crate::consts::IS_WINDOWS;

/// `user.CurrentUsername` — the current login OS username.
///
/// Upstream calls `os/user.Current()`, which reads the account database
/// (`/etc/passwd` on Unix, the token on Windows) rather than the environment;
/// the `whoami` crate is the direct equivalent. When the account lookup fails
/// this falls back to `USER` then `USERNAME`, exactly like upstream's
/// `fallbackCurrentUsername`.
pub fn current_username() -> String {
    let username = whoami::username().ok().or_else(fallback_current_username);

    match username {
        Some(u) => strip_domain(u),
        None => String::new(),
    }
}

/// `user.fallbackCurrentUsername` — used when the account lookup fails.
fn fallback_current_username() -> Option<String> {
    let cur = std::env::var("USER").ok().filter(|s| !s.is_empty());
    if cur.is_some() {
        return cur;
    }
    std::env::var("USERNAME").ok().filter(|s| !s.is_empty())
}

/// `user.CurrentUsername`'s Windows branch — drops a `DOMAIN\` prefix.
fn strip_domain(username: String) -> String {
    if IS_WINDOWS {
        match username.rsplit_once('\\') {
            Some((_, user)) => user.to_string(),
            None => username,
        }
    } else {
        username
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestCurrentUsername` in modules/user/user_test.go.
    #[test]
    fn current_username_matches_whoami() {
        let user = current_username();
        assert!(!user.is_empty(), "current username must not be empty");

        // Windows whoami is weird; upstream skips the comparison there.
        if IS_WINDOWS {
            return;
        }

        let whoami = std::process::Command::new("whoami")
            .output()
            .expect("whoami must run");
        let whoami = String::from_utf8_lossy(&whoami.stdout).trim().to_string();
        assert_eq!(current_username(), whoami);
    }

    /// Upstream's `t.Setenv("USER", "spoofed")` check: a spoofed `USER` must not
    /// change the answer, because the account database wins.
    #[test]
    fn a_spoofed_user_env_var_does_not_change_the_answer() {
        if IS_WINDOWS {
            return;
        }
        let expected = current_username();

        // `set_var` is unsafe in edition 2024 because it races with readers;
        // this test only reads afterwards, in a single-threaded test binary.
        unsafe { std::env::set_var("USER", "spoofed") };
        let got = current_username();
        unsafe { std::env::remove_var("USER") };

        assert_eq!(got, expected);
    }

    #[test]
    fn domain_prefix_is_stripped_only_on_windows() {
        let got = strip_domain(r"DOMAIN\alice".to_string());
        if IS_WINDOWS {
            assert_eq!(got, "alice");
        } else {
            assert_eq!(got, r"DOMAIN\alice");
        }
    }
}
