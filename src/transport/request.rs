//! Immutable HTTP request plans.
//!
//! Endpoint services describe intent with this value type. The transport is
//! responsible for the effectful work of encrypting and sending that plan.

use std::collections::BTreeMap;

use md5::{Digest, Md5};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{TResult, crypto::Crypto};

#[derive(Debug, Serialize)]
pub(crate) struct RequestPlan {
    url: String,
    payload: Value,
    cookies: BTreeMap<String, String>,
    encryption: Crypto,
    api_path: Option<String>,
    /// Read-only requests may be served from the short-lived response cache.
    /// Mutations, polling endpoints and non-idempotent reads must not be.
    cacheable: bool,
}

pub(crate) struct RequestParts {
    pub(crate) url: String,
    pub(crate) payload: Value,
    pub(crate) cookies: BTreeMap<String, String>,
    pub(crate) encryption: Crypto,
    pub(crate) api_path: Option<String>,
}

/// Consuming builder for a [`RequestPlan`].
pub(crate) struct RequestPlanBuilder(RequestPlan);

impl RequestPlanBuilder {
    pub(crate) fn post(url: impl Into<String>) -> Self {
        Self(RequestPlan {
            url: url.into(),
            payload: json!({}),
            cookies: BTreeMap::new(),
            encryption: Crypto::Weapi,
            api_path: None,
            cacheable: true,
        })
    }

    pub(crate) fn payload(mut self, payload: Value) -> Self {
        self.0.payload = payload;
        self
    }

    pub(crate) fn cookie(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.0.cookies.insert(name.into(), value.into());
        self
    }

    pub(crate) fn encryption(mut self, encryption: Crypto) -> Self {
        self.0.encryption = encryption;
        self
    }

    pub(crate) fn api_path(mut self, path: impl Into<String>) -> Self {
        self.0.api_path = Some(path.into());
        self
    }

    /// Opts the plan out of the response cache.
    ///
    /// Use for anything that changes server state or whose answer is expected
    /// to change between identical calls (QR polling, FM batches, likes).
    pub(crate) fn uncacheable(mut self) -> Self {
        self.0.cacheable = false;
        self
    }

    pub(crate) fn build(self) -> RequestPlan {
        self.0
    }
}

impl RequestPlan {
    pub(crate) fn id(&self) -> TResult<String> {
        let serialized = serde_json::to_vec(self)?;
        Ok(hex::encode(Md5::digest(&serialized)))
    }

    pub(crate) const fn cacheable(&self) -> bool {
        self.cacheable
    }

    pub(crate) fn into_parts(self) -> RequestParts {
        RequestParts {
            url: self.url,
            payload: self.payload,
            cookies: self.cookies,
            encryption: self.encryption,
            api_path: self.api_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::RequestPlanBuilder;
    use crate::crypto::Crypto;

    #[test]
    fn plans_are_cacheable_by_default_and_can_opt_out() {
        let cached = RequestPlanBuilder::post("https://example.test/a").build();
        assert!(cached.cacheable());

        let uncached = RequestPlanBuilder::post("https://example.test/a")
            .uncacheable()
            .build();
        assert!(!uncached.cacheable());
    }

    #[test]
    fn request_identity_is_stable_for_equal_plans() {
        let first = RequestPlanBuilder::post("https://example.test/api/search")
            .payload(json!({"keywords": "mota"}))
            .cookie("sid", "value")
            .encryption(Crypto::Eapi)
            .api_path("/api/search")
            .build();
        let second = RequestPlanBuilder::post("https://example.test/api/search")
            .payload(json!({"keywords": "mota"}))
            .cookie("sid", "value")
            .encryption(Crypto::Eapi)
            .api_path("/api/search")
            .build();

        assert_eq!(first.id().unwrap(), second.id().unwrap());
    }
}
