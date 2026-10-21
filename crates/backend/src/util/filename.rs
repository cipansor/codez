//! Port of `modules/util/filename.go`.

use crate::util::truncate::truncate_string_bytes;

/// `util.PathNameValidator` — the invalid-character and reserved-name patterns.
///
/// Built once on first use, like upstream's `sync.OnceValue`.
fn invalid_chars_pattern() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r#"(?i)[<>:"/\\|?*\x{0000}-\x{001F}]"#).expect("valid pattern")
    })
}

fn invalid_names_pattern() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    // "COM10" is valid because it needs a UNC path to be special.
    RE.get_or_init(|| {
        regex::Regex::new(r"(?i)^(con|prn|aux|nul|com\d|lpt\d)$").expect("valid pattern")
    })
}

/// `util.PathNameValidator().InvalidChars` — replaces invalid characters with `_`.
pub fn sanitize_invalid_chars(name: &str) -> String {
    invalid_chars_pattern().replace_all(name, "_").into_owned()
}

/// Whether a name matches the reserved-device-name pattern.
pub fn is_reserved_file_name(name: &str) -> bool {
    invalid_names_pattern().is_match(name)
}

/// `util.FileNameJoinFields` — joins fields into a safe filename.
///
/// The final field is treated as the extension and is not otherwise rewritten.
pub fn file_name_join_fields(fields: &[String]) -> String {
    // Linux caps a filename at 255 bytes; leave room for separators and the ext.
    const MAX_FILENAME_LEN: usize = 210;
    file_name_join_fields_with_limit(MAX_FILENAME_LEN, fields)
}

/// The limit is a suggestion: enough fields can still push past it, as upstream notes.
pub fn file_name_join_fields_with_limit(stem_name_limit: usize, fields: &[String]) -> String {
    let mut out = String::new();

    for (i, raw) in fields.iter().enumerate() {
        let mut field = sanitize_invalid_chars(raw);
        if i < fields.len() - 1 {
            field = field.replace(['.', '-'], "_");
            let estimated_remaining = (fields.len() - 1 - i) * 3;
            let estimated_limit = stem_name_limit.saturating_sub(estimated_remaining);
            if out.len() + field.len() > estimated_limit {
                let room = estimated_limit.saturating_sub(out.len() + 2);
                field = format!("{}__", truncate_string_bytes(&field, room));
            }
            out.push_str(&field);
            if i < fields.len() - 2 {
                out.push('-');
            }
        } else {
            out.push_str(&field);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    // Transcribed from `TestFileNameJoinFields` in modules/util/filename_test.go.
    #[test]
    fn file_name_join_fields_matches_upstream() {
        assert_eq!(
            file_name_join_fields(&f(&["a-b", "1", ".txt"])),
            "a_b-1.txt"
        );
        assert_eq!(
            file_name_join_fields(&f(&["a", "/../", "b", ".txt"])),
            "a-____-b.txt"
        );
        assert_eq!(
            file_name_join_fields(&f(&["a", "\\..\\", "b", ".txt"])),
            "a-____-b.txt"
        );

        assert_eq!(
            file_name_join_fields_with_limit(14, &f(&["🌞", "🌛", ".txt"])),
            "🌞-🌛.txt"
        );
        assert_eq!(
            file_name_join_fields_with_limit(14, &f(&["🌞", "🌛🌛🌛🌛", ".txt"])),
            "🌞-🌛__.txt"
        );
        assert_eq!(
            file_name_join_fields_with_limit(14, &f(&["🌞🌞🌞🌞", "🌛🌛🌛🌛", ".txt"])),
            "🌞__-__.txt"
        );
    }

    #[test]
    fn reserved_names_are_detected() {
        assert!(is_reserved_file_name("CON"));
        assert!(is_reserved_file_name("com1"));
        assert!(is_reserved_file_name("LPT9"));
        assert!(!is_reserved_file_name("console"));
    }

    #[test]
    fn invalid_characters_are_replaced() {
        assert_eq!(sanitize_invalid_chars("a/b"), "a_b");
        assert_eq!(sanitize_invalid_chars("a:b"), "a_b");
        assert_eq!(sanitize_invalid_chars("ok-name.txt"), "ok-name.txt");
    }
}
