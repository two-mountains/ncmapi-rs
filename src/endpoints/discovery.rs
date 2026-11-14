use serde_json::json;

use crate::{
    ApiResponse, DailyBody, Error, NcmClient, Result, SearchBody, ToplistBody,
    transport::{CLOUD_SEARCH, RECOMMEND_SONGS, RequestPlanBuilder, TOPLIST},
};

/// Discovery endpoints exposed by [`NcmClient::discovery`].
pub struct Discovery<'client> {
    client: &'client NcmClient,
}

impl NcmClient {
    pub fn discovery(&self) -> Discovery<'_> {
        Discovery { client: self }
    }
}

/// The category searched by [`Discovery::search`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u16)]
pub enum SearchKind {
    #[default]
    Song = 1,
    Album = 10,
    Artist = 100,
    Playlist = 1000,
    User = 1002,
    MusicVideo = 1004,
    Lyric = 1006,
    Podcast = 1009,
    Video = 1014,
    All = 1018,
}

/// Validated-by-construction input to a search request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchQuery {
    keywords: String,
    kind: SearchKind,
    limit: u16,
    offset: u32,
}

impl SearchQuery {
    pub fn new(keywords: impl Into<String>) -> Result<Self> {
        let keywords = keywords.into();
        if keywords.trim().is_empty() {
            return Err(Error::InvalidInput {
                field: "search keywords",
                reason: "must not be blank",
            });
        }
        Ok(Self {
            keywords,
            kind: SearchKind::Song,
            limit: 30,
            offset: 0,
        })
    }

    pub fn kind(mut self, kind: SearchKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn page(mut self, limit: u16, offset: u32) -> Result<Self> {
        if limit == 0 {
            return Err(Error::InvalidInput {
                field: "search limit",
                reason: "must be greater than zero",
            });
        }
        self.limit = limit;
        self.offset = offset;
        Ok(self)
    }
}

impl<'client> Discovery<'client> {
    /// Searches NetEase Cloud Music with a strongly typed query.
    /// Daily recommendations for the signed-in account.
    pub async fn daily(&self) -> Result<ApiResponse<DailyBody>> {
        self.client
            .execute(
                RequestPlanBuilder::post(RECOMMEND_SONGS)
                    .cookie("os", "ios")
                    .build(),
            )
            .await?
            .decode()
    }

    /// The official chart list (no session required).
    pub async fn toplists(&self) -> Result<ApiResponse<ToplistBody>> {
        self.client
            .execute(RequestPlanBuilder::post(TOPLIST).build())
            .await?
            .decode()
    }

    pub async fn search(&self, query: SearchQuery) -> Result<ApiResponse<SearchBody>> {
        let request = RequestPlanBuilder::post(CLOUD_SEARCH)
            .payload(json!({
                "s": query.keywords,
                "type": query.kind as u16,
                "limit": query.limit,
                "offset": query.offset,
            }))
            .build();

        self.client.execute(request).await?.decode()
    }
}

#[cfg(test)]
mod tests {
    use super::{SearchKind, SearchQuery};

    #[test]
    fn search_query_is_an_immutable_value_builder() {
        let query = SearchQuery::new("mota")
            .unwrap()
            .kind(SearchKind::Artist)
            .page(10, 20)
            .unwrap();

        assert_eq!(query.kind, SearchKind::Artist);
        assert_eq!(query.limit, 10);
        assert_eq!(query.offset, 20);
    }

    #[test]
    fn search_query_rejects_blank_keywords_and_zero_limit() {
        assert!(SearchQuery::new(" \t ").is_err());
        assert!(SearchQuery::new("mota").unwrap().page(0, 0).is_err());
    }

    #[tokio::test]
    async fn daily_decodes_songs_and_reasons() {
        use std::sync::Arc;

        use async_trait::async_trait;

        use crate::{
            NcmClient,
            transport::{RequestPlan, Response, Transport},
        };

        struct Stub;

        #[async_trait]
        impl Transport for Stub {
            async fn execute(&self, plan: RequestPlan) -> crate::Result<Response> {
                assert!(
                    plan.into_parts()
                        .url
                        .ends_with("/api/v3/discovery/recommend/songs")
                );
                Ok(Response::new(
                    br#"{"code":200,"data":{"dailySongs":[{"id":1,"name":"mota"}],"recommendReasons":[{"songId":1,"reason":"because"}]}}"#
                        .to_vec(),
                ))
            }
        }

        let client = NcmClient::with_transport(Arc::new(Stub));
        let response = client.discovery().daily().await.unwrap();
        assert_eq!(response.body.data.daily_songs[0].name, "mota");
        assert_eq!(
            response.body.data.recommend_reasons[0].reason.as_deref(),
            Some("because")
        );
    }

    #[tokio::test]
    async fn toplists_decode_entries() {
        use std::sync::Arc;

        use async_trait::async_trait;

        use crate::{
            NcmClient,
            transport::{RequestPlan, Response, Transport},
        };

        struct Stub;

        #[async_trait]
        impl Transport for Stub {
            async fn execute(&self, _: RequestPlan) -> crate::Result<Response> {
                Ok(Response::new(
                    r#"{"code":200,"list":[{"id":1,"name":"热歌榜","trackCount":200,"updateFrequency":"每天更新"}]}"#
                        .as_bytes()
                        .to_vec(),
                ))
            }
        }

        let client = NcmClient::with_transport(Arc::new(Stub));
        let response = client.discovery().toplists().await.unwrap();
        assert_eq!(response.body.list[0].name, "热歌榜");
        assert_eq!(response.body.list[0].track_count, 200);
    }
}
