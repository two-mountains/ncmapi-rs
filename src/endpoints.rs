//! Typed endpoint APIs grouped by domain.
//!
//! Each service owns a focused part of the NetEase protocol surface.

mod auth;
mod discovery;
mod library;
mod profile;
mod radio;

pub use auth::Auth;
pub use discovery::{Discovery, SearchKind, SearchQuery};
pub use library::Library;
pub use profile::Profile;
pub use radio::Radio;
