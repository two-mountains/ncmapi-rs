//! NetEase Cloud Music API For Rust.

mod api;
mod client;
mod crypto;
pub mod types;

pub use api::{NcmApi, ResourceType, SearchType};
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
mod tests {
    use super::NcmApi;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_search() {
        let api = NcmApi::default();
        let resp = api.search("mota", None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }
}
