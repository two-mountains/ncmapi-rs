use serde_json::json;

use crate::{
    ApiResponse, LyricsBody, NcmClient, PlaylistDetailBody, Result, StreamUrlsBody,
    TrackDetailsBody, TrackId,
    crypto::Crypto,
    transport::{LYRIC, PLAYLIST_DETAIL, RequestPlanBuilder, STREAM_URLS, TRACK_DETAILS},
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
                    body: TrackDetailsBody::default(),
                });
            }
            songs.append(&mut response.body.songs);
        }
        Ok(ApiResponse {
            code: 200,
            body: TrackDetailsBody { songs },
        })
    }

    /// Fetches the LRC lyric document for a track.
    pub async fn lyrics(&self, id: u64) -> Result<ApiResponse<LyricsBody>> {
        let request = RequestPlanBuilder::post(LYRIC)
            .payload(json!({ "id": id, "lv": -1, "tv": -1, "kv": -1 }))
            .build();
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
        LyricsBody, NcmClient, Result,
        transport::{RequestPlan, Response, Transport},
    };

    struct StubTransport {
        url: &'static str,
        body: &'static str,
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
    async fn playlist_track_limit_must_be_positive() {
        let client = client("https://music.163.com/api/playlist/track/all", "{}");
        assert!(client.library().playlist_tracks(1, 0, 0).await.is_err());
    }
}
