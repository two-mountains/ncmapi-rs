//! NetEase Cloud Music API For Rust.

mod crypto;
mod domain;
mod endpoints;
mod error;
mod models;
mod sdk;
mod transport;

pub use domain::Page;
pub use endpoints::{Auth, Discovery, Library, Profile, Radio, SearchKind, SearchQuery};
pub use error::{Error, Result};
pub use models::{
    AccountBody, Album, ApiResponse, Artist, CloudBody, CloudTrack, Creator, DailyBody, DailyData,
    EmptyBody, FmBody, LikeListBody, Lyric, LyricsBody, Playlist, PlaylistDetail,
    PlaylistDetailBody, QrKeyBody, QrLoginBody, RecommendReason, SearchBody, SearchResults,
    StreamUrl, StreamUrlsBody, ToplistBody, ToplistEntry, Track, TrackDetailsBody, TrackId,
    TrackRef, UserDetailsBody, UserId, UserPlaylistsBody, UserProfile,
};
pub use sdk::{NcmClient, NcmClientBuilder};
#[deprecated(note = "use NcmClient instead")]
pub type NcmApi = NcmClient;
pub(crate) type TResult<T> = Result<T>;

#[cfg(test)]
mod public_api_tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::{NcmClient, SearchQuery};
    use crate::{
        Result,
        transport::{RequestPlan, Response, Transport},
    };

    struct StubTransport;

    #[async_trait]
    impl Transport for StubTransport {
        async fn execute(&self, _: RequestPlan) -> Result<Response> {
            Ok(Response::new(
                br#"{"code":200,"result":{"songs":[{"id":1,"name":"mota"}]}}"#.to_vec(),
            ))
        }
    }

    #[test]
    fn ncm_client_is_the_primary_constructible_api() {
        NcmClient::builder()
            .cache(false)
            .persist_cookies(false)
            .build()
            .expect("default client configuration is valid");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn services_are_testable_without_a_remote_server() {
        let client = NcmClient::with_transport(Arc::new(StubTransport));
        let response = client
            .discovery()
            .search(SearchQuery::new("mota").unwrap())
            .await
            .expect("the stub transport returns a response");

        assert_eq!(response.code, 200);
        assert!(response.is_success());
        assert_eq!(response.body.result.songs[0].name, "mota");
    }
}
