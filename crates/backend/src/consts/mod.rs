//! Port of `gitea.dev/modules/consts`.
//!
//! | upstream | here |
//! | --- | --- |
//! | `runtime.go` | [`is_windows`] |
//! | `asymkey.go` | [`asymkey`] |

pub mod asymkey;
pub mod is_windows;

pub use asymkey::{
    ASYM_KEY_DEFAULT_BITS_ECDSA, ASYM_KEY_DEFAULT_BITS_RSA, ASYM_KEY_MIN_BITS_EC,
    ASYM_KEY_MIN_BITS_RSA,
};
pub use is_windows::IS_WINDOWS;
