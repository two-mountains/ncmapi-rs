use std::{path::PathBuf, sync::Arc, time::Duration};

use crate::{
    TResult,
    transport::{HttpTransportBuilder, RequestPlan, Response, Transport},
};

/// The public entry point for the Netease Cloud Music SDK.
///
/// Construct it with [`NcmClient::builder`]. Endpoint APIs are exposed as
/// focused services, while this type owns the shared transport boundary.
pub struct NcmClient {
    transport: Arc<dyn Transport>,
}

/// Builds an [`NcmClient`] from explicit infrastructure choices.
#[derive(Clone, Debug)]
pub struct NcmClientBuilder {
    cache_enabled: bool,
    cache_ttl: Duration,
    persist_cookies: bool,
    cookie_path: PathBuf,
}

impl Default for NcmClientBuilder {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            cache_ttl: Duration::from_secs(3 * 60),
            persist_cookies: true,
            cookie_path: std::env::temp_dir().join("ncmapi-cookies"),
        }
    }
}

impl NcmClientBuilder {
    /// Disables or enables the short-lived response cache.
    pub fn cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }

    /// Sets the lifetime of cached responses.
    pub fn cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Chooses whether cookies received from the service are persisted.
    pub fn persist_cookies(mut self, enabled: bool) -> Self {
        self.persist_cookies = enabled;
        self
    }

    /// Sets the file used to persist cookies between client instances.
    pub fn cookie_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.cookie_path = path.into();
        self
    }

    /// Creates the client and reports invalid local configuration eagerly.
    pub fn build(self) -> TResult<NcmClient> {
        let transport = HttpTransportBuilder::new(&self.cookie_path)?
            .cache(self.cache_enabled)
            .cache_ttl(self.cache_ttl)
            .persist_cookies(self.persist_cookies)
            .build()?;
        Ok(NcmClient {
            transport: Arc::new(transport),
        })
    }
}

impl NcmClient {
    pub fn builder() -> NcmClientBuilder {
        NcmClientBuilder::default()
    }

    pub(crate) async fn execute(&self, request: RequestPlan) -> TResult<Response> {
        self.transport.execute(request).await
    }

    #[cfg(test)]
    pub(crate) fn with_transport(transport: Arc<dyn Transport>) -> Self {
        Self { transport }
    }
}
