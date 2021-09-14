// This module was generated at https://transform.tools/json-to-rust-serde
// However, some fields of struct was stripped for concision.
#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultResp<T> {
    pub code: usize,
    pub result: Option<T>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CloudSearchSong {
    pub songs: Vec<Song>,
    pub has_more: bool,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub name: String,
    pub id: usize,
    #[serde(rename = "ar")]
    pub artists: Vec<Artist>,
    #[serde(rename = "al")]
    pub album: Album,
    #[serde(rename = "dt")]
    pub duration: usize,
    pub fee: i64,
    pub pop: f32,
    // pub resource_state: bool,
    // pub publish_time: i64,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongVerbose {
    pub name: String,
    pub id: usize,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub duration: usize,
    pub fee: i64,
    pub popularity: f32,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: usize,
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: usize,
    pub name: Option<String>,
    #[serde(default)]
    pub pic_url: String,
    pub pic: usize,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub user_id: usize,
    pub nickname: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAccountResp {
    pub code: usize,
    pub profile: Option<UserProfile>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UserPlaylistResp {
    pub code: usize,
    #[serde(default)]
    pub playlist: Vec<Playlist>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PlaylistDetailResp {
    pub code: usize,
    pub playlist: Option<PlaylistDetail>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: usize,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Id {
    pub id: usize,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SongUrlResp {
    pub code: usize,
    #[serde(default)]
    pub data: Vec<SongUrl>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SongUrl {
    pub id: usize,
    pub url: String,
    pub br: usize,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCloudResp {
    pub code: usize,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub data: Vec<CloudSongMeta>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedSongs {
    #[serde(default)]
    pub daily_songs: Vec<Song>,
    #[serde(default)]
    pub order_songs: Vec<Song>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecommendedSongsResp {
    pub code: usize,
    pub data: RecommendedSongs,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub user: UserProfile,
    #[serde(default)]
    pub content: String,
    pub time: u64,
    pub liked_count: usize,
    pub liked: bool,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceComments {
    #[serde(default)]
    pub comments: Vec<Comment>,
    pub total_count: usize,
    pub has_more: bool,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceCommentsResp {
    pub code: usize,
    pub data: ResourceComments,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotCommentsResp {
    pub code: usize,
    #[serde(default)]
    pub hot_comments: Vec<Comment>,
    pub has_more: bool,
    pub total: usize,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lyric {
    #[serde(default)]
    pub version: usize,
    #[serde(default)]
    pub lyric: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalFmResp {
    pub code: usize,
    #[serde(default)]
    pub data: Vec<SongVerbose>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedPlaylistsResp {
    pub code: usize,
    #[serde(default)]
    pub recommend: Vec<Playlist>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimiSongsResp {
    pub code: usize,
    #[serde(default)]
    pub songs: Vec<SongVerbose>,
}

#[cfg(test)]
mod tests {
    use super::{CloudSearchSong, ResultResp};
    use crate::{
        types::{
            HotCommentsResp, LyricResp, PersonalFmResp, PlaylistDetailResp,
            RecommendedPlaylistsResp, RecommendedSongsResp, ResourceCommentsResp, SimiSongsResp,
            SongUrlResp, UserAccountResp, UserCloudResp, UserPlaylistResp,
        },
        NcmApi,
    };

    type CloudSearchSongResp = ResultResp<CloudSearchSong>;

    #[tokio::test]
    async fn test_de_cloud_search_song() {
        let api = NcmApi::default();
        let resp = api.cloud_search("xusong", None).await;
        assert!(resp.is_ok());

        let res = serde_json::from_slice::<CloudSearchSongResp>(resp.unwrap().data()).unwrap();
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

    // let res = resp.unwrap();
    // let mut f = std::fs::OpenOptions::new().create(true).write(true).open("test-data/comments.json").unwrap();
    // f.write_all(res.data()).unwrap();

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
}
