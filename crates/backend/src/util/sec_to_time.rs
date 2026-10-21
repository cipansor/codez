//! Port of `modules/util/sec_to_time.go`.

/// `util.SecToHours` — seconds rendered as hours and minutes for timesheets.
///
/// Only hours and minutes are supported: a "work day" is 6, 7, or 8 hours
/// depending on configuration, so days are deliberately not converted. A
/// duration under a minute falls back to seconds.
pub fn sec_to_hours(duration: i64) -> String {
    let hours = duration / 3600;
    let minutes = (duration / 60) % 60;

    let mut formatted = String::new();
    formatted = format_time(hours, "hour", &formatted);
    formatted = format_time(minutes, "minute", &formatted);

    if formatted.is_empty() && duration > 0 {
        formatted = format_time(duration, "second", "");
    }
    formatted.trim_end_matches(' ').to_string()
}

/// `util.formatTime` — appends `"<value> <name>(s) "` when `value` is non-zero.
fn format_time(value: i64, name: &str, formatted: &str) -> String {
    if value == 1 {
        format!("{formatted}1 {name} ")
    } else if value > 1 {
        format!("{formatted}{value} {name}s ")
    } else {
        formatted.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestSecToHours` in modules/util/sec_to_time_test.go.
    #[test]
    fn sec_to_hours_matches_upstream() {
        let second: i64 = 1;
        let minute = 60 * second;
        let hour = 60 * minute;
        let day = 24 * hour;

        assert_eq!(sec_to_hours(minute + 6 * second), "1 minute");
        assert_eq!(sec_to_hours(hour), "1 hour");
        assert_eq!(sec_to_hours(hour + second), "1 hour");
        assert_eq!(
            sec_to_hours(14 * hour + 33 * minute + 30 * second),
            "14 hours 33 minutes"
        );
        assert_eq!(
            sec_to_hours(6 * day + 12 * hour + 30 * minute + 18 * second),
            "156 hours 30 minutes"
        );
        assert_eq!(
            sec_to_hours(4 * day + 2 * hour + 16 * minute + 58 * second),
            "98 hours 16 minutes"
        );
        assert_eq!(sec_to_hours(4 * 7 * day), "672 hours");
        assert_eq!(sec_to_hours(1), "1 second");
        assert_eq!(sec_to_hours(2), "2 seconds");
        // Zero yields an empty string, matching the old behaviour.
        assert_eq!(sec_to_hours(0), "");
    }
}
