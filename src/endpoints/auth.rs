//! QR-code login.
//!
//! The flow is two calls: [`Auth::qr_start`] returns a key, which the user
//! encodes in a QR code; [`Auth::qr_poll`] is then polled until the user
//! confirms in the NetEase mobile app. Confirmation sets the session cookies,
//! which the transport persists for later requests.

use serde_json::json;

use crate::{
    ApiResponse, Error, NcmClient, QrKeyBody, QrLoginBody, Result,
    transport::{QR_LOGIN, QR_UNIKEY, RequestPlanBuilder},
};

/// Login endpoints exposed by [`NcmClient::auth`].
pub struct Auth<'client> {
    client: &'client NcmClient,
}

impl NcmClient {
    pub fn auth(&self) -> Auth<'_> {
        Auth { client: self }
    }
}

impl Auth<'_> {
    /// Starts a QR login and returns the key to render.
    pub async fn qr_start(&self) -> Result<ApiResponse<QrKeyBody>> {
        let request = RequestPlanBuilder::post(QR_UNIKEY)
            .payload(json!({ "type": 1 }))
            .build();
        self.client.execute(request).await?.decode()
    }

    /// Polls the state of a pending QR login.
    ///
    /// Codes: `800` expired, `801` waiting, `802` scanned, `803` confirmed.
    ///
    /// # Errors
    ///
    /// Rejects a blank key before any request is made.
    pub async fn qr_poll(&self, key: &str) -> Result<ApiResponse<QrLoginBody>> {
        if key.trim().is_empty() {
            return Err(Error::InvalidInput {
                field: "qr key",
                reason: "must not be blank",
            });
        }
        let request = RequestPlanBuilder::post(QR_LOGIN)
            .payload(json!({ "key": key, "type": 1 }))
            .build();
        self.client.execute(request).await?.decode()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::{
        NcmClient, Result,
        transport::{RequestPlan, Response, Transport},
    };

    struct StubTransport {
        body: &'static str,
    }

    #[async_trait]
    impl Transport for StubTransport {
        async fn execute(&self, _: RequestPlan) -> Result<Response> {
            Ok(Response::new(self.body.as_bytes().to_vec()))
        }
    }

    fn client(body: &'static str) -> NcmClient {
        NcmClient::with_transport(Arc::new(StubTransport { body }))
    }

    #[tokio::test]
    async fn qr_start_returns_a_key_and_login_url() {
        let response = client(r#"{"code":200,"unikey":"abc123"}"#)
            .auth()
            .qr_start()
            .await
            .unwrap();
        assert_eq!(response.body.unikey, "abc123");
        assert_eq!(
            response.body.login_url(),
            "https://music.163.com/login?codekey=abc123"
        );
    }

    #[tokio::test]
    async fn qr_poll_decodes_state_codes() {
        let response = client(r#"{"code":803,"nickname":"ben"}"#)
            .auth()
            .qr_poll("abc123")
            .await
            .unwrap();
        assert_eq!(response.code, 803);
        assert_eq!(response.body.nickname.as_deref(), Some("ben"));
    }

    #[tokio::test]
    async fn blank_keys_are_rejected() {
        let client = client("{}");
        assert!(client.auth().qr_poll(" ").await.is_err());
    }
}
