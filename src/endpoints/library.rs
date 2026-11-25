use serde_json::json;

use crate::{
    ApiResponse, CloudBody, CommentsBody, EmptyBody, LyricsBody, NcmClient, PlaylistCreatedBody,
    PlaylistDetailBody, Result, StreamUrlsBody, TrackDetailsBody, TrackId,
    crypto::Crypto,
    transport::{
        COMMENTS, LYRIC, PLAYLIST_CREATE, PLAYLIST_DESCRIBE, PLAYLIST_DETAIL, PLAYLIST_MANIPULATE,
        PLAYLIST_REMOVE, PLAYLIST_RENAME, RequestPlanBuilder, STREAM_URLS, TRACK_DETAILS,
        USER_CLOUD,
    },
};

/// Music-library endpoints exposed by [`NcmClient::library`].
pub struct Library<'client> {
    client: &'client NcmClient,
}

impl NcmClient {
    pub fn library(&self) -> Library<'_> {
        Library { client: self }
    }
}

impl<'client> Library<'client> {
    /// Retrieves metadata for every requested track.
    pub async fn tracks(&self, ids: &[TrackId]) -> Result<ApiResponse<TrackDetailsBody>> {
        let tracks = ids
            .iter()
            .map(|id| json!({ "id": id.0 }).to_string())
            .collect::<Vec<_>>();
        let request = RequestPlanBuilder::post(TRACK_DETAILS)
            .payload(json!({ "c": tracks }))
            .build();

        self.client.execute(request).await?.decode()
    }

    /// Fetches playlist metadata together with its first page of tracks.
    pub async fn playlist_detail(&self, id: u64) -> Result<ApiResponse<PlaylistDetailBody>> {
        let request = RequestPlanBuilder::post(PLAYLIST_DETAIL)
            .payload(json!({ "id": id, "n": 1000, "s": 8 }))
            .build();
        self.client.execute(request).await?.decode()
    }

    /// Creates an empty playlist and returns its id.
    ///
    /// The provider does not return the full playlist page; a later
    /// [`Self::playlist_detail`] is authoritative.
    pub async fn playlist_create(
        &self,
        name: &str,
        private: bool,
    ) -> Result<ApiResponse<PlaylistCreatedBody>> {
        let request = RequestPlanBuilder::post(PLAYLIST_CREATE)
            .payload(json!({
                "name": name,
                "privacy": if private { 10 } else { 0 },
            }))
            .uncacheable()
            .build();

        let response = self.client.execute(request).await?.decode();
        self.client.clear_cache();
        response
    }

