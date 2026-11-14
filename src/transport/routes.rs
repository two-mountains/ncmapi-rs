//! The small, audited route surface supported by this SDK.

pub(crate) const CLOUD_SEARCH: &str = "https://music.163.com/api/cloudsearch/pc";
pub(crate) const TRACK_DETAILS: &str = "https://music.163.com/api/v3/song/detail";
pub(crate) const STREAM_URLS: &str = "https://interface3.music.163.com/api/song/enhance/player/url";
pub(crate) const ACCOUNT: &str = "https://music.163.com/api/nuser/account/get";
pub(crate) const USER_PLAYLISTS: &str = "https://music.163.com/api/user/playlist";
pub(crate) const PLAYLIST_DETAIL: &str = "https://music.163.com/api/v6/playlist/detail";
pub(crate) const LYRIC: &str = "https://music.163.com/api/song/lyric";
pub(crate) const QR_UNIKEY: &str = "https://music.163.com/weapi/login/qrcode/unikey";
pub(crate) const QR_LOGIN: &str = "https://music.163.com/weapi/login/qrcode/client/login";
pub(crate) const RECOMMEND_SONGS: &str = "https://music.163.com/api/v3/discovery/recommend/songs";
pub(crate) const PERSONAL_FM: &str = "https://music.163.com/weapi/v1/radio/get";
pub(crate) const LIKE: &str = "https://music.163.com/api/radio/like";
pub(crate) const LIKELIST: &str = "https://music.163.com/weapi/song/like/get";
pub(crate) const TOPLIST: &str = "https://music.163.com/api/toplist";

pub(crate) fn fm_trash(song_id: u64, time: u64) -> String {
    format!("https://music.163.com/weapi/radio/trash/add?alg=RT&songId={song_id}&time={time}")
}

pub(crate) fn user_details(user_id: u64) -> String {
    format!("https://music.163.com/api/v1/user/detail/{user_id}")
}
