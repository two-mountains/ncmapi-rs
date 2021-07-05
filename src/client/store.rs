use memory_cache::MemoryCache;

use super::ApiResponse;
use std::{sync::RwLock, time};

pub(crate) trait InMemStore {
    fn get(&self, id: &String) -> Option<ApiResponse>;
    fn contains_key(&self, id: &String) -> bool;
    fn insert(
        &self,
        id: String,
        val: ApiResponse,
        lifetime: Option<time::Duration>,
    ) -> Option<ApiResponse>;
}

pub(crate) struct Store(RwLock<MemoryCache<String, ApiResponse>>);

impl Store {
    pub fn new(scan_interval: time::Duration) -> Self {
        Self(RwLock::new(MemoryCache::with_full_scan(scan_interval)))
    }
}

impl InMemStore for Store {
    fn get(&self, id: &String) -> Option<ApiResponse> {
        if let Some(res) = self.0.read().unwrap().get(id) {
            return Some(ApiResponse::new(res.data().to_owned()));
        }
        None
    }

    fn contains_key(&self, id: &String) -> bool {
        self.0.read().unwrap().contains_key(id)
    }

    fn insert(
        &self,
        id: String,
        val: ApiResponse,
        lifetime: Option<time::Duration>,
    ) -> Option<ApiResponse> {
        self.0.write().unwrap().insert(id, val, lifetime)
    }
}
