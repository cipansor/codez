//! Port of `gitea.dev/modules/util`.
//!
//! One module per upstream file so the migration can be tracked file by file.
//!
//! | upstream | here |
//! | --- | --- |
//! | `path.go` | [`path`] |
//! | `string.go` | [`string`] |
//! | `truncate.go` | [`truncate`] |
//! | `slice.go` | [`slice`] |
//! | `paginate.go` | [`paginate`] |
//! | `bytesize.go` | [`bytesize`] |
//! | `filename.go` | [`filename`] |
//! | `error.go`, `sanitize.go` | [`error`] |
//! | `url.go` | [`url`] |
//! | `buffer.go` | [`buffer`] |
//! | `color.go` | [`color`] |
//! | `file_retry.go` | [`file_retry`] |
//! | `file_unix.go`, `file_windows.go` | [`file_mode`] |
//! | `io.go` | [`io`] |
//! | `keypair.go` | [`keypair`] |
//! | `legacy.go` | [`legacy`] |
//! | `pack.go` | [`pack`] |
//! | `runtime.go` | [`runtime`] |
//! | `sec_to_time.go` | [`sec_to_time`] |
//! | `shellquote.go` | [`shellquote`] |
//! | `sync.go` | [`sync`] |
//! | `time_str.go` | [`time_str`] |
//! | `timer.go` | [`timer`] |
//! | `trie.go` | [`trie`] |
//! | `util.go` | [`misc`] |

pub mod buffer;
pub mod bytesize;
pub mod color;
pub mod error;
pub mod file_mode;
pub mod file_retry;
pub mod filename;
pub mod io;
pub mod keypair;
pub mod legacy;
pub mod misc;
pub mod pack;
pub mod paginate;
pub mod path;
pub mod runtime;
pub mod sec_to_time;
pub mod shellquote;
pub mod slice;
pub mod string;
pub mod sync;
pub mod time_str;
pub mod timer;
pub mod trie;
pub mod truncate;
pub mod url;

pub use error::{ErrorKind, UserFacingError};
pub use path::{
    clean_posix_path, file_path_join_abs, path_base_stem, path_escape_segments, path_join_rel,
    path_join_rel_x,
};
pub use string::{normalize_eol, normalize_string_eol, to_snake_case};
pub use truncate::{ellipsis_display_string, truncate_runes, truncate_string_bytes};
