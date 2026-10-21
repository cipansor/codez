//! Port of `modules/consts/asymkey.go`.

/// `consts.AsymKeyMinBitsRsa` — 3072 minus one, to tolerate a leading zero.
pub const ASYM_KEY_MIN_BITS_RSA: u32 = 3071;

/// `consts.AsymKeyMinBitsEC`.
pub const ASYM_KEY_MIN_BITS_EC: u32 = 256;

/// `consts.AsymKeyDefaultBitsRsa` — `ssh-keygen` defaults to 3072, Gitea to 4096.
pub const ASYM_KEY_DEFAULT_BITS_RSA: u32 = 4096;

/// `consts.AsymKeyDefaultBitsEcdsa`.
pub const ASYM_KEY_DEFAULT_BITS_ECDSA: u32 = 256;

/// Compile-time invariants tying the minimums to the defaults.
const _: () = {
    assert!(
        ASYM_KEY_MIN_BITS_RSA == 3071,
        "tolerates a leading zero byte"
    );
    assert!(ASYM_KEY_DEFAULT_BITS_RSA > ASYM_KEY_MIN_BITS_RSA);
    assert!(ASYM_KEY_DEFAULT_BITS_ECDSA >= ASYM_KEY_MIN_BITS_EC);
};
