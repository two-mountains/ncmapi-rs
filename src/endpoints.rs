//! Typed endpoint APIs grouped by domain.
//!
//! Each service owns a focused part of the NetEase protocol surface.

mod discovery;
mod library;
mod profile;

pub use discovery::{Discovery, SearchKind, SearchQuery};
pub use library::Library;
pub use profile::Profile;
