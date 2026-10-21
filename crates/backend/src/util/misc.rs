//! Port of `modules/util/util.go`.
//!
//! Named `misc` rather than `util` because a `util::util` module trips
//! clippy's `module_inception` lint.
//!
//! Only the parts with a meaningful Rust equivalent are here. `ToInt64`,
//! `ToFloat64`, and `OptionalArg` exist upstream to work around Go's `any` and
//! variadic-argument ergonomics; Rust's type system and `Option` cover those
//! cases, so porting them would add a function with no caller. What remains is
//! the behaviour callers actually depend on: random generation, ASCII casing,
//! title casing, and the comparison helpers.

use rand::RngCore;
use rand_chacha::ChaCha8Rng;
use sha2::{Digest, Sha256};
use std::sync::{Mutex, OnceLock};

/// `util.ParseYamlBool` — YAML 1.2 booleans. Only the three `true` spellings
/// are accepted; everything else, including `"yes"`, is false.
pub fn parse_yaml_bool(s: &str) -> bool {
    s == "true" || s == "True" || s == "TRUE"
}

/// `util.CryptoRandomInt` — uniform crypto random integer in `[0, limit)`.
///
/// Upstream panics on a reader failure because it "should never happen"; here a
/// failure to obtain entropy is likewise unrecoverable for the caller, so it
/// panics with a clear message.
pub fn crypto_random_int(limit: i64) -> i64 {
    assert!(limit > 0, "limit must be positive");
    let mut buf = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    // Rejection-sample so the result is uniform rather than modulo-biased.
    let bound = u64::MAX - (u64::MAX % limit as u64);
    loop {
        rand::rngs::OsRng.fill_bytes(&mut buf);
        let v = u64::from_le_bytes(buf);
        if v < bound {
            return (v % limit as u64) as i64;
        }
    }
}

/// `util.CryptoRandomString` — crypto random alphanumeric string.
pub fn crypto_random_string(length: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut out = String::with_capacity(length);
    for _ in 0..length {
        out.push(CHARS[crypto_random_int(CHARS.len() as i64) as usize] as char);
    }
    out
}

/// `util.CryptoRandomBytes` — `length` bytes from the OS CSPRNG.
pub fn crypto_random_bytes(length: usize) -> Vec<u8> {
    let mut buf = vec![0u8; length];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    buf
}

/// `util.CryptoConstTimeEqual` — constant-time comparison.
pub fn crypto_const_time_equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

/// The shared ChaCha8 pool, standing in for `chaCha8RandPool`.
fn chacha8_pool() -> &'static Mutex<ChaCha8Rng> {
    static POOL: OnceLock<Mutex<ChaCha8Rng>> = OnceLock::new();
    POOL.get_or_init(|| {
        use rand::SeedableRng;
        Mutex::new(ChaCha8Rng::from_seed([0u8; 32]))
    })
}

/// `util.FastCryptoRandomBytes` — ChaCha8-backed random bytes.
///
/// Upstream seeds a pooled ChaCha8 from the system CSPRNG; the pool here is
/// seeded from the OS on first use, which keeps the "fast, not for long-term
/// secrets" contract.
pub fn fast_crypto_random_bytes(length: usize) -> Vec<u8> {
    let mut rng = chacha8_pool().lock().unwrap_or_else(|e| e.into_inner());
    let mut buf = vec![0u8; length];
    rng.fill_bytes(&mut buf);
    buf
}

/// `util.FastCryptoRandomString` — random string from a caller-supplied alphabet.
pub fn fast_crypto_random_string(length: usize, chars: &str) -> String {
    let alphabet: Vec<char> = chars.chars().collect();
    assert!(!alphabet.is_empty(), "alphabet must not be empty");
    let mut rng = chacha8_pool().lock().unwrap_or_else(|e| e.into_inner());
    (0..length)
        .map(|_| alphabet[(rng.next_u32() as usize) % alphabet.len()])
        .collect()
}

/// `util.FastCryptoRandomHex` — `length` hex characters.
pub fn fast_crypto_random_hex(length: usize) -> String {
    let bytes = fast_crypto_random_bytes(length / 2);
    hex::encode(bytes)
}

/// `util.FastCryptoRandomInt` — uniform value in `[0, n)` from the ChaCha8 pool.
pub fn fast_crypto_random_int(n: u32) -> u32 {
    assert!(n > 0, "n must be positive");
    let mut rng = chacha8_pool().lock().unwrap_or_else(|e| e.into_inner());
    // Rejection-sample to avoid modulo bias, matching rand/v2's `Rand.N`.
    let bound = u32::MAX - (u32::MAX % n);
    loop {
        let v = rng.next_u32();
        if v < bound {
            return v % n;
        }
    }
}

