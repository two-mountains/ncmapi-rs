mod api;
mod api_types;
mod client;
mod crypto;

pub use api::NcmApi;

type TResult<T> = std::result::Result<T, TError>;
type TError = Box<dyn std::error::Error>;

#[cfg(test)]
mod tests {
    use super::NcmApi;
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_search() {
        let api = NcmApi::default();
        let resp = api.search("mota", None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }
}
