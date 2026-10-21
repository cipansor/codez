//! Port of `modules/util/bytesize.go`.
//!
//! Upstream delegates to `github.com/dustin/go-humanize`; the formatting and
//! parsing rules are reimplemented here so the output matches exactly, since
//! these strings are shown to users and the parse side accepts user input.

const KIB: u64 = 1024;
const MIB: u64 = KIB * 1024;
const GIB: u64 = MIB * 1024;
const TIB: u64 = GIB * 1024;
const PIB: u64 = TIB * 1024;
const EIB: u64 = PIB * 1024;

const SIZES: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

/// `util.FormatByteSize` — formats a byte count as e.g. `"512 KiB"`.
pub fn format_byte_size(size: i64) -> String {
    let s = size.max(0) as u64;
    if s < 10 {
        return format!("{s} B");
    }

    // floor(log(s, 1024)) without floating point drift.
    let mut e = 0usize;
    let mut pow = 1u64;
    while e + 1 < SIZES.len() && s >= pow.saturating_mul(1024) {
        pow = pow.saturating_mul(1024);
        e += 1;
    }

    let val = ((s as f64 / pow as f64) * 10.0 + 0.5).floor() / 10.0;
    let suffix = SIZES[e];
    if val < 10.0 {
        format!("{val:.1} {suffix}")
    } else {
        format!("{val:.0} {suffix}")
    }
}

/// `util.ParseByteSize` — parses `"100MB"`, `"1.5 GiB"`, and friends.
pub fn parse_byte_size(value: &str) -> Result<u64, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("unhandled size name: ".to_string());
    }

    let split = trimmed
        .char_indices()
        .find(|(_, c)| !(c.is_ascii_digit() || *c == '.' || *c == ','))
        .map(|(i, _)| i)
        .unwrap_or(trimmed.len());

    let (num, unit) = trimmed.split_at(split);
    let num = num.replace(',', "");
    if num.is_empty() {
        return Err(format!(
            "unhandled size name: {}",
            unit.trim().to_lowercase()
        ));
    }

    let f: f64 = num
        .parse()
        .map_err(|_| format!("unhandled size name: {}", unit.trim().to_lowercase()))?;

    let extra = unit.trim().to_lowercase();
    let mult = match extra.as_str() {
        "b" => 1,
        "kib" => KIB,
        "kb" => 1000,
        "mib" => MIB,
        "mb" => 1000u64.pow(2),
        "gib" => GIB,
        "gb" => 1000u64.pow(3),
        "tib" => TIB,
        "tb" => 1000u64.pow(4),
        "pib" => PIB,
        "pb" => 1000u64.pow(5),
        "eib" => EIB,
        "eb" => 1000u64.pow(6),
        other => return Err(format!("unhandled size name: {other}")),
    };

    let scaled = f * mult as f64;
    if scaled >= u64::MAX as f64 {
        return Err(format!("too large: {value}"));
    }
    Ok(scaled as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestFormatByteSize` in modules/util/bytesize_test.go.
    #[test]
    fn format_byte_size_matches_upstream() {
        let mut size: i64 = 512;
        assert_eq!(format_byte_size(size), "512 B");
        size *= 1024;
        assert_eq!(format_byte_size(size), "512 KiB");
        size *= 1024;
        assert_eq!(format_byte_size(size), "512 MiB");
        size *= 1024;
        assert_eq!(format_byte_size(size), "512 GiB");
        size *= 1024;
        assert_eq!(format_byte_size(size), "512 TiB");
        size *= 1024;
        assert_eq!(format_byte_size(size), "512 PiB");
        size *= 4;
        assert_eq!(format_byte_size(size), "2.0 EiB");
    }

    #[test]
    fn parse_byte_size_handles_binary_and_decimal_units() {
        assert_eq!(parse_byte_size("100MB").unwrap(), 100_000_000);
        assert_eq!(
            parse_byte_size("1.5 GiB").unwrap(),
            (1.5 * GIB as f64) as u64
        );
        assert_eq!(parse_byte_size("512 B").unwrap(), 512);
        assert!(parse_byte_size("100XB").is_err());
        assert!(parse_byte_size("").is_err());
        // A bare number has no unit suffix; go-humanize rejects it, and so do we.
        assert!(parse_byte_size("512").is_err());
    }

    #[test]
    fn round_trips_a_known_value() {
        assert_eq!(
            parse_byte_size(&format_byte_size(512 * 1024)).unwrap(),
            512 * 1024
        );
    }
}
