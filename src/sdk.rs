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
    device_id_path: Option<PathBuf>,
    user_agent: Option<String>,
}

impl Default for NcmClientBuilder {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            cache_ttl: Duration::from_secs(3 * 60),
            persist_cookies: true,
            cookie_path: std::env::temp_dir().join("ncmapi-cookies"),
            device_id_path: None,
            user_agent: None,
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

    /// Persists the anonymous device id used by `eapi` request headers.
    ///
    /// Without this the transport generates a new id for every client, which
    /// makes each process look like a different device to the provider. The
    /// file is created with `0600` permissions when it does not exist.
    pub fn device_id_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.device_id_path = Some(path.into());
        self
    }

    /// Overrides the `User-Agent` sent with every request.
    ///
    /// The default is the SDK identifier. Applications that talk to the web
    /// endpoints can pass a browser string to match a real client more closely.
    ///
    /// # Errors
    ///
    /// The value is validated in [`NcmClientBuilder::build`]; a string that is
    /// not a valid header value is rejected there.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Creates the client and reports invalid local configuration eagerly.
    pub fn build(self) -> TResult<NcmClient> {
        let mut transport = HttpTransportBuilder::new(&self.cookie_path)?
            .cache(self.cache_enabled)
            .cache_ttl(self.cache_ttl)
            .persist_cookies(self.persist_cookies);
        if let Some(path) = self.device_id_path {
            transport = transport.device_id_path(path);
        }
        if let Some(user_agent) = self.user_agent {
            transport = transport.user_agent(user_agent);
        }
        let transport = transport.build()?;
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
