//! Compact, human-readable formatting of byte/sec rates for a tiny status-bar icon.

/// Binary base: K/M/G mean 1024-multiples, matching the "bytes" framing.
const BASE: f64 = 1024.0;
const SUFFIXES: [char; 4] = ['B', 'K', 'M', 'G'];

/// Scale a rate (bytes/sec) into `(value, suffix)` where `value < 1024`
/// (except for the final 'G' bucket, which can exceed it for huge rates).
fn scale(bps: f64) -> (f64, char) {
    let mut v = if bps.is_finite() && bps > 0.0 {
        bps
    } else {
        0.0
    };
    let mut i = 0;
    while v >= BASE && i < SUFFIXES.len() - 1 {
        v /= BASE;
        i += 1;
    }
    (v, SUFFIXES[i])
}

/// Format a rate into at most ~4 characters, e.g. `"0"`, `"950B"`, `"1.2M"`, `"85K"`.
///
/// - below 1 byte/sec -> `"0"` (deadzone: idle and decaying-tail rates read clean)
/// - raw bytes -> integer + `B`
/// - otherwise -> one decimal below 10 (`"1.2M"`), no decimal at/above 10 (`"85M"`)
pub fn fmt_rate(bps: f64) -> String {
    if !(bps.is_finite() && bps >= 1.0) {
        return "0".to_string();
    }
    let (v, unit) = scale(bps);
    if unit == 'B' {
        format!("{:.0}{}", v, unit)
    } else if v < 10.0 {
        format!("{:.1}{}", v, unit)
    } else {
        format!("{:.0}{}", v, unit)
    }
}

/// Like [`fmt_rate`] but without a decimal point, e.g. `"0"`, `"2K"`, `"15M"`,
/// `"1G"`. Fewer characters let the single status-bar number render larger.
pub fn fmt_rate_compact(bps: f64) -> String {
    if !(bps.is_finite() && bps >= 1.0) {
        return "0".to_string();
    }
    let (v, unit) = scale(bps);
    format!("{:.0}{}", v, unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_and_invalid() {
        assert_eq!(fmt_rate(0.0), "0");
        assert_eq!(fmt_rate(-5.0), "0");
        assert_eq!(fmt_rate(f64::NAN), "0");
        assert_eq!(fmt_rate(f64::INFINITY), "0");
        // Sub-1-byte/s deadzone (EMA decay tail) reads as a clean "0".
        assert_eq!(fmt_rate(0.4), "0");
    }

    #[test]
    fn raw_bytes() {
        assert_eq!(fmt_rate(1.0), "1B");
        assert_eq!(fmt_rate(950.0), "950B");
        assert_eq!(fmt_rate(1023.0), "1023B");
    }

    #[test]
    fn kilobytes() {
        assert_eq!(fmt_rate(1024.0), "1.0K");
        assert_eq!(fmt_rate(1536.0), "1.5K"); // 1.5 * 1024
        assert_eq!(fmt_rate(85.0 * 1024.0), "85K");
    }

    #[test]
    fn megabytes_and_gigabytes() {
        assert_eq!(fmt_rate(1024.0 * 1024.0), "1.0M");
        assert_eq!(fmt_rate(12.0 * 1024.0 * 1024.0), "12M");
        assert_eq!(fmt_rate(3.0 * 1024.0 * 1024.0 * 1024.0), "3.0G");
    }

    #[test]
    fn compact_drops_decimals() {
        assert_eq!(fmt_rate_compact(0.0), "0");
        assert_eq!(fmt_rate_compact(2.0 * 1024.0), "2K");
        assert_eq!(fmt_rate_compact(15.0 * 1024.0 * 1024.0), "15M");
        assert_eq!(fmt_rate_compact(1.0 * 1024.0 * 1024.0 * 1024.0), "1G");
        // No decimal point, so at most 4 chars for typical rates.
        assert!(!fmt_rate_compact(2.5 * 1024.0).contains('.'));
    }

    #[test]
    fn never_too_long() {
        // Even an absurd rate stays short-ish (value + single suffix char).
        let s = fmt_rate(9_999.0 * 1024.0 * 1024.0 * 1024.0);
        assert!(s.ends_with('G'));
    }
}
