use super::ApiResponse;
use moka::sync::Cache;
use std::time::Duration;

/// The cache is an infrastructure detail: request construction and response
/// decoding remain pure, while expiration lives at this boundary.
pub(crate) struct Store(Cache<String, ApiResponse>);

impl Store {
    pub fn new(time_to_live: Duration) -> Self {
        Self(Cache::builder().time_to_live(time_to_live).build())
    }

    pub fn get(&self, id: &str) -> Option<ApiResponse> {
        self.0.get(id)
    }

    pub fn insert(&self, id: String, response: ApiResponse) {
        self.0.insert(id, response);
    }
}
