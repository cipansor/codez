//! The Gitea-compatible `/api/v1` surface.
//!
//! `routes.rs` and `ops.rs` are generated from the upstream OpenAPI document;
//! `dispatch` matches an incoming request against that table and hands it to
//! the resource family in the sibling modules.

pub mod actions;
pub mod admin;
pub mod dispatch;
pub mod issues;
pub mod misc;
pub mod ops;
pub mod orgs;
pub mod packages;
pub mod repos;
pub mod routes;
pub mod search;
pub mod users;
pub mod wiki;

pub use dispatch::ApiRequest;
#[cfg(test)]
pub use dispatch::QueryParams;
