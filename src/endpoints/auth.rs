//! QR-code login.
//!
//! The flow is two calls: [`Auth::qr_start`] returns a key, which the user
//! encodes in a QR code; [`Auth::qr_poll`] is then polled until the user
//! confirms in the NetEase mobile app. Confirmation sets the session cookies,
//! which the transport persists for later requests.

use serde_json::json;

use crate::{
    ApiResponse, Error, NcmClient, QrKeyBody, QrLoginBody, Result,
    crypto::Crypto,
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
    ///
    /// Uses the new-client protocol (`eapi`, `type: 3`). The legacy weapi
    /// `type: 1` flow is rejected with `8821` (“behavioral verification”) once
    /// the user confirms, because it lacks the client fingerprint.
    pub async fn qr_start(&self) -> Result<ApiResponse<QrKeyBody>> {
        let request = RequestPlanBuilder::post(QR_UNIKEY)
            .encryption(Crypto::Eapi)
            .api_path("/api/login/qrcode/unikey")
            .payload(json!({ "type": 3, "noCheckToken": true }))
            .uncacheable()
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
            .encryption(Crypto::Eapi)
            .api_path("/api/login/qrcode/client/login")
            .payload(json!({ "key": key, "type": 3, "noCheckToken": true }))
            .uncacheable()
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
        url: &'static str,
        body: &'static str,
    }

    #[async_trait]
    impl Transport for StubTransport {
        async fn execute(&self, plan: RequestPlan) -> Result<Response> {
            let parts = plan.into_parts();
            assert_eq!(parts.url, self.url);
            assert_eq!(parts.encryption, crate::crypto::Crypto::Eapi);
            assert_eq!(parts.payload["type"], 3);
            Ok(Response::new(self.body.as_bytes().to_vec()))
        }
    }

    const UNIKEY: &str = "https://music.163.com/eapi/login/qrcode/unikey";
    const CHECK: &str = "https://music.163.com/eapi/login/qrcode/client/login";

    fn client(url: &'static str, body: &'static str) -> NcmClient {
        NcmClient::with_transport(Arc::new(StubTransport { url, body }))
    }

    #[tokio::test]
    async fn qr_start_returns_a_key_and_login_url() {
        let response = client(UNIKEY, r#"{"code":200,"unikey":"abc123"}"#)
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
        let response = client(CHECK, r#"{"code":803,"nickname":"ben"}"#)
            .auth()
            .qr_poll("abc123")
            .await
            .unwrap();
        assert_eq!(response.code, 803);
        assert_eq!(response.body.nickname.as_deref(), Some("ben"));
    }

    #[tokio::test]
    async fn blank_keys_are_rejected() {
        let client = client(CHECK, "{}");
        assert!(client.auth().qr_poll(" ").await.is_err());
    }
}