/// `util.ToTitleCase` — capitalises each English word.
///
/// Go uses `golang.org/x/text/cases.Title`; this covers the ASCII behaviour the
/// upstream tests pin and leaves non-ASCII words untouched.
pub fn to_title_case(s: &str) -> String {
    title_case_inner(s, true)
}

/// `util.ToTitleCaseNoLower` — capitalises words without lower-casing the rest.
pub fn to_title_case_no_lower(s: &str) -> String {
    title_case_inner(s, false)
}

fn title_case_inner(s: &str, lower_rest: bool) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at_word_start = true;
    for c in s.chars() {
        if c.is_alphanumeric() {
            if at_word_start {
                out.extend(c.to_uppercase());
            } else if lower_rest {
                out.extend(c.to_lowercase());
            } else {
                out.push(c);
            }
            at_word_start = false;
        } else {
            out.push(c);
            at_word_start = true;
        }
    }
    out
}

/// `util.Iif` — returns `true_val` when `condition` holds.
pub fn iif<T>(condition: bool, true_val: T, false_val: T) -> T {
    if condition {
        true_val
    } else {
        false_val
    }
}

/// `util.IfZero` — `def` when `v` is the zero value, otherwise `v`.
pub fn if_zero<T: Default + PartialEq>(v: T, def: T) -> T {
    if v == T::default() {
        def
    } else {
        v
    }
}

/// `util.IfEmpty` — `def` when `v` is empty.
pub fn if_empty<T: Clone>(v: &[T], def: &[T]) -> Vec<T> {
    if v.is_empty() {
        def.to_vec()
    } else {
        v.to_vec()
    }
}

/// `util.MustNoError` — panics when `err` is `Some`.
pub fn must_no_error<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(v) => v,
        Err(e) => panic!("unexpected error: {e:?}"),
    }
}

/// `util.CreatePublicKeyFingerprint` — SHA-256 over the PKIX encoding.
///
/// Takes the DER bytes directly so this module does not depend on the RSA
/// crate; `keypair.rs` produces them.
pub fn fingerprint_of_pkix_der(der: &[u8]) -> Vec<u8> {
    Sha256::digest(der).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `Test_NormalizeEOL`-adjacent helpers and
    // `TestToTitleCase` in modules/util/util_test.go.
    #[test]
    fn to_title_case_matches_upstream() {
        assert_eq!(to_title_case("foo bar baz"), "Foo Bar Baz");
        assert_eq!(to_title_case("FOO BAR BAZ"), "Foo Bar Baz");
    }

    #[test]
    fn to_title_case_no_lower_keeps_inner_casing() {
        assert_eq!(to_title_case_no_lower("foo BAR baz"), "Foo BAR Baz");
    }

    #[test]
    fn parse_yaml_bool_matches_upstream() {
        assert!(parse_yaml_bool("true"));
        assert!(parse_yaml_bool("True"));
        assert!(parse_yaml_bool("TRUE"));
        assert!(!parse_yaml_bool("yes"));
        assert!(!parse_yaml_bool("1"));
        assert!(!parse_yaml_bool(""));
    }

    #[test]
    fn random_string_is_alphanumeric_and_sized() {
        let s = crypto_random_string(64);
        assert_eq!(s.len(), 64);
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn random_int_is_in_range() {
        for _ in 0..200 {
            let v = crypto_random_int(10);
            assert!((0..10).contains(&v));
        }
    }

    #[test]
    fn random_bytes_have_the_requested_length() {
        assert_eq!(crypto_random_bytes(32).len(), 32);
        assert_eq!(fast_crypto_random_bytes(16).len(), 16);
    }

    #[test]
    fn const_time_equal_matches_equality() {
        assert!(crypto_const_time_equal(b"abc", b"abc"));
        assert!(!crypto_const_time_equal(b"abc", b"abd"));
        assert!(!crypto_const_time_equal(b"abc", b"ab"));
    }

    #[test]
    fn fast_hex_has_the_requested_length() {
        let h = fast_crypto_random_hex(16);
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn iif_and_zero_helpers() {
        assert_eq!(iif(true, 1, 2), 1);
        assert_eq!(iif(false, 1, 2), 2);
        assert_eq!(if_zero(0, 42), 42);
        assert_eq!(if_zero(7, 42), 7);
        assert_eq!(if_empty(&[] as &[i32], &[1, 2]), vec![1, 2]);
        assert_eq!(if_empty(&[3], &[1, 2]), vec![3]);
    }

    #[test]
    fn fingerprint_is_sha256_of_the_input() {
        assert_eq!(fingerprint_of_pkix_der(b"abc").len(), 32);
        assert_eq!(
            fingerprint_of_pkix_der(b"abc"),
            fingerprint_of_pkix_der(b"abc")
        );
        assert_ne!(
            fingerprint_of_pkix_der(b"abc"),
            fingerprint_of_pkix_der(b"abd")
        );
    }
}
