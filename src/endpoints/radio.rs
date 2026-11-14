//! Personal FM and like endpoints.
//!
//! All three actions are user-initiated: nothing here runs on a timer or in the
//! background. That is a product decision as much as an API one.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::{
    ApiResponse, EmptyBody, FmBody, LikeListBody, NcmClient, Result, TrackId, UserId,
    transport::{LIKE, LIKELIST, PERSONAL_FM, RequestPlanBuilder, fm_trash},
};

/// Radio endpoints exposed by [`NcmClient::radio`].
pub struct Radio<'client> {
    client: &'client NcmClient,
}

impl NcmClient {
    pub fn radio(&self) -> Radio<'_> {
        Radio { client: self }
    }
}

impl Radio<'_> {
    /// Fetches the next personal-FM batch (requires a session).
    pub async fn fm_batch(&self) -> Result<ApiResponse<FmBody>> {
        self.client
            .execute(RequestPlanBuilder::post(PERSONAL_FM).uncacheable().build())
            .await?
            .decode()
    }

    /// Removes a track from the personal FM station.
    pub async fn trash(&self, id: TrackId) -> Result<ApiResponse<EmptyBody>> {
        let time = 10
            + SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |elapsed| u64::from(elapsed.subsec_nanos()) % 20);
        let url = fm_trash(id.0, time);
        self.client
            .execute(
                RequestPlanBuilder::post(url)
                    .payload(json!({ "songId": id.0 }))
                    .uncacheable()
                    .build(),
            )
            .await?
            .decode()
    }

    /// Sets or clears the like flag for a track.
    ///
    /// This is set-state, not toggle: repeating the same call is harmless.
    pub async fn set_like(&self, id: TrackId, liked: bool) -> Result<ApiResponse<EmptyBody>> {
        self.client
            .execute(
                RequestPlanBuilder::post(LIKE)
                    .cookie("os", "pc")
                    .payload(json!({
                        "alg": "itembased",
                        "time": 3,
                        "like": liked,
                        "trackId": id.0,
                    }))
                    .uncacheable()
                    .build(),
            )
            .await?
            .decode()
    }

    /// Returns every liked track id for a user.
    pub async fn liked_ids(&self, user: UserId) -> Result<ApiResponse<LikeListBody>> {
        self.client
            .execute(
                RequestPlanBuilder::post(LIKELIST)
                    .payload(json!({ "uid": user.0 }))
                    .uncacheable()
                    .build(),
            )
            .await?
            .decode()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::{
        NcmClient, Result, TrackId, UserId,
        transport::{RequestPlan, Response, Transport},
    };

    struct StubTransport {
        url_prefix: &'static str,
        body: &'static str,
    }

    #[async_trait]
    impl Transport for StubTransport {
        async fn execute(&self, plan: RequestPlan) -> Result<Response> {
            let parts = plan.into_parts();
            assert!(
                parts.url.starts_with(self.url_prefix),
                "unexpected url {}",
                parts.url
            );
            if parts.url.contains("/radio/trash/add") {
                assert!(parts.url.contains("songId=42"));
                assert!(parts.url.contains("alg=RT"));
            }
            if parts.url.contains("/radio/like") {
                assert_eq!(parts.payload["like"], true);
                assert_eq!(parts.payload["trackId"], 42);
                assert_eq!(parts.payload["alg"], "itembased");
            }
            Ok(Response::new(self.body.as_bytes().to_vec()))
        }
    }

    fn client(url_prefix: &'static str, body: &'static str) -> NcmClient {
        NcmClient::with_transport(Arc::new(StubTransport { url_prefix, body }))
    }

    #[tokio::test]
    async fn fm_batch_decodes_tracks() {
        let client = client(
            "https://music.163.com/weapi/v1/radio/get",
            r#"{"code":200,"data":[{"id":1,"name":"mota"}]}"#,
        );
        let response = client.radio().fm_batch().await.unwrap();
        assert_eq!(response.body.data[0].name, "mota");
    }

    #[tokio::test]
    async fn trash_targets_the_song_and_alg() {
        let client = client(
            "https://music.163.com/weapi/radio/trash/add",
            r#"{"code":200}"#,
        );
        client.radio().trash(TrackId(42)).await.unwrap();
    }

    #[tokio::test]
    async fn like_is_set_state() {
        let client = client("https://music.163.com/api/radio/like", r#"{"code":200}"#);
        client.radio().set_like(TrackId(42), true).await.unwrap();
    }

    #[tokio::test]
    async fn liked_ids_decode() {
        let client = client(
            "https://music.163.com/weapi/song/like/get",
            r#"{"code":200,"ids":[1,2,3]}"#,
        );
        let response = client.radio().liked_ids(UserId(9)).await.unwrap();
        assert_eq!(response.body.ids, vec![1, 2, 3]);
    }
}
