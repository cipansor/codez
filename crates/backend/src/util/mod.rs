//! Port of `gitea.dev/modules/util`.
//!
//! One module per upstream file so the migration can be tracked file by file.
//! Ported so far: `path.go`, `string.go`, `truncate.go`, `slice.go`,
//! `paginate.go`, `bytesize.go`, `filename.go`, `error.go`, `url.go`.

pub mod bytesize;
pub mod error;
pub mod filename;
pub mod paginate;
pub mod path;
pub mod slice;
pub mod string;
pub mod truncate;
pub mod url;

pub use error::{ErrorKind, UserFacingError};
pub use path::{
    clean_posix_path, file_path_join_abs, path_base_stem, path_escape_segments, path_join_rel,
    path_join_rel_x,
};
pub use string::{normalize_eol, normalize_string_eol, to_snake_case};
pub use truncate::{ellipsis_display_string, truncate_runes, truncate_string_bytes};
