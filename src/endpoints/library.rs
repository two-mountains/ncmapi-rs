use serde_json::json;

use crate::{
    ApiResponse, NcmClient, Result, StreamUrlsBody, TrackDetailsBody, TrackId,
    crypto::Crypto,
    transport::{RequestPlanBuilder, STREAM_URLS, TRACK_DETAILS},
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
