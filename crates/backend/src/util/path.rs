//! Port of `gitea.dev/modules/util` — path helpers.
//!
//! Mirrors `modules/util/path.go`. The Go code leans on `path.Clean` /
//! `path.Join`; `clean_posix_path` reproduces those semantics because the
//! upstream tests assert exact outputs and the results are used as storage
//! keys, so a "close enough" cleaning would silently change behaviour.

use std::fmt;

/// Error returned when a path element would escape its own level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotRegularPathFile;

impl fmt::Display for NotRegularPathFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a regular file")
    }
}

impl std::error::Error for NotRegularPathFile {}

/// Reproduces Go's `path.Clean` for slash-separated paths.
///
/// * collapses repeated slashes
/// * drops `.` elements
/// * resolves `..` against the previous element, and drops leading `..` at root
/// * preserves a leading `/`; returns `"."` when the result would be empty
pub fn clean_posix_path(p: &str) -> String {
    if p.is_empty() {
        return ".".to_string();
    }

    let rooted = p.starts_with('/');
    let mut out: Vec<&str> = Vec::new();

    for seg in p.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                // `..` pops a real parent, but never pops past the root.
                match out.last() {
                    Some(&last) if last != ".." => {
                        out.pop();
                    }
                    _ if !rooted => out.push(".."),
                    _ => {}
                }
            }
            s => out.push(s),
        }
    }

    let joined = out.join("/");
    match (rooted, joined.is_empty()) {
        (true, true) => "/".to_string(),
        (true, false) => format!("/{joined}"),
        (false, true) => ".".to_string(),
        (false, false) => joined,
    }
}

/// `util.PathJoinRel` — joins elements, cleaning each one independently.
///
/// Returns `""` for an empty result and `"."` when everything cancels out.
pub fn path_join_rel<S: AsRef<str>>(elem: &[S]) -> String {
    let cleaned: Vec<String> = elem
        .iter()
        .map(|e| {
            let e = e.as_ref();
            if e.is_empty() {
                String::new()
            } else {
                clean_posix_path(&format!("/{e}"))
            }
        })
        .filter(|e| !e.is_empty())
        .collect();

    if cleaned.is_empty() {
        return String::new();
    }

    match clean_posix_path(&cleaned.join("/")).as_str() {
        "" => String::new(),
        "/" => ".".to_string(),
        p => p[1..].to_string(),
    }
}

/// `util.PathJoinRelX` — like [`path_join_rel`] but treats `\` as `/`.
pub fn path_join_rel_x<S: AsRef<str>>(elem: &[S]) -> String {
    let normalized: Vec<String> = elem.iter().map(|e| e.as_ref().replace('\\', "/")).collect();
    path_join_rel(&normalized)
}

/// `util.FilePathJoinAbs` — joins onto an absolute base, rejecting a relative one.
pub fn file_path_join_abs<S: AsRef<str>>(
    base: &str,
    sub: &[S],
) -> Result<String, NotRegularPathFile> {
    let base = base.replace('\\', "/");
    if !base.starts_with('/') {
        return Err(NotRegularPathFile);
    }

    let mut parts = vec![clean_posix_path(&base)];
    for s in sub {
        let s = s.as_ref();
        if s.is_empty() {
            continue;
        }
        parts.push(clean_posix_path(&format!("/{}", s.replace('\\', "/"))));
    }

    Ok(clean_posix_path(&parts.join("/")))
}

/// `util.PathBaseStem` — the base name with its extension removed.
pub fn path_base_stem(s: &str) -> String {
    let base = match s.rfind('/') {
        Some(i) => &s[i + 1..],
        None => s,
    };
    match base.rfind('.') {
        // A leading dot is part of the name, not an extension separator.
        Some(0) | None => base.to_string(),
        Some(i) => base[..i].to_string(),
    }
}

/// `util.PathEscapeSegments` — percent-escapes each segment, keeping `/`.
pub fn path_escape_segments(path: &str) -> String {
    path.split('/')
        .map(percent_encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

/// Percent-encodes one path segment the way Go's `url.PathEscape` does.
fn percent_encode_segment(seg: &str) -> String {
    let mut out = String::with_capacity(seg.len());
    for b in seg.bytes() {
        let unreserved = b.is_ascii_alphanumeric()
            || matches!(
                b,
                b'-' | b'_' | b'.' | b'~' | b'$' | b'&' | b'+' | b',' | b':' | b'=' | b'@'
            );
        if unreserved {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Cases transcribed from `TestCleanPath` in modules/util/path_test.go.
    #[test]
    fn path_join_rel_matches_upstream() {
        let cases: &[(&[&str], &str)] = &[
            (&[], ""),
            (&[""], ""),
            (&[".."], "."),
            (&["a"], "a"),
            (&["/a/"], "a"),
            (&["../a/", "../b", "c/..", "d"], "a/b/d"),
            (&["a\\..\\b"], "a\\..\\b"),
            (&["a", "", "b"], "a/b"),
            (&["a", "..", "b"], "a/b"),
            (&["lfs", "repo/..", "user/../path"], "lfs/path"),
        ];
        for (elems, expected) in cases {
            assert_eq!(&path_join_rel(elems), expected, "case: {elems:?}");
        }
    }

    #[test]
    fn path_join_rel_x_treats_backslash_as_separator() {
        let cases: &[(&[&str], &str)] = &[
            (&[], ""),
            (&[""], ""),
            (&[".."], "."),
            (&["a"], "a"),
            (&["/a/"], "a"),
            (&["../a/", "../b", "c/..", "d"], "a/b/d"),
            (&["a\\..\\b"], "b"),
            (&["a", "", "b"], "a/b"),
            (&["a", "..", "b"], "a/b"),
            (&["lfs", "repo/..", "user/../path"], "lfs/path"),
        ];
        for (elems, expected) in cases {
            assert_eq!(&path_join_rel_x(elems), expected, "case: {elems:?}");
        }
    }

    #[test]
    fn file_path_join_abs_posix_cases() {
        let cases: &[(&[&str], &str)] = &[
            (&["/.."], "/"),
            (&["/a"], "/a"),
            (&["/a/"], "/a"),
            (&["/../a/", "../b", "c/..", "d"], "/a/b/d"),
            (&["/a\\..\\b"], "/b"),
            (&["/a", "", "b"], "/a/b"),
            (&["/a", "..", "b"], "/a/b"),
            (&["/lfs", "repo/..", "user/../path"], "/lfs/path"),
        ];
        for (elems, expected) in cases {
            let joined = file_path_join_abs(elems[0], &elems[1..]).unwrap();
            assert_eq!(&joined, expected, "case: {elems:?}");
        }
    }

    #[test]
    fn file_path_join_abs_rejects_relative_base() {
        assert!(file_path_join_abs("relative", &["a"]).is_err());
    }

    #[test]
    fn path_base_stem_keeps_dotfiles() {
        assert_eq!(path_base_stem("a/b/c.txt"), "c");
        assert_eq!(path_base_stem("a/b/.gitignore"), ".gitignore");
        assert_eq!(path_base_stem("noext"), "noext");
    }

    #[test]
    fn path_escape_segments_keeps_slashes() {
        assert_eq!(path_escape_segments("a b/c d"), "a%20b/c%20d");
        assert_eq!(path_escape_segments("a/b"), "a/b");
    }
}
