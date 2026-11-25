//! Concurrent response cache used only by the transport adapter.

use super::Response;
use moka::sync::Cache;
use std::time::Duration;

/// The cache is an infrastructure detail: request construction and response
/// decoding remain pure, while expiration lives at this boundary.
pub(crate) struct Store(Cache<String, Response>);

impl Store {
    pub fn new(time_to_live: Duration) -> Self {
        Self(Cache::builder().time_to_live(time_to_live).build())
    }

    pub fn get(&self, id: &str) -> Option<Response> {
        self.0.get(id)
    }

    pub fn insert(&self, id: String, response: Response) {
        self.0.insert(id, response);
    }

    pub fn clear(&self) {
        self.0.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Store;
    use crate::transport::response::Response;

    #[test]
    fn clearing_drops_every_cached_response() {
        let store = Store::new(Duration::from_secs(60));
        store.insert("key".to_owned(), Response::new(b"body".to_vec()));
        assert!(store.get("key").is_some());

        store.clear();

        assert!(store.get("key").is_none());
    }
}
