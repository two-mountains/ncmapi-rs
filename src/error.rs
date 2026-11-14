use std::io;

use thiserror::Error;

/// Errors returned by the SDK boundary.
///
/// The variants retain their original causes so callers can distinguish a
/// malformed local configuration from a remote HTTP failure.
#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP transport failed")]
    Http(#[source] reqwest::Error),

    #[error("cookie persistence failed while {operation}")]
    CookiePersistence {
        operation: &'static str,
        #[source]
        source: io::Error,
    },

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("invalid {field}: {reason}")]
    InvalidInput {
        field: &'static str,
        reason: &'static str,
    },

    #[error("protocol invariant violated: {0}")]
    Protocol(String),

    #[error("cryptographic operation failed: {0}")]
    Crypto(#[from] crate::crypto::CryptoError),

    #[error("JSON serialization or deserialization failed")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
