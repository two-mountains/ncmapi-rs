use serde_json::json;

use crate::{
    AlbumDetailBody, ApiResponse, ArtistAlbumsBody, ArtistDetailBody, DailyBody, Error, NcmClient,
    Page, Result, SearchBody, ToplistBody,
    transport::{
        CLOUD_SEARCH, RECOMMEND_SONGS, RequestPlanBuilder, TOPLIST, album_detail, artist_albums,
        artist_detail,
    },
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

    /// Fetches a full artist profile (public, no session required).
    ///
    /// # Errors
    ///
    /// Rejects a zero artist id before any request is made.
    pub async fn artist(&self, id: u64) -> Result<ApiResponse<ArtistDetailBody>> {
        if id == 0 {
            return Err(Error::InvalidInput {
                field: "artist id",
                reason: "must be greater than zero",
            });
        }
        let request = RequestPlanBuilder::get(artist_detail(id)).build();
        self.client.execute(request).await?.decode()
    }

    /// Fetches a page of an artist's albums.
    ///
    /// # Errors
    ///
    /// Rejects a zero artist id or zero page limit before any request is made.
    pub async fn artist_albums(
        &self,
        id: u64,
        limit: u16,
        offset: u32,
    ) -> Result<ApiResponse<ArtistAlbumsBody>> {
        if id == 0 {
            return Err(Error::InvalidInput {
                field: "artist id",
                reason: "must be greater than zero",
            });
        }
        let page = Page::new(limit, offset)?;
        let request =
            RequestPlanBuilder::get(artist_albums(id, page.limit(), page.offset())).build();
        self.client.execute(request).await?.decode()
    }

    /// Fetches an album with its full ordered track list.
    ///
    /// # Errors
    ///
    /// Rejects a zero album id before any request is made.
    pub async fn album(&self, id: u64) -> Result<ApiResponse<AlbumDetailBody>> {
        if id == 0 {
            return Err(Error::InvalidInput {
                field: "album id",
                reason: "must be greater than zero",
            });
        }
        let request = RequestPlanBuilder::get(album_detail(id)).build();
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
                    r#"{"code":200,"list":[{"id":1,"name":"热歌榜","trackCount":200,"updateFrequency":"每天更新","updateFrequency":"刚刚更新"}]}"#
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

    #[tokio::test]
    async fn search_decodes_album_results() {
        use std::sync::Arc;

        use async_trait::async_trait;

        use crate::{
            NcmClient, SearchQuery,
            transport::{RequestPlan, Response, Transport},
        };

        struct Stub;

        #[async_trait]
        impl Transport for Stub {
            async fn execute(&self, plan: RequestPlan) -> crate::Result<Response> {
                assert!(plan.into_parts().url.ends_with("/api/cloudsearch/pc"));
                Ok(Response::new(
                    br#"{"code":200,"result":{"albums":[{"id":242091487,"name":"Mota","picUrl":"https://img","publishTime":1720800000000,"size":1}],"albumCount":4,"songs":[]}}"#
                        .to_vec(),
                ))
            }
        }

        let client = NcmClient::with_transport(Arc::new(Stub));
        let query = SearchQuery::new("mota").unwrap().kind(SearchKind::Album);
        let response = client.discovery().search(query).await.unwrap();
        assert_eq!(response.body.result.album_count, 4);
        assert_eq!(response.body.result.albums[0].name, "Mota");
        assert_eq!(response.body.result.albums[0].size, 1);
    }

    #[tokio::test]
    async fn artist_detail_decodes_a_null_tolerant_profile() {
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
                        .ends_with("/api/artist/head/info/get?id=6452")
                );
                Ok(Response::new(
                    r#"{"code":200,"data":{"artist":{"id":6452,"name":"周杰伦","avatar":null,"cover":"https://cover","albumSize":49,"musicSize":592,"briefDesc":"bio","alias":["Jay Chou"],"identities":null}}}"#
                        .as_bytes()
                        .to_vec(),
                ))
            }
        }

        let client = NcmClient::with_transport(Arc::new(Stub));
        let response = client.discovery().artist(6452).await.unwrap();
        let artist = response.body.data.artist;
        assert_eq!(artist.name, "周杰伦");
        assert_eq!(artist.avatar_url, "");
        assert_eq!(artist.album_size, 49);
        assert_eq!(artist.aliases, vec!["Jay Chou"]);
        assert!(artist.identities.is_empty());
    }

    #[tokio::test]
    async fn artist_albums_pages_and_decodes_entries() {
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
                        .ends_with("/api/artist/albums/6452?limit=2&offset=4")
                );
                Ok(Response::new(
                    r#"{"code":200,"artist":{"id":6452,"name":"周杰伦"},"hotAlbums":[{"id":1,"name":"叶惠美","picUrl":"https://img","publishTime":1056988800000,"size":11}],"more":true}"#
                        .as_bytes()
                        .to_vec(),
                ))
            }
        }

        let client = NcmClient::with_transport(Arc::new(Stub));
        let response = client.discovery().artist_albums(6452, 2, 4).await.unwrap();
        assert!(response.body.more);
        assert_eq!(response.body.hot_albums[0].name, "叶惠美");
        assert_eq!(response.body.artist.unwrap().name, "周杰伦");
    }

    #[tokio::test]
    async fn album_detail_decodes_metadata_and_tracks() {
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
                assert!(plan.into_parts().url.ends_with("/api/v1/album/242091487"));
                Ok(Response::new(
                    br#"{"code":200,"album":{"id":242091487,"name":"Mota","picUrl":"https://img","publishTime":1720800000000,"size":1,"artist":{"id":31787312,"name":"Aash"},"description":"single"},"songs":[{"id":7,"name":"Mota","ar":[{"id":31787312,"name":"Aash"}],"al":{"id":242091487,"name":"Mota","picUrl":"https://img"},"dt":154000}]}"#
                        .to_vec(),
                ))
            }
        }

        let client = NcmClient::with_transport(Arc::new(Stub));
        let response = client.discovery().album(242_091_487).await.unwrap();
        assert_eq!(response.body.album.name, "Mota");
        assert_eq!(response.body.album.artist.unwrap().name, "Aash");
        assert_eq!(response.body.album.description, "single");
        assert_eq!(response.body.songs[0].duration, 154_000);
        assert_eq!(response.body.songs[0].artists[0].name, "Aash");
    }

    #[tokio::test]
    async fn catalogue_requests_validate_their_inputs() {
        use std::sync::Arc;

        use async_trait::async_trait;

        use crate::{
            Error, NcmClient,
            transport::{RequestPlan, Response, Transport},
        };

        struct Stub;

        #[async_trait]
        impl Transport for Stub {
            async fn execute(&self, _: RequestPlan) -> crate::Result<Response> {
                panic!("validation must happen before the request")
            }
        }

        let client = NcmClient::with_transport(Arc::new(Stub));
        assert!(matches!(
            client.discovery().artist(0).await.unwrap_err(),
            Error::InvalidInput {
                field: "artist id",
                ..
            }
        ));
        assert!(matches!(
            client.discovery().artist_albums(1, 0, 0).await.unwrap_err(),
            Error::InvalidInput { .. }
        ));
        assert!(matches!(
            client.discovery().album(0).await.unwrap_err(),
            Error::InvalidInput {
                field: "album id",
                ..
            }
        ));
    }
}
