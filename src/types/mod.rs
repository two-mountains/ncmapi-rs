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
pub struct Artist {
    pub id: usize,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: usize,
    #[serde(default)]
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



#[cfg(test)]
mod tests {

    use super::{CloudSearchSong, ResultResp};
    use crate::{NcmApi, types::{PlaylistDetailResp, SongUrlResp, UserAccountResp, UserCloudResp, UserPlaylistResp}};

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
}
