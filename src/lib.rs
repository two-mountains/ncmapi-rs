//! NetEase Cloud Music API For Rust.

mod api;
mod client;
mod crypto;
pub mod types;

pub use api::{NcmApi, ResourceType, SearchType};

type TResult<T> = std::result::Result<T, TError>;
type TError = Box<dyn std::error::Error>;

#[cfg(test)]
mod tests {
    use super::NcmApi;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_search() {
        let api = NcmApi::default();
        let resp = api.cloud_search("mota", None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }
}