    /// Renames a playlist and/or replaces its description.
    ///
    /// The provider has separate routes for the two fields; `None` skips that
    /// request, so an update that changes one field cannot clear the other. The
    /// first failing response is returned, and the response cache is dropped
    /// even when a request fails, so a later read never serves the pre-write
    /// answer.
    pub async fn playlist_update(
        &self,
        id: u64,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<ApiResponse<EmptyBody>> {
        let result = self.playlist_update_inner(id, name, description).await;
        self.client.clear_cache();
        result
    }

    async fn playlist_update_inner(
        &self,
        id: u64,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<ApiResponse<EmptyBody>> {
        if name.is_none() && description.is_none() {
            return Err(crate::Error::InvalidInput {
                field: "playlist update",
                reason: "pass a name or a description",
            });
        }

        if let Some(name) = name {
            let response: ApiResponse<EmptyBody> = self
                .client
                .execute(
                    RequestPlanBuilder::post(PLAYLIST_RENAME)
                        .payload(json!({ "id": id, "name": name }))
                        .uncacheable()
                        .build(),
                )
                .await?
                .decode()?;
            if response.code != 200 {
                return Ok(response);
            }
        }
        if let Some(description) = description {
            let response: ApiResponse<EmptyBody> = self
                .client
                .execute(
                    RequestPlanBuilder::post(PLAYLIST_DESCRIBE)
                        .payload(json!({ "id": id, "desc": description }))
                        .uncacheable()
                        .build(),
                )
                .await?
                .decode()?;
            if response.code != 200 {
                return Ok(response);
            }
        }

        Ok(ApiResponse {
            code: 200,
            message: None,
            body: EmptyBody {},
        })
    }

    /// Adds tracks to a playlist.
    ///
    /// The provider answers `502 歌单内歌曲重复` when a track is already there;
    /// that is treated as success so a retry stays harmless.
    pub async fn playlist_add_tracks(
        &self,
        id: u64,
        tracks: &[TrackId],
    ) -> Result<ApiResponse<EmptyBody>> {
        let response = self.manipulate_tracks("add", id, tracks).await?;
        self.client.clear_cache();
        let duplicate = response.code == 502
            && response
                .message
                .as_deref()
                .is_some_and(|message| message.contains("重复"));
        if duplicate {
            return Ok(ApiResponse {
                code: 200,
                message: None,
                body: EmptyBody {},
            });
        }
        Ok(response)
    }

    /// Removes tracks from a playlist. Missing tracks are ignored by the
    /// provider, so repeating the call is harmless.
    pub async fn playlist_remove_tracks(
        &self,
        id: u64,
        tracks: &[TrackId],
    ) -> Result<ApiResponse<EmptyBody>> {
        let response = self.manipulate_tracks("del", id, tracks).await?;
        self.client.clear_cache();
        Ok(response)
    }

    async fn manipulate_tracks(
        &self,
        op: &str,
        id: u64,
        tracks: &[TrackId],
    ) -> Result<ApiResponse<EmptyBody>> {
        let ids = tracks.iter().map(|track| track.0).collect::<Vec<_>>();
        let request = RequestPlanBuilder::post(PLAYLIST_MANIPULATE)
            .payload(json!({
                "op": op,
                "pid": id,
                "trackIds": json!(ids).to_string(),
                "imme": true,
            }))
            .uncacheable()
            .build();

        self.client.execute(request).await?.decode()
    }

    /// Deletes one of the user's playlists.
    ///
    /// The provider's remove route takes `ids` as a JSON array string, not
    /// `id`; sending `id` answers `400 请求参数错误`.
    pub async fn playlist_delete(&self, id: u64) -> Result<ApiResponse<EmptyBody>> {
        let request = RequestPlanBuilder::post(PLAYLIST_REMOVE)
            .payload(json!({ "ids": json!([id]).to_string() }))
            .uncacheable()
            .build();

        let response = self.client.execute(request).await?.decode();
        self.client.clear_cache();
        response
    }

    /// Fetches a page of a playlist's tracks.
    ///
    /// Playlist detail returns the full ordered `trackIds` list, so paging is
    /// resolved locally and the selected ids are batched through the song
    /// detail endpoint.
    ///
    /// # Errors
    ///
    /// Rejects a zero limit before any request is made.
    pub async fn playlist_tracks(
        &self,
        id: u64,
        limit: u16,
        offset: u32,
    ) -> Result<ApiResponse<TrackDetailsBody>> {
        if limit == 0 {
            return Err(crate::Error::InvalidInput {
                field: "playlist track limit",
                reason: "must be greater than zero",
            });
        }

        let detail = self.playlist_detail(id).await?;
        if detail.code != 200 {
            return Ok(ApiResponse {
                code: detail.code,
                message: detail.message,
                body: TrackDetailsBody::default(),
            });
        }
        let mut ids: Vec<u64> = detail
            .body
            .playlist
            .as_ref()
            .map(|playlist| {
                playlist
                    .track_ids
                    .iter()
                    .map(|track| track.id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if ids.is_empty() {
            ids = detail
                .body
                .playlist
                .map(|playlist| playlist.tracks.into_iter().map(|track| track.id).collect())
                .unwrap_or_default();
        }

        let page: Vec<u64> = ids
            .into_iter()
            .skip(offset as usize)
            .take(usize::from(limit))
            .collect();
        let mut songs = Vec::with_capacity(page.len());
        for chunk in page.chunks(500) {
            let ids = chunk.iter().map(|id| TrackId(*id)).collect::<Vec<_>>();
            let mut response = self.tracks(&ids).await?;
            if response.code != 200 {
                return Ok(ApiResponse {
                    code: response.code,
                    message: response.message,
                    body: TrackDetailsBody::default(),
                });
            }
            songs.append(&mut response.body.songs);
        }
        Ok(ApiResponse {
            code: 200,
            message: None,
            body: TrackDetailsBody { songs },
        })
    }

    /// Fetches a page of the account's music cloud.
    ///
    /// # Errors
    ///
    /// Rejects a zero limit before any request is made.
    pub async fn cloud(&self, limit: u32, offset: u32) -> Result<ApiResponse<CloudBody>> {
        if limit == 0 {
            return Err(crate::Error::InvalidInput {
                field: "cloud limit",
                reason: "must be greater than zero",
            });
        }
        let request = RequestPlanBuilder::post(USER_CLOUD)
            .payload(json!({ "limit": limit, "offset": offset }))
            .build();
        self.client.execute(request).await?.decode()
    }

    /// Fetches the LRC lyric document for a track.
    pub async fn lyrics(&self, id: u64) -> Result<ApiResponse<LyricsBody>> {
        let request = RequestPlanBuilder::post(LYRIC)
            .payload(json!({ "id": id, "lv": -1, "tv": -1, "kv": -1 }))
            .build();
        self.client.execute(request).await?.decode()
    }

    /// Fetches public comments for a track.
    ///
    /// `limit` is capped by the provider; callers page with `offset`.
    ///
    /// # Errors
    ///
    /// Rejects a zero limit before any request is made.
    pub async fn comments(
        &self,
        id: u64,
        limit: u16,
        offset: u32,
    ) -> Result<ApiResponse<CommentsBody>> {
        if limit == 0 {
            return Err(crate::Error::InvalidInput {
                field: "comment limit",
                reason: "must be greater than zero",
            });
        }
        let url = format!("{COMMENTS}{id}?limit={limit}&offset={offset}");
        let request = RequestPlanBuilder::get(url).build();
        self.client.execute(request).await?.decode()
    }

    /// Resolves playable URLs using the documented maximum bitrate.
    pub async fn stream_urls(&self, ids: &[TrackId]) -> Result<ApiResponse<StreamUrlsBody>> {
        let request = RequestPlanBuilder::post(STREAM_URLS)
            .encryption(Crypto::Eapi)
            .cookie("os", "pc")
            .api_path("/api/song/enhance/player/url")
            .payload(json!({ "ids": ids.iter().map(|id| id.0).collect::<Vec<_>>(), "br": 999_000 }))
            .build();

        self.client.execute(request).await?.decode()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::{
        LyricsBody, NcmClient, Result, TrackId,
        transport::{RequestPlan, Response, Transport},
    };

    struct StubTransport {
        url: &'static str,
        body: &'static str,
    }

    struct CapturingTransport {
        url: &'static str,
        body: &'static str,
        payload: Arc<std::sync::Mutex<Option<serde_json::Value>>>,
    }

    #[async_trait]
    impl Transport for CapturingTransport {
        async fn execute(&self, plan: RequestPlan) -> Result<Response> {
            let parts = plan.into_parts();
            assert_eq!(parts.url, self.url);
            *self.payload.lock().unwrap() = Some(parts.payload);
            Ok(Response::new(self.body.as_bytes().to_vec()))
        }
    }

    struct ScriptedTransport {
        steps: std::sync::Mutex<Vec<(&'static str, &'static str)>>,
    }

    #[async_trait]
    impl Transport for ScriptedTransport {
        async fn execute(&self, plan: RequestPlan) -> Result<Response> {
            let parts = plan.into_parts();
            let (url, body) = {
                let mut steps = self.steps.lock().unwrap();
                steps.remove(0)
            };
            assert_eq!(parts.url, url);
            Ok(Response::new(body.as_bytes().to_vec()))
        }
    }

    fn scripted(steps: Vec<(&'static str, &'static str)>) -> NcmClient {
        NcmClient::with_transport(Arc::new(ScriptedTransport {
            steps: std::sync::Mutex::new(steps),
        }))
    }

    fn capturing(
        url: &'static str,
        body: &'static str,
    ) -> (NcmClient, Arc<std::sync::Mutex<Option<serde_json::Value>>>) {
        let payload = Arc::new(std::sync::Mutex::new(None));
        let client = NcmClient::with_transport(Arc::new(CapturingTransport {
            url,
            body,
            payload: Arc::clone(&payload),
        }));
        (client, payload)
    }

    #[async_trait]
    impl Transport for StubTransport {
        async fn execute(&self, plan: RequestPlan) -> Result<Response> {
            assert_eq!(plan.into_parts().url, self.url);
            Ok(Response::new(self.body.as_bytes().to_vec()))
        }
    }

    fn client(url: &'static str, body: &'static str) -> NcmClient {
        NcmClient::with_transport(Arc::new(StubTransport { url, body }))
    }

    #[tokio::test]
    async fn every_write_drops_the_response_cache() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct Counting {
            cleared: AtomicUsize,
            body: &'static str,
        }

        #[async_trait]
        impl Transport for Counting {
            async fn execute(&self, _plan: RequestPlan) -> Result<Response> {
                Ok(Response::new(self.body.as_bytes().to_vec()))
            }

            fn clear_cache(&self) {
                self.cleared.fetch_add(1, Ordering::SeqCst);
            }
        }

        let transport = Arc::new(Counting {
            cleared: AtomicUsize::new(0),
            body: r#"{"code":200,"id":77}"#,
        });
        let shared: Arc<dyn Transport> = transport.clone();
        let client = NcmClient::with_transport(shared);

        client
            .library()
            .playlist_create("Name", false)
            .await
            .unwrap();
        client
            .library()
            .playlist_update(77, Some("Name"), Some("Desc"))
            .await
            .unwrap();
        client
            .library()
            .playlist_add_tracks(77, &[TrackId(42)])
            .await
            .unwrap();
        client
            .library()
            .playlist_remove_tracks(77, &[TrackId(42)])
            .await
            .unwrap();
        client.library().playlist_delete(77).await.unwrap();

        assert_eq!(
            transport.cleared.load(Ordering::SeqCst),
            5,
            "create, update, add, remove and delete each clear the cache"
        );
    }

    #[tokio::test]
    async fn the_client_can_drop_cached_responses_after_a_write() {
        struct Counting {
            cleared: std::sync::atomic::AtomicBool,
        }

        #[async_trait]
        impl Transport for Counting {
            async fn execute(&self, _plan: RequestPlan) -> Result<Response> {
                Ok(Response::new(b"{\"code\":200}".to_vec()))
            }

            fn clear_cache(&self) {
                self.cleared
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }

        let transport = Arc::new(Counting {
            cleared: std::sync::atomic::AtomicBool::new(false),
        });
        let shared: Arc<dyn Transport> = transport.clone();
        let client = NcmClient::with_transport(shared);
        client.clear_cache();

        assert!(transport.cleared.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn playlist_create_sends_privacy_and_returns_the_id() {
        let (client, payload) = capturing(
            "https://music.163.com/api/playlist/create",
            r#"{"code":200,"id":77,"name":"Late night"}"#,
        );
        let response = client
            .library()
            .playlist_create("Late night", true)
            .await
            .unwrap();

        assert_eq!(response.body.id, 77);
        assert_eq!(response.body.name.as_deref(), Some("Late night"));
        let sent = payload.lock().unwrap().clone().unwrap();
        assert_eq!(sent["name"], "Late night");
        assert_eq!(sent["privacy"], 10);
    }

    #[tokio::test]
    async fn a_combined_update_stops_at_the_first_failure() {
        let client = scripted(vec![
            (
                "https://music.163.com/api/playlist/update/name",
                r#"{"code":400,"message":"标题字数异常"}"#,
            ),
            (
                "https://music.163.com/api/playlist/desc/update",
                r#"{"code":200}"#,
            ),
        ]);

        let response = client
            .library()
            .playlist_update(77, Some("x".repeat(250).leak()), Some("fine"))
            .await
            .unwrap();

        assert_eq!(response.code, 400, "the rejected rename is not hidden");
    }

    #[tokio::test]
    async fn a_combined_update_runs_both_routes() {
        let client = scripted(vec![
            (
                "https://music.163.com/api/playlist/update/name",
                r#"{"code":200}"#,
            ),
            (
                "https://music.163.com/api/playlist/desc/update",
                r#"{"code":200}"#,
            ),
        ]);

        let response = client
            .library()
            .playlist_update(77, Some("Name"), Some("Description"))
            .await
            .unwrap();

        assert_eq!(response.code, 200);
    }

    #[tokio::test]
    async fn an_empty_update_is_rejected_before_any_request() {
        let client = scripted(Vec::new());
        let error = client
            .library()
            .playlist_update(77, None, None)
            .await
            .unwrap_err();
        assert!(
            matches!(error, crate::Error::InvalidInput { .. }),
            "{error}"
        );
    }

    #[tokio::test]
    async fn an_unrelated_502_is_not_swallowed() {
        let (client, _) = capturing(
            "https://music.163.com/api/playlist/manipulate/tracks",
            r#"{"code":502,"message":"server busy"}"#,
        );

        let response = client
            .library()
            .playlist_add_tracks(77, &[TrackId(42)])
            .await
            .unwrap();

        assert_eq!(response.code, 502);
    }

    #[tokio::test]
    async fn adding_a_duplicate_track_is_treated_as_success() {
        let (client, _) = capturing(
            "https://music.163.com/api/playlist/manipulate/tracks",
            r#"{"code":502,"message":"歌单内歌曲重复"}"#,
        );

        let response = client
            .library()
            .playlist_add_tracks(77, &[TrackId(42)])
            .await
            .unwrap();

        assert_eq!(response.code, 200, "a duplicate add is idempotent");
    }

    #[tokio::test]
    async fn a_create_without_an_id_decodes_to_zero() {
        let (client, _) = capturing(
            "https://music.163.com/api/playlist/create",
            r#"{"code":200}"#,
        );
        let response = client
            .library()
            .playlist_create("Late night", false)
            .await
            .unwrap();
        assert_eq!(response.body.id, 0, "the caller turns this into an error");
    }

    #[tokio::test]
    async fn a_create_error_code_is_passed_through() {
        let (client, _) = capturing(
            "https://music.163.com/api/playlist/create",
            r#"{"code":301,"message":"need login"}"#,
        );
        let response = client
            .library()
            .playlist_create("Late night", false)
            .await
            .unwrap();
        assert_eq!(response.code, 301);
    }

    #[tokio::test]
    async fn playlist_rename_uses_its_own_route() {
        let (client, payload) = capturing(
            "https://music.163.com/api/playlist/update/name",
            r#"{"code":200}"#,
        );
        client
            .library()
            .playlist_update(77, Some("Late night 2"), None)
            .await
            .unwrap();

        let sent = payload.lock().unwrap().clone().unwrap();
        assert_eq!(sent["id"], 77);
        assert_eq!(sent["name"], "Late night 2");
        assert!(sent.get("desc").is_none(), "an omitted field is untouched");
    }

    #[tokio::test]
    async fn playlist_describe_uses_its_own_route() {
        let (client, payload) = capturing(
            "https://music.163.com/api/playlist/desc/update",
            r#"{"code":200}"#,
        );
        client
            .library()
            .playlist_update(77, None, Some("after 22:00"))
            .await
            .unwrap();

        let sent = payload.lock().unwrap().clone().unwrap();
        assert_eq!(sent["id"], 77);
        assert_eq!(sent["desc"], "after 22:00");
        assert!(sent.get("name").is_none());
    }

    #[tokio::test]
    async fn playlist_tracks_are_added_and_removed_with_the_same_route() {
        for (op, add) in [("add", true), ("del", false)] {
            let (client, payload) = capturing(
                "https://music.163.com/api/playlist/manipulate/tracks",
                r#"{"code":200}"#,
            );
            if add {
                client
                    .library()
                    .playlist_add_tracks(77, &[TrackId(42), TrackId(43)])
                    .await
                    .unwrap();
            } else {
                client
                    .library()
                    .playlist_remove_tracks(77, &[TrackId(42)])
                    .await
                    .unwrap();
            }

            let sent = payload.lock().unwrap().clone().unwrap();
            assert_eq!(sent["op"], op);
            assert_eq!(sent["pid"], 77);
            assert_eq!(sent["imme"], true);
            let ids: Vec<u64> = serde_json::from_str(sent["trackIds"].as_str().unwrap()).unwrap();
            if add {
                assert_eq!(ids, vec![42, 43]);
            } else {
                assert_eq!(ids, vec![42]);
            }
        }
    }

    #[tokio::test]
    async fn playlist_delete_sends_an_ids_array_string() {
        let (client, payload) = capturing(
            "https://music.163.com/api/playlist/remove",
            r#"{"code":200}"#,
        );
        client.library().playlist_delete(77).await.unwrap();

        let sent = payload.lock().unwrap().clone().unwrap();
        let ids: Vec<u64> = serde_json::from_str(sent["ids"].as_str().unwrap()).unwrap();
        assert_eq!(ids, vec![77]);
    }

    #[tokio::test]
    async fn comments_decode_hot_and_regular_entries() {
        let body = r#"{"code":200,"total":2,"more":false,
            "hotComments":[{"commentId":1,"content":"first","time":1,"likedCount":9,
                "user":{"userId":7,"nickname":"ben","avatarUrl":"https://img/7"},
                "beReplied":[{"content":"parent","user":{"userId":8,"nickname":"aka"}}]}],
            "comments":[{"commentId":2,"content":"second","user":null}]}"#;
        let url = "https://music.163.com/api/v1/resource/comments/R_SO_4_186016?limit=20&offset=0";
        let response = client(url, body)
            .library()
            .comments(186_016, 20, 0)
            .await
            .unwrap();
        assert_eq!(response.body.total, 2);
        assert!(!response.body.more);
        assert_eq!(response.body.hot_comments[0].liked_count, 9);
        assert_eq!(
            response.body.hot_comments[0]
                .user
                .as_ref()
                .unwrap()
                .nickname,
            "ben"
        );
        assert_eq!(
            response.body.hot_comments[0].be_replied[0]
                .user
                .as_ref()
                .unwrap()
                .nickname,
            "aka"
        );
        assert!(response.body.comments[0].user.is_none());
    }

    #[tokio::test]
    async fn comments_tolerate_null_fields_from_deleted_content() {
        let body = r#"{"code":200,"total":null,"more":null,
            "hotComments":null,
            "comments":[{"commentId":2,"content":null,"time":null,"likedCount":null,
                "user":{"userId":null,"nickname":null,"avatarUrl":null},
                "beReplied":[{"content":null,"user":{"nickname":null,"avatarUrl":null}}]}]}"#;
        let url = "https://music.163.com/api/v1/resource/comments/R_SO_4_186016?limit=20&offset=0";
        let response = client(url, body)
            .library()
            .comments(186_016, 20, 0)
            .await
            .unwrap();
        assert_eq!(response.body.total, 0);
        assert!(!response.body.more);
        assert!(response.body.hot_comments.is_empty());
        let comment = &response.body.comments[0];
        assert_eq!(comment.comment_id, 2);
        assert_eq!(comment.content, "");
        assert_eq!(comment.liked_count, 0);
        assert_eq!(comment.user.as_ref().unwrap().nickname, "");
        assert_eq!(comment.be_replied[0].content, "");
    }

    #[tokio::test]
    async fn comments_reject_a_zero_limit() {
        let error = client("https://example.test", "{}")
            .library()
            .comments(1, 0, 0)
            .await
            .unwrap_err();
        assert!(matches!(error, crate::Error::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn playlist_detail_decodes_metadata_and_tracks() {
        let body = r#"{"code":200,"playlist":{"id":5,"name":"Focus","trackCount":42,
            "coverImgUrl":"https://img/5.jpg","creator":{"userId":9,"nickname":"ben"},
            "tracks":[{"id":1,"name":"mota","ar":[],"al":{"id":0,"name":""},"dt":1000}]}}"#;
        let response = client("https://music.163.com/api/v6/playlist/detail", body)
            .library()
            .playlist_detail(5)
            .await
            .unwrap();
        let playlist = response.body.playlist.unwrap();
        assert_eq!(playlist.track_count, 42);
        assert_eq!(playlist.creator.unwrap().nickname, "ben");
        assert_eq!(playlist.tracks[0].name, "mota");
    }

    struct PlaylistStub;

    #[async_trait]
    impl Transport for PlaylistStub {
        async fn execute(&self, plan: RequestPlan) -> Result<Response> {
            let url = plan.into_parts().url;
            let body = if url.ends_with("/api/v6/playlist/detail") {
                r#"{"code":200,"playlist":{"id":5,"name":"Focus",
                    "trackIds":[{"id":2},{"id":1},{"id":3}],"tracks":[]}}"#
            } else if url.ends_with("/api/v3/song/detail") {
                r#"{"code":200,"songs":[{"id":2,"name":"second"},{"id":1,"name":"first"}]}"#
            } else {
                panic!("unexpected request to {url}");
            };
            Ok(Response::new(body.as_bytes().to_vec()))
        }
    }

    #[tokio::test]
    async fn playlist_tracks_pages_through_track_ids() {
        let client = NcmClient::with_transport(Arc::new(PlaylistStub));
        let response = client.library().playlist_tracks(5, 2, 0).await.unwrap();
        assert_eq!(response.body.songs.len(), 2);
        assert_eq!(response.body.songs[0].name, "second");

        let second = client.library().playlist_tracks(5, 2, 2).await.unwrap();
        assert_eq!(second.body.songs.len(), 2);
    }

    #[tokio::test]
    async fn lyrics_expose_text_and_translation() {
        let body = r#"{"code":200,"lrc":{"lyric":"[00:00.00]hello"},
            "tlyric":{"lyric":"[00:00.00]你好"}}"#;
        let response = client("https://music.163.com/api/song/lyric", body)
            .library()
            .lyrics(1)
            .await
            .unwrap();
        assert_eq!(response.body.text(), "[00:00.00]hello");
        assert_eq!(response.body.translation(), Some("[00:00.00]你好"));
        assert_eq!(LyricsBody::default().text(), "");
    }

    #[tokio::test]
    async fn tolerates_null_names_in_track_metadata() {
        let body = r#"{"code":200,"playlist":{"id":5,"name":"Liked",
            "tracks":[{"id":1,"name":null,"ar":[{"id":2,"name":null}],
            "al":{"id":3,"name":null},"dt":null}],"trackIds":[{"id":1}]}}"#;
        let response = client("https://music.163.com/api/v6/playlist/detail", body)
            .library()
            .playlist_detail(5)
            .await
            .unwrap();
        let playlist = response.body.playlist.unwrap();
        assert_eq!(playlist.tracks[0].name, "");
        assert_eq!(playlist.tracks[0].artists[0].name, "");
        assert_eq!(playlist.tracks[0].album.name, "");
        assert_eq!(playlist.tracks[0].duration, 0);
    }

    #[tokio::test]
    async fn cloud_decodes_uploads() {
        let body = r#"{"code":200,"count":2,"size":100,"data":[
            {"songId":1,"songName":"a","artist":"X","album":"Y",
             "simpleSong":{"id":1,"name":"a"}}]}"#;
        let response = client("https://music.163.com/api/v1/cloud/get", body)
            .library()
            .cloud(30, 0)
            .await
            .unwrap();
        assert_eq!(response.body.count, 2);
        assert_eq!(response.body.data[0].song_name, "a");
        assert_eq!(
            response.body.data[0].simple_song.as_ref().unwrap().name,
            "a"
        );
    }

    #[tokio::test]
    async fn playlist_track_limit_must_be_positive() {
        let client = client("https://music.163.com/api/playlist/track/all", "{}");
        assert!(client.library().playlist_tracks(1, 0, 0).await.is_err());
    }
}
