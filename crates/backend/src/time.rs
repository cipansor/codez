//! Time helpers.
//!
//! Gitea stamps every API response with RFC 3339 timestamps. We render them
//! from the system clock directly so the backend carries no date library.

use std::time::{SystemTime, UNIX_EPOCH};

/// Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Format a Unix timestamp as an RFC 3339 UTC string, e.g. `2026-10-20T15:54:47Z`.
pub fn rfc3339(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Seconds since the Unix epoch, or 0 if the clock is before it.
pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Current time as an RFC 3339 string.
pub fn now() -> String {
    rfc3339(now_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_formats_as_rfc3339() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339(1_000_000_000), "2001-09-09T01:46:40Z");
    }

    #[test]
    fn leap_days_are_handled() {
        // 2024-02-29 is day 19782 of the epoch.
        assert_eq!(rfc3339(19782 * 86_400), "2024-02-29T00:00:00Z");
        // 2000-02-29 (divisible by 400, unlike 1900).
        assert_eq!(rfc3339(11016 * 86_400), "2000-02-29T00:00:00Z");
    }
}
