//! Port of `gitea.dev/modules/util` — string helpers.
//!
//! Mirrors `modules/util/string.go`, `truncate.go`, and the string helpers in
//! `util.go`.

/// `util.ToSnakeCase` — converts an identifier to `snake_case`.
///
/// The upstream implementation is byte-oriented and lets non-ASCII bytes pass
/// through untouched, so it is reproduced byte-for-byte rather than delegated to
/// a Unicode-aware library; `TestToSnakeCase` pins the exact outputs, including
/// the quirks (`ObjIDs` -> `obj_i_ds`).
pub fn to_snake_case(input: &str) -> String {
    let b = input.as_bytes();
    if b.is_empty() {
        return String::new();
    }

    let is_upper = |c: u8| c.is_ascii_uppercase();
    let is_lower_or_num = |c: u8| c.is_ascii_lowercase() || c.is_ascii_digit();

    if b.len() == 1 {
        let c = b[0];
        return if is_upper(c) {
            ((c + b'a' - b'A') as char).to_string()
        } else if is_lower_or_num(c) {
            (c as char).to_string()
        } else {
            "_".to_string()
        };
    }

    let mut res: Vec<u8> = Vec::with_capacity(b.len() * 4 / 3);
    let mut pos = 0usize;
    let mut need_sep = false;

    while pos < b.len() {
        let c = b[pos];
        if c >= 0x80 {
            res.push(c);
            pos += 1;
            continue;
        }

        if is_upper(c) || is_lower_or_num(c) {
            let mut end = pos + 1;
            if is_upper(c) {
                // Consume the run of capitals, then back off one so the last
                // capital starts the following word ("HTTPServer" -> "http_server").
                while end < b.len() && is_upper(b[end]) {
                    end += 1;
                }
                if end - pos > 1 && end < b.len() && is_lower_or_num(b[end]) {
                    end -= 1;
                }
            }
            while end < b.len() && (is_lower_or_num(b[end]) || b[end] >= 0x80) {
                end += 1;
            }
            if need_sep {
                res.push(b'_');
            }
            res.extend_from_slice(&b[pos..end]);
            pos = end;
            need_sep = true;
        } else {
            res.push(b'_');
            pos += 1;
            need_sep = false;
        }
    }

    for byte in res.iter_mut() {
        if is_upper(*byte) {
            *byte += b'a' - b'A';
        }
    }

    // Input was validated UTF-8 and only ASCII case folding happened, so this
    // cannot produce invalid UTF-8.
    String::from_utf8(res).unwrap_or_default()
}

/// `util.SplitTrimSpace` — splits on `sep`, trimming each part and dropping empties.
pub fn split_trim_space(input: &str, sep: &str) -> Vec<String> {
    input
        .trim()
        .split(sep)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// `util.AsciiEqualFold` — ASCII-only case-insensitive comparison.
pub fn ascii_equal_fold(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.eq_ignore_ascii_case(y))
}

/// `util.ToLowerASCII` — lowercases ASCII letters only, leaving the rest alone.
pub fn to_lower_ascii(s: &str) -> String {
    // Operate on bytes: the input is valid UTF-8 and ASCII case folding never
    // touches a continuation byte, so the result stays valid UTF-8.
    let bytes: Vec<u8> = s
        .bytes()
        .map(|b| if b.is_ascii_uppercase() { b + 32 } else { b })
        .collect();
    String::from_utf8(bytes).unwrap_or_default()
}

/// `util.NormalizeEOL` — converts CRLF and CR line endings to LF.
pub fn normalize_eol(input: &[u8]) -> Vec<u8> {
    let Some(first_cr) = input.iter().position(|&c| c == b'\r') else {
        return input.to_vec();
    };

    let mut out = Vec::with_capacity(input.len());
    out.extend_from_slice(&input[..first_cr]);
    out.push(b'\n');

    let mut left = first_cr + 1;
    while left < input.len() {
        if input[left] == b'\n' {
            left += 1;
        }
        match input[left..].iter().position(|&c| c == b'\r') {
            Some(rel) => {
                let right = left + rel;
                out.extend_from_slice(&input[left..right]);
                out.push(b'\n');
                left = right + 1;
            }
            None => {
                out.extend_from_slice(&input[left..]);
                break;
            }
        }
    }
    out
}

