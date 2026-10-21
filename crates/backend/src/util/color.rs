//! Port of `modules/util/color.go`.
//!
//! Keep the constants in sync with `web_src/js/utils/color.ts` upstream; the
//! frontend and backend must agree on which foreground colour a background
//! gets.

/// `util.HexToRBGColor` — parses `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa` into
/// 0..255 channel values. Anything else yields black.
///
/// The name's typo (`RBG`) is upstream's and is preserved so the mapping stays
/// greppable.
pub fn hex_to_rbg_color(color_string: &str) -> (f64, f64, f64) {
    let hex_string = color_string.strip_prefix('#').unwrap_or(color_string);

    // Only rgb, rgba, rrggbb, and rrggbbaa are supported.
    if !matches!(hex_string.len(), 3 | 4 | 6 | 8) {
        return (0.0, 0.0, 0.0);
    }

    // A 3- or 4-digit form duplicates each nibble.
    let expanded = if matches!(hex_string.len(), 3 | 4) {
        let b = hex_string.as_bytes();
        format!(
            "{}{}{}{}{}{}",
            b[0] as char, b[0] as char, b[1] as char, b[1] as char, b[2] as char, b[2] as char
        )
    } else {
        hex_string.to_string()
    };

    // The alpha channel is dropped.
    let six = &expanded[..6];
    let Ok(color) = u32::from_str_radix(six, 16) else {
        return (0.0, 0.0, 0.0);
    };

    (
        f64::from((0xFF & (color >> 16)) as u8),
        f64::from((0xFF & (color >> 8)) as u8),
        f64::from((0xFF & color) as u8),
    )
}

/// `util.linearizeChannel` — undoes the sRGB transfer function.
fn linearize_channel(channel: f64) -> f64 {
    let srgb = channel / 255.0;
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

/// `util.getRelativeLuminance` — WCAG relative luminance.
fn get_relative_luminance(color: &str) -> f64 {
    let (r, g, b) = hex_to_rbg_color(color);
    0.2126 * linearize_channel(r) + 0.7152 * linearize_channel(g) + 0.0722 * linearize_channel(b)
}

/// `util.GetPerceivedBrightness` — weighted gamma-encoded channels.
///
/// This is not a luminance; it stays in the raw-channel space so channels can
/// be scaled proportionally. Do not use it for contrast decisions.
pub fn get_perceived_brightness(color: &str) -> f64 {
    let (r, g, b) = hex_to_rbg_color(color);
    (0.2126729 * r + 0.7151522 * g + 0.0721750 * b) / 255.0
}

/// `util.UseLightText` — whether a background needs light foreground text.
///
/// The 0.36 threshold tracks APCA better than WCAG's own 0.179.
pub fn use_light_text(background_color: &str) -> bool {
    get_relative_luminance(background_color) < 0.36
}

/// `util.ContrastColor` — black or white, whichever contrasts more.
pub fn contrast_color(background_color: &str) -> &'static str {
    if use_light_text(background_color) {
        "#fff"
    } else {
        "#000"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `Test_HexToRBGColor` in modules/util/color_test.go.
    #[test]
    fn hex_to_rbg_color_matches_upstream() {
        let cases: &[(&str, f64, f64, f64)] = &[
            ("2b8685", 43.0, 134.0, 133.0),
            ("1e1", 17.0, 238.0, 17.0),
            ("#1e1", 17.0, 238.0, 17.0),
            ("1e16", 17.0, 238.0, 17.0),
            ("3bb6b3", 59.0, 182.0, 179.0),
            ("#3bb6b399", 59.0, 182.0, 179.0),
            ("#0", 0.0, 0.0, 0.0),
            ("#00000", 0.0, 0.0, 0.0),
            ("#1234567", 0.0, 0.0, 0.0),
        ];
        for (i, (s, er, eg, eb)) in cases.iter().enumerate() {
            let (r, g, b) = hex_to_rbg_color(s);
            assert_eq!((r, g, b), (*er, *eg, *eb), "case {i}: {s}");
        }
    }

    // Transcribed from `Test_UseLightText` in modules/util/color_test.go.
    #[test]
    fn contrast_color_matches_upstream() {
        let cases: &[(&str, &str)] = &[
            ("#d73a4a", "#fff"),
            ("#0075ca", "#fff"),
            ("#cfd3d7", "#000"),
            ("#a2eeef", "#000"),
            ("#7057ff", "#fff"),
            ("#008672", "#fff"),
            ("#e4e669", "#000"),
            ("#d876e3", "#fff"),
            ("#ffffff", "#000"),
            ("#2b8684", "#fff"),
            ("#2b8786", "#fff"),
            ("#2c8786", "#fff"),
            ("#2bb3b2", "#fff"),
            ("#2bb4b3", "#000"),
            ("#3bb6b3", "#000"),
            ("#7c7268", "#fff"),
            ("#7e716c", "#fff"),
            ("#81706d", "#fff"),
            ("#807070", "#fff"),
            ("#84b6eb", "#000"),
        ];
        for (i, (color, expected)) in cases.iter().enumerate() {
            assert_eq!(&contrast_color(color), expected, "case {i}: {color}");
        }
    }

    #[test]
    fn perceived_brightness_is_bounded() {
        assert!(get_perceived_brightness("#ffffff") > 0.99);
        assert!(get_perceived_brightness("#000000") < 0.01);
    }
}
