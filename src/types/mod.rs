// This module was generated at https://transform.tools/json-to-rust-serde
// However, some fields of struct was stripped for concision.
pub type SearchSongResp = ResultResp<SearchResultSong>;
pub type SearchArtistResp = ResultResp<SearchResultArtist>;
pub type SearchPodcastResp = ResultResp<SearchResultPodcast>;
pub type SearchPlaylistResp = ResultResp<SearchResultPlaylist>;
pub type SearchAlbumResp = ResultResp<SearchResultAlbum>;

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultResp<T> {
    pub code: usize,
    pub result: Option<T>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchResultSong {
    pub songs: Vec<Song>,
    pub has_more: bool,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: usize,
    pub name: String,
    #[serde(alias = "ar")]
    pub artists: Vec<Artist>,
    #[serde(alias = "al")]
    pub album: Album,
    #[serde(alias = "dt")]
    pub duration: usize,
    pub fee: usize,
    #[serde(alias = "popularity")]
    pub pop: f32,
    // pub resource_state: bool,
    // pub publish_time: i64,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: usize,
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: usize,
    pub name: Option<String>,
    #[serde(default)]
    pub pic_url: String,
    pub pic: usize,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodcastAudio {
    pub main_song: Song,
    pub dj: UserProfile,
    pub liked_count: usize,
    pub comment_count: usize,
}

/// User created podcasts
#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPodcastsResp {
    pub code: usize,
    #[serde(default)]
    pub dj_radios: Vec<Podcast>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodcastAudiosResp {
    pub code: usize,
    #[serde(default)]
    pub programs: Vec<PodcastAudio>,
    #[serde(default)]
    pub more: bool,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub user_id: usize,
    pub nickname: String,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAccountResp {
    pub code: usize,
    pub profile: Option<UserProfile>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
pub struct UserPlaylistResp {
    pub code: usize,
    #[serde(default)]
    pub playlist: Vec<Playlist>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
pub struct PlaylistDetailResp {
    pub code: usize,
    pub playlist: Option<PlaylistDetail>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistDetail {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub tracks: Vec<Song>,
    #[serde(default)]
    pub track_ids: Vec<Id>,
    pub user_id: usize,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
pub struct Id {
    pub id: usize,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
pub struct SongUrlResp {
    pub code: usize,
    #[serde(default)]
    pub data: Vec<SongUrl>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
pub struct SongUrl {
    pub id: usize,
    pub url: String,
    pub br: usize,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCloudResp {
    pub code: usize,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub data: Vec<CloudSongMeta>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSongMeta {
    pub simple_song: Song,
    pub song_id: usize,
    pub song_name: String,
    pub add_time: i128,
    pub file_size: usize,
    pub bitrate: usize,
    pub file_name: String,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedSongs {
    #[serde(default)]
    pub daily_songs: Vec<Song>,
    #[serde(default)]
    pub order_songs: Vec<Song>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
pub struct RecommendedSongsResp {
    pub code: usize,
    pub data: RecommendedSongs,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub user: UserProfile,
    #[serde(default)]
    pub content: String,
    pub time: u64,
    pub liked_count: usize,
    pub liked: bool,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceComments {
    #[serde(default)]
    pub comments: Vec<Comment>,
    pub total_count: usize,
    pub has_more: bool,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceCommentsResp {
    pub code: usize,
    pub data: ResourceComments,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotCommentsResp {
    pub code: usize,
    #[serde(default)]
    pub hot_comments: Vec<Comment>,
    pub has_more: bool,
    pub total: usize,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricResp {
    pub code: usize,
    pub sgc: bool,
    pub sfy: bool,
    pub qfy: bool,
    pub lrc: Option<Lyric>,
    pub klyric: Option<Lyric>,
    pub tlyric: Option<Lyric>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lyric {
    #[serde(default)]
    pub version: usize,
    #[serde(default)]
    pub lyric: String,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalFmResp {
    pub code: usize,
    #[serde(default)]
    pub data: Vec<Song>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedPlaylistsResp {
    pub code: usize,
    #[serde(default)]
    pub recommend: Vec<Playlist>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimiSongsResp {
    pub code: usize,
    #[serde(default)]
    pub songs: Vec<Song>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistSongsResp {
    pub code: usize,
    #[serde(default)]
    pub songs: Vec<Song>,
    #[serde(default)]
    pub more: bool,
    #[serde(default)]
    pub total: usize,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistSublistResp {
    pub code: usize,
    #[serde(default)]
    pub data: Vec<Artist>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Podcast {
    pub id: usize,
    pub name: String,
    pub desc: String,
    pub sub_count: usize,
    pub category: String,
    pub dj: UserProfile,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultArtist {
    #[serde(default)]
    pub artists: Vec<Artist>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultPodcast {
    #[serde(default)]
    pub dj_radios: Vec<Podcast>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultPlaylist {
    #[serde(default)]
    pub playlists: Vec<Playlist>,
}

#[derive(Default, Debug, Clone,  serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultAlbum {
    #[serde(default)]
    pub albums: Vec<Album>,
}

#[cfg(test)]
mod tests {

    use serde_json::json;

    use crate::{
        types::{
            ArtistSongsResp, ArtistSublistResp, HotCommentsResp, LyricResp, PersonalFmResp,
            PlaylistDetailResp, PodcastAudiosResp, RecommendedPlaylistsResp, RecommendedSongsResp,
            ResourceCommentsResp, SearchAlbumResp, SearchArtistResp, SearchPlaylistResp,
            SearchPodcastResp, SearchSongResp, SimiSongsResp, SongUrlResp, UserAccountResp,
            UserCloudResp, UserPlaylistResp, UserPodcastsResp,
        },
        NcmApi, SearchType,
    };

    // let res = resp.unwrap();
    // let mut f = std::fs::OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .open("test-data/search_podcast.json")
    //     .unwrap();
    // f.write_all(res.data()).unwrap();

    #[tokio::test]
    async fn test_de_search_song() {
        let api = NcmApi::default();
        let resp = api.search("xusong", None).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<SearchSongResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_search_artist() {
        let api = NcmApi::default();
        let resp = api
            .search("xusong", Some(json!({ "type": SearchType::Artist })))
            .await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<SearchArtistResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_search_playlist() {
        let api = NcmApi::default();
        let resp = api
            .search("ost", Some(json!({ "type": SearchType::Collection })))
            .await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<SearchPlaylistResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_search_podcast() {
        let api = NcmApi::default();
        let resp = api
            .search("asmr", Some(json!({ "type": SearchType::Podcast })))
            .await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<SearchPodcastResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_search_album() {
        let api = NcmApi::default();
        let resp = api
            .search("Mota", Some(json!({ "type": SearchType::Album })))
            .await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<SearchAlbumResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_user_account() {
        let api = NcmApi::default();
        let resp = api.user_account().await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<UserAccountResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_user_playlist() {
        let api = NcmApi::default();
        let uid = 49668844;
        let resp = api.user_playlist(uid, None).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<UserPlaylistResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_playlist_detail() {
        let api = NcmApi::default();
        let resp = api.playlist_detail(6591923961, None).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<PlaylistDetailResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_song_url() {
        let api = NcmApi::default();
        let resp = api.song_url(&vec![28497094, 28497093]).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<SongUrlResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_user_cloud() {
        let api = NcmApi::default();
        let resp = api.user_cloud(None).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<UserCloudResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_recommended_songs() {
        let api = NcmApi::default();
        let resp = api.recommend_songs().await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<RecommendedSongsResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_comments() {
        let api = NcmApi::default();
        let resp = api
            .comment(32977061, crate::api::ResourceType::Song, 10, 1, 1, 0, false)
            .await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<ResourceCommentsResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_hot_comments() {
        let api = NcmApi::default();
        let resp = api
            .comment_hot(32977061, crate::ResourceType::Song, None)
            .await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<HotCommentsResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_lyric() {
        let api = NcmApi::default();
        let resp = api.lyric(17346999).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<LyricResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_personal_fm() {
        let api = NcmApi::default();
        let resp = api.personal_fm().await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<PersonalFmResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_recommended_playlists() {
        let api = NcmApi::default();
        let resp = api.recommend_resource().await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<RecommendedPlaylistsResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_simi_songs() {
        let api = NcmApi::default();
        let resp = api.simi_song(347230, None).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<SimiSongsResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_artist_songs() {
        let api = NcmApi::default();
        let resp = api.artist_songs(6452, None).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<ArtistSongsResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_artist_sublist() {
        let api = NcmApi::default();
        let resp = api.artist_sublist(None).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<ArtistSublistResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_user_podcast() {
        let api = NcmApi::default();
        let resp = api.user_podcast(1398995370).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<UserPodcastsResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }

    #[tokio::test]
    async fn test_de_podcast_audio() {
        let api = NcmApi::default();
        let resp = api.podcast_audio(965114264, None).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<PodcastAudiosResp>(resp.unwrap().data()).unwrap();
        assert_eq!(res.code, 200);
    }
}
