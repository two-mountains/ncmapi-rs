//! Raw response decoding at the transport boundary.

use serde::de::DeserializeOwned;
use std::fmt::{self, Display};

#[derive(Clone)]
pub(crate) struct Response {
    data: Vec<u8>,
}

impl Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.data()))
    }
}

impl Response {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub(crate) fn data(&self) -> &[u8] {
        &self.data
    }

    /// Deserializes the response body without hiding malformed remote data.
    pub(crate) fn decode<T>(&self) -> crate::Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(serde_json::from_slice(self.data())?)
    }
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("data", &self.to_string())
            .finish()
    }
}
