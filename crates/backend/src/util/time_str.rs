//! Port of `modules/util/time_str.go`.
//!
//! Parses and renders working-time estimates such as `1h 30m`.

use regex::Regex;
use std::sync::OnceLock;

/// `(h|m|s)` and their length in seconds, in descending order.
const UNITS: &[(&str, i64)] = &[("h", 3600), ("m", 60), ("s", 1)];

fn unit_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)(\d+)\s*([hms])").expect("static pattern"))
}

/// `util.TimeEstimateParse` — seconds from a string like `1h 30m`, or an error
/// describing why it is not a valid estimate.
///
/// Only hours, minutes, and seconds are accepted: converting a "day" would
/// depend on how many working hours a day has, which is configurable.
pub fn time_estimate_parse(time_str: &str) -> Result<i64, String> {
    let time_str = time_str.trim();
    if time_str.is_empty() {
        return Ok(0);
    }

    let re = unit_regex();
    let matches: Vec<_> = re.find_iter(time_str).collect();
    if matches.is_empty() {
        return Err(format!("invalid time string: {time_str}"));
    }
    // The matches must cover the whole string; a prefix or suffix is an error.
    if matches[0].start() != 0 || matches[matches.len() - 1].end() != time_str.len() {
        return Err(format!("invalid time string: {time_str}"));
    }

    let mut total: i64 = 0;
    let mut prev_end = 0usize;
    for m in &matches {
        // Only whitespace may separate two units, so "1h x 2m" is rejected.
        if !time_str[prev_end..m.start()].trim().is_empty() {
            return Err(format!("invalid time string: {time_str}"));
        }
        prev_end = m.end();

        let caps = re.captures(m.as_str()).expect("match implies captures");
        let amount: i64 = caps[1]
            .parse()
            .map_err(|e| format!("invalid time string: {e}"))?;
        let unit = &caps[2];

        match UNITS
            .iter()
            .find(|(name, _)| unit.eq_ignore_ascii_case(name))
        {
            Some((_, num)) => total += amount * num,
            None => return Err(format!("invalid time unit: {unit}")),
        }
    }
    Ok(total)
}

/// `util.TimeEstimateString` — renders seconds back into `1h 30m` form.
pub fn time_estimate_string(amount: i64) -> String {
    let mut remaining = amount;
    let mut parts = Vec::new();
    for (name, num) in UNITS {
        if remaining >= *num {
            let n = remaining / num;
            remaining %= num;
            parts.push(format!("{n}{name}"));
        }
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestTimeStr` in modules/util/time_str_test.go.
    #[test]
    fn time_estimate_parse_matches_upstream() {
        let cases: &[(&str, i64, bool)] = &[
            ("1h", 3600, false),
            ("1m", 60, false),
            ("1s", 1, false),
            ("1h 1m 1s", 3600 + 60 + 1, false),
            ("1d1x", 0, true),
            ("1h 2x 3m", 0, true),
            ("1h_2m", 0, true),
            ("1h,1m", 0, true),
        ];
        for (input, output, is_err) in cases {
            match time_estimate_parse(input) {
                Ok(v) => {
                    assert!(!is_err, "expected error for {input:?}");
                    assert_eq!(v, *output, "input {input:?}");
                }
                Err(_) => assert!(is_err, "unexpected error for {input:?}"),
            }
        }
    }

    #[test]
    fn time_estimate_string_matches_upstream() {
        let cases: &[(i64, &str)] = &[(3600, "1h"), (60, "1m"), (1, "1s"), (3600 + 1, "1h 1s")];
        for (input, output) in cases {
            assert_eq!(&time_estimate_string(*input), output);
        }
    }

    #[test]
    fn empty_input_is_zero() {
        assert_eq!(time_estimate_parse("").unwrap(), 0);
        assert_eq!(time_estimate_parse("   ").unwrap(), 0);
    }
}
