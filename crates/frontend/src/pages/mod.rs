//! Page components, one module per feature area.

pub mod actions;
pub mod admin;
pub mod compare;
pub mod dashboard;
pub mod org;
pub mod packages;
pub mod release;
pub mod repo;
pub mod user;

pub use actions::*;
pub use admin::*;
pub use compare::*;
pub use dashboard::*;
pub use org::*;
pub use packages::*;
pub use release::*;
pub use repo::*;
pub use user::*;
