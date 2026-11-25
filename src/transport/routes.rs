//! The small, audited route surface supported by this SDK.

pub(crate) const CLOUD_SEARCH: &str = "https://music.163.com/api/cloudsearch/pc";
pub(crate) const TRACK_DETAILS: &str = "https://music.163.com/api/v3/song/detail";
pub(crate) const STREAM_URLS: &str = "https://interface3.music.163.com/api/song/enhance/player/url";
pub(crate) const ACCOUNT: &str = "https://music.163.com/api/nuser/account/get";
pub(crate) const USER_PLAYLISTS: &str = "https://music.163.com/api/user/playlist";
pub(crate) const PLAYLIST_DETAIL: &str = "https://music.163.com/api/v6/playlist/detail";
pub(crate) const LYRIC: &str = "https://music.163.com/api/song/lyric";
pub(crate) const QR_UNIKEY: &str = "https://music.163.com/eapi/login/qrcode/unikey";
pub(crate) const QR_LOGIN: &str = "https://music.163.com/eapi/login/qrcode/client/login";
pub(crate) const RECOMMEND_SONGS: &str = "https://music.163.com/api/v3/discovery/recommend/songs";
pub(crate) const PERSONAL_FM: &str = "https://music.163.com/weapi/v1/radio/get";
pub(crate) const LIKE: &str = "https://music.163.com/api/radio/like";
pub(crate) const LIKELIST: &str = "https://music.163.com/weapi/song/like/get";
pub(crate) const TOPLIST: &str = "https://music.163.com/api/toplist";
pub(crate) const USER_CLOUD: &str = "https://music.163.com/api/v1/cloud/get";
pub(crate) const COMMENTS: &str = "https://music.163.com/api/v1/resource/comments/R_SO_4_";
pub(crate) const PLAYLIST_CREATE: &str = "https://music.163.com/api/playlist/create";
pub(crate) const PLAYLIST_RENAME: &str = "https://music.163.com/api/playlist/update/name";
pub(crate) const PLAYLIST_DESCRIBE: &str = "https://music.163.com/api/playlist/desc/update";
pub(crate) const PLAYLIST_MANIPULATE: &str = "https://music.163.com/api/playlist/manipulate/tracks";
pub(crate) const PLAYLIST_REMOVE: &str = "https://music.163.com/api/playlist/remove";

pub(crate) fn fm_trash(song_id: u64, time: u64) -> String {
    format!("https://music.163.com/weapi/radio/trash/add?alg=RT&songId={song_id}&time={time}")
}

pub(crate) fn user_details(user_id: u64) -> String {
    format!("https://music.163.com/api/v1/user/detail/{user_id}")
}

pub(crate) fn artist_detail(artist_id: u64) -> String {
    format!("https://music.163.com/api/artist/head/info/get?id={artist_id}")
}

pub(crate) fn artist_albums(artist_id: u64, limit: u16, offset: u32) -> String {
    format!("https://music.163.com/api/artist/albums/{artist_id}?limit={limit}&offset={offset}")
}

pub(crate) fn album_detail(album_id: u64) -> String {
    format!("https://music.163.com/api/v1/album/{album_id}")
}