/// `util.NormalizeStringEOL` — [`normalize_eol`] for a string.
pub fn normalize_string_eol(input: &str) -> String {
    String::from_utf8_lossy(&normalize_eol(input.as_bytes())).into_owned()
}

/// `util.DiffSlice` — elements added to and removed from a slice, de-duplicated
/// and preserving first-seen order.
pub fn diff_slice<T: PartialEq + Clone>(old: &[T], new: &[T]) -> (Vec<T>, Vec<T>) {
    let mut added = Vec::new();
    let mut removed = Vec::new();

    for v in new {
        if !old.contains(v) && !added.contains(v) {
            added.push(v.clone());
        }
    }
    for v in old {
        if !new.contains(v) && !removed.contains(v) {
            removed.push(v.clone());
        }
    }
    (added, removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestToSnakeCase` in modules/util/string_test.go.
    #[test]
    fn to_snake_case_matches_upstream() {
        let cases: &[(&str, &str)] = &[
            ("HTTPServer", "http_server"),
            ("_camelCase", "_camel_case"),
            ("NoHTTPS", "no_https"),
            ("Wi_thF", "wi_th_f"),
            ("_AnotherTES_TCaseP", "_another_tes_t_case_p"),
            ("ALL", "all"),
            ("_HELLO_WORLD_", "_hello_world_"),
            ("HELLO_WORLD", "hello_world"),
            ("HELLO____WORLD", "hello____world"),
            ("TW", "tw"),
            ("_C", "_c"),
            ("  sentence case  ", "__sentence_case__"),
            (
                " Mixed-hyphen case _and SENTENCE_case and UPPER-case",
                "_mixed_hyphen_case__and_sentence_case_and_upper_case",
            ),
            (" ", "_"),
            ("A", "a"),
            ("A0", "a0"),
            ("a0", "a0"),
            ("Aa0", "aa0"),
            ("啊", "啊"),
            ("A啊", "a啊"),
            ("Aa啊b", "aa啊b"),
            ("A啊B", "a啊_b"),
            ("Aa啊B", "aa啊_b"),
            ("TheCase2", "the_case2"),
            ("ObjIDs", "obj_i_ds"),
            ("", ""),
        ];
        for (input, expected) in cases {
            assert_eq!(&to_snake_case(input), expected, "input: {input:?}");
        }
    }

    // Transcribed from `TestSplitTrimSpace`.
    #[test]
    fn split_trim_space_matches_upstream() {
        assert_eq!(split_trim_space("a\nb\nc", "\n"), ["a", "b", "c"]);
        assert_eq!(split_trim_space("\r\na\n\r\nb\n\n", "\n"), ["a", "b"]);
    }

    #[test]
    fn ascii_equal_fold_is_ascii_only() {
        assert!(ascii_equal_fold(b"Hello", b"hELLo"));
        assert!(!ascii_equal_fold(b"Hello", b"hell"));
        assert!(!ascii_equal_fold("Ä".as_bytes(), "ä".as_bytes()));
    }

    #[test]
    fn to_lower_ascii_leaves_non_ascii() {
        assert_eq!(to_lower_ascii("AbC"), "abc");
        assert_eq!(to_lower_ascii("Äb"), "Äb");
    }

    #[test]
    fn normalize_eol_handles_crlf_and_cr() {
        assert_eq!(normalize_eol(b"a\r\nb"), b"a\nb");
        assert_eq!(normalize_eol(b"a\rb"), b"a\nb");
        assert_eq!(normalize_eol(b"a\nb"), b"a\nb");
        assert_eq!(normalize_eol(b""), b"");
    }

    #[test]
    fn diff_slice_dedupes_and_preserves_order() {
        let (added, removed) = diff_slice(&[1, 2, 3], &[3, 4, 4, 5]);
        assert_eq!(added, [4, 5]);
        assert_eq!(removed, [1, 2]);
    }
}
