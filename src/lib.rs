//! NetEase Cloud Music API For Rust.

mod api;
mod client;
mod crypto;
mod domain;
pub mod types;

pub use api::{NcmClient, NcmClientBuilder, NcmClientConfig, ResourceType, SearchType};
#[deprecated(note = "use NcmClient instead")]
pub type NcmApi = NcmClient;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiErr {
    #[error("reqwest error")]
    ReqwestErr,

    #[error("deserialize error")]
    DeserializeErr,

    #[error("parse url error")]
    ParseUrlErr,

    #[error("read cookie error")]
    ReadCookieErr,

    #[error("write cookie error")]
    WriteCookieErr,
}

type TResult<T> = std::result::Result<T, TError>;
type TError = ApiErr;

#[cfg(test)]
mod public_api_tests {
    use super::NcmClient;

    #[test]
    fn ncm_client_is_the_primary_constructible_api() {
        NcmClient::builder()
            .cache(false)
            .preserve_cookies(false)
            .build()
            .expect("default client configuration is valid");
    }
}

#[cfg(all(test, feature = "live-tests"))]
mod tests {
    use super::NcmClient;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_search() {
        let api = NcmClient::default();
        let resp = api.search("mota", None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }
}
