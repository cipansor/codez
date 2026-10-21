//! Codez backend — a Rust port of Gitea.
//!
//! Modules mirror the upstream Go layout one file at a time so migration
//! progress is trackable; see `MIGRATION.md`.

pub mod cachegroup;
pub mod consts;
pub mod container;
pub mod regexplru;
pub mod secret;
pub mod tempdir;
pub mod uri;
pub mod user;
pub mod util;
