//! Port of `gitea.dev/modules/util` — display truncation.
//!
//! Mirrors `modules/util/truncate.go`. The width guess is deliberately a guess
//! upstream (CJK/emoji count as 2, combining marks as 1) and the result is
//! written to the database, so the invariant that matters is that a returned
//! string never exceeds the limit in guessed width.

pub const UTF8_ELLIPSIS: &str = "…";
pub const ASCII_ELLIPSIS: &str = "...";

/// `util.IsLikelyEllipsisLeftPart` — whether a string looks already truncated.
pub fn is_likely_ellipsis_left_part(s: &str) -> bool {
    s.ends_with(UTF8_ELLIPSIS) || s.ends_with(ASCII_ELLIPSIS)
}

/// `util.ellipsisDisplayGuessWidth` — approximate display width of a rune.
///
/// Must never return 0: the caller relies on every rune costing at least one
/// unit so the truncated value fits a `VARCHAR(n)` column.
pub fn ellipsis_display_guess_width(r: char) -> usize {
    if (r as u32) <= 255 {
        return 1;
    }
    match r {
        // Ideographic space still counts as 2, like other CJK.
        '\u{3000}' => 2,
        _ => {
            let is_combining = matches!(
                unicode_general_category(r),
                Category::Mark | Category::Format | Category::Surrogate
            );
            if is_combining || r.is_whitespace() {
                1
            } else {
                2
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    Mark,
    Format,
    Surrogate,
    Other,
}

/// Minimal stand-in for the `unicode.M` / `Cf` / `Cs` tables the Go code uses.
fn unicode_general_category(r: char) -> Category {
    let c = r as u32;
    match c {
        // Combining diacritical marks and common combining ranges.
        0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F => Category::Mark,
        // Zero-width and bidi control characters.
        0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x2064 | 0xFEFF => Category::Format,
        0xD800..=0xDFFF => Category::Surrogate,
        _ => Category::Other,
    }
}

/// `util.TruncateStringBytes` — truncates to a byte limit on a char boundary.
pub fn truncate_string_bytes(s: &str, limit: usize) -> String {
    let mut len = 0;
    for (i, c) in s.char_indices() {
        let rl = c.len_utf8();
        if len + rl > limit {
            return s[..i].to_string();
        }
        len += rl;
    }
    s.to_string()
}

/// `util.TruncateRunes` — truncates to a rune limit.
pub fn truncate_runes(s: &str, limit: usize) -> String {
    let count = s.chars().count();
    if count < limit {
        return s.to_string();
    }
    s.chars().take(limit).collect()
}

/// `util.EllipsisDisplayString` — truncates for display, appending an ellipsis.
pub fn ellipsis_display_string(s: &str, limit: usize) -> String {
    let (left, _, _, _) = ellipsis_display_string_inner(s, limit, ellipsis_display_guess_width);
    left
}

/// `util.EllipsisDisplayStringX` — truncation that also returns the removed tail.
pub fn ellipsis_display_string_x(s: &str, limit: usize) -> (String, String) {
    let (left, offset, truncated, encounter_invalid) =
        ellipsis_display_string_inner(s, limit, ellipsis_display_guess_width);
    if !truncated {
        return (left, String::new());
    }

    let right = &s[offset..];
    let invalid = encounter_invalid || right.starts_with(char::REPLACEMENT_CHARACTER);
    let ellipsis = if invalid {
        ASCII_ELLIPSIS
    } else {
        UTF8_ELLIPSIS
    };
    (left, format!("{ellipsis}{right}"))
}

/// `util.EllipsisTruncateRunes` — truncation where every rune costs 1.
pub fn ellipsis_truncate_runes(s: &str, limit: usize) -> (String, String) {
    let (left, offset, truncated, encounter_invalid) =
        ellipsis_display_string_inner(s, limit, |_| 1);
    if !truncated {
        return (left, String::new());
    }
    let right = &s[offset..];
    let invalid = encounter_invalid || right.starts_with(char::REPLACEMENT_CHARACTER);
    let ellipsis = if invalid {
        ASCII_ELLIPSIS
    } else {
        UTF8_ELLIPSIS
    };
    (left, format!("{ellipsis}{right}"))
}

fn ellipsis_display_string_inner(
    s: &str,
    limit: usize,
    width_guess: impl Fn(char) -> usize,
) -> (String, usize, bool, bool) {
    if s.len() <= limit {
        return (s.to_string(), s.len(), false, false);
    }

    let mut offset = 0usize;
    let mut used = 0usize;
    let mut pos = 0usize;
    let mut encounter_invalid = false;

    for (i, r) in s.char_indices() {
        encounter_invalid |= r == char::REPLACEMENT_CHARACTER;
        pos = i;
        let rune_width = width_guess(r);
        if used + rune_width + 3 > limit {
            break;
        }
        used += rune_width;
        offset += r.len_utf8();
    }

    // If only a handful of runes remain, include them instead of ellipsing.
    if s.len() - pos <= 12 {
        let mut next_cnt = 0usize;
        let mut next_width = 0usize;
        for r in s[pos..].chars() {
            if next_cnt >= 4 {
                break;
            }
            next_width += width_guess(r);
            next_cnt += 1;
        }
        if next_cnt <= 3 && used + next_width <= limit {
            return (s.to_string(), s.len(), false, false);
        }
    }

    if limit < 3 {
        // Too small for an ellipsis to help.
        return (s[..offset].to_string(), offset, true, false);
    }

    let ellipsis = if encounter_invalid {
        ASCII_ELLIPSIS
    } else {
        UTF8_ELLIPSIS
    };
    (
        format!("{}{ellipsis}", &s[..offset]),
        offset,
        true,
        encounter_invalid,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_strings_are_returned_unchanged() {
        assert_eq!(ellipsis_display_string("hello", 10), "hello");
        assert_eq!(truncate_runes("hello", 10), "hello");
        assert_eq!(truncate_string_bytes("hello", 10), "hello");
    }

    #[test]
    fn long_ascii_string_is_ellipsed_within_limit() {
        let s = "a".repeat(50);
        let out = ellipsis_display_string(&s, 10);
        assert!(out.ends_with(UTF8_ELLIPSIS), "got {out:?}");
        assert!(out.chars().count() <= 10, "got {out:?}");
    }

    #[test]
    fn ascii_ellipsis_used_for_invalid_utf8_replacement() {
        // The replacement char must be reached before the width limit trips,
        // because upstream only sets the flag while it is still iterating.
        let s = format!("\u{FFFD}{}", "a".repeat(50));
        let out = ellipsis_display_string(&s, 10);
        assert!(out.ends_with(ASCII_ELLIPSIS), "got {out:?}");
    }

    #[test]
    fn truncate_string_bytes_respects_char_boundaries() {
        // "é" is 2 bytes: a limit of 3 admits "a"+"é", a limit of 2 admits only "a".
        assert_eq!(truncate_string_bytes("aé", 3), "aé");
        assert_eq!(truncate_string_bytes("aé", 2), "a");
        assert_eq!(truncate_string_bytes("aé", 4), "aé");
    }

    #[test]
    fn ellipsis_display_string_x_returns_removed_tail() {
        let s = "a".repeat(50);
        let (left, right) = ellipsis_display_string_x(&s, 10);
        assert!(!left.is_empty());
        assert!(right.starts_with(UTF8_ELLIPSIS));
        // Both halves carry their own ellipsis, so subtract one from each.
        let reconstructed = left.len() - UTF8_ELLIPSIS.len() + right.len() - UTF8_ELLIPSIS.len();
        assert_eq!(reconstructed, s.len());
    }

    #[test]
    fn guess_width_is_never_zero() {
        for c in ['a', '\u{3000}', '\u{0301}', '\u{200B}', '啊', '🙂'] {
            assert!(ellipsis_display_guess_width(c) >= 1, "rune {c:?}");
        }
    }

    #[test]
    fn is_likely_ellipsis_left_part_detects_both_forms() {
        assert!(is_likely_ellipsis_left_part("abc…"));
        assert!(is_likely_ellipsis_left_part("abc..."));
        assert!(!is_likely_ellipsis_left_part("abc"));
    }
}
