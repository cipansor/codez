//! Port of `gitea.dev/modules/container`.
//!
//! | upstream | here |
//! | --- | --- |
//! | `set.go` | [`set`] |
//! | `filter.go` | [`filter`] |

pub mod filter;
pub mod set;

pub use filter::filter_slice;
pub use set::Set;
