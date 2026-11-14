//! Stable domain types returned by the supported SDK services.

use serde::Deserialize;

/// A response returned by the NetEase API.
///
/// The API keeps endpoint-specific fields beside `code`; those fields are
/// captured by the strongly typed `body` rather than exposed as unstructured
/// JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    #[serde(flatten)]
    pub body: T,
}

impl<T> ApiResponse<T> {
    /// Returns whether the remote service accepted the request.
    pub const fn is_success(&self) -> bool {
        self.code == 200
    }
}

/// A stable track identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TrackId(pub u64);

/// A stable user identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct UserId(pub u64);

/// A compact artist projection shared by search and track results.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: u64,
    #[serde(default)]
    pub name: String,
}

/// A compact album projection shared by search and track results.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub pic_url: String,
}

/// Track metadata returned by search and library queries.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default, alias = "ar")]
    pub artists: Vec<Artist>,
    #[serde(default, alias = "al")]
    pub album: Album,
    #[serde(default, alias = "dt")]
    pub duration: u32,
}

/// The payload of a song search response.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct SearchBody {
    #[serde(default)]
    pub result: SearchResults,
}

/// Search results that are common across search modes.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    #[serde(default)]
    pub songs: Vec<Track>,
    #[serde(default)]
    pub playlists: Vec<Playlist>,
    #[serde(default)]
    pub artists: Vec<Artist>,
    #[serde(default)]
    pub has_more: bool,
}

/// The payload of a track-details response.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct TrackDetailsBody {
    #[serde(default)]
    pub songs: Vec<Track>,
}

/// A resolved playable stream.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct StreamUrl {
    pub id: u64,
    pub url: Option<String>,
    #[serde(default)]
    pub br: u32,
}

/// The payload of a stream URL response.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct StreamUrlsBody {
    #[serde(default)]
    pub data: Vec<StreamUrl>,
}

/// A user profile projection.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub user_id: u64,
    #[serde(default)]
    pub nickname: String,
}

/// The payload returned for the authenticated account.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct AccountBody {
    pub profile: Option<UserProfile>,
}

/// The payload returned for a public user profile.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct UserDetailsBody {
    pub profile: Option<UserProfile>,
}

/// The account that owns a playlist.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub nickname: String,
}

/// A playlist projection shared by search and user-library results.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub track_count: u64,
    #[serde(default)]
    pub cover_img_url: String,
    #[serde(default)]
    pub creator: Option<Creator>,
}

/// A lightweight track reference used by playlist detail payloads.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct TrackRef {
    pub id: u64,
}

/// A playlist with its first page of tracks.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistDetail {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub track_count: u64,
    #[serde(default)]
    pub cover_img_url: String,
    #[serde(default)]
    pub creator: Option<Creator>,
    #[serde(default)]
    pub tracks: Vec<Track>,
    /// Every track id in the playlist, in order. `tracks` only carries the
    /// first page, so this is the authoritative listing.
    #[serde(default)]
    pub track_ids: Vec<TrackRef>,
}

/// The payload of a playlist-detail response.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct PlaylistDetailBody {
    #[serde(default)]
    pub playlist: Option<PlaylistDetail>,
}

/// One lyric document. NetEase returns LRC-formatted text, optionally with
/// translation and romanisation in sibling fields.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct Lyric {
    #[serde(default)]
    pub lyric: String,
}

/// The payload of a lyric response.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct LyricsBody {
    #[serde(default)]
    pub lrc: Option<Lyric>,
    #[serde(default)]
    pub tlyric: Option<Lyric>,
    #[serde(default)]
    pub romalrc: Option<Lyric>,
}

impl LyricsBody {
    /// The original lyric text, or an empty string when the track has none.
    #[must_use]
    pub fn text(&self) -> &str {
        self.lrc.as_ref().map_or("", |lyric| lyric.lyric.as_str())
    }

    /// The translated lyric text, when available.
    #[must_use]
    pub fn translation(&self) -> Option<&str> {
        self.tlyric
            .as_ref()
            .map(|lyric| lyric.lyric.as_str())
            .filter(|text| !text.is_empty())
    }
}

/// One entry of the daily recommendation reasons.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecommendReason {
    #[serde(default)]
    pub song_id: u64,
    #[serde(default)]
    pub reason: Option<String>,
}

/// The payload of daily recommendations.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DailyData {
    #[serde(default)]
    pub daily_songs: Vec<Track>,
    #[serde(default)]
    pub recommend_reasons: Vec<RecommendReason>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct DailyBody {
    #[serde(default)]
    pub data: DailyData,
}

/// The payload of a personal-FM batch.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct FmBody {
    #[serde(default)]
    pub data: Vec<Track>,
}

/// The payload returned by the liked-ids endpoint.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct LikeListBody {
    #[serde(default)]
    pub ids: Vec<u64>,
}

/// A chart entry from the toplist endpoint.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToplistEntry {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub cover_img_url: String,
    #[serde(default)]
    pub track_count: u64,
    #[serde(default)]
    pub update_frequency: String,
}

/// The payload of the toplist endpoint.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct ToplistBody {
    #[serde(default)]
    pub list: Vec<ToplistEntry>,
}

/// Placeholder body for endpoints that only return a status code.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct EmptyBody {}

/// The payload that starts a QR login flow.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct QrKeyBody {
    #[serde(default)]
    pub unikey: String,
}

impl QrKeyBody {
    /// The URL to encode as a QR code for the NetEase mobile app.
    #[must_use]
    pub fn login_url(&self) -> String {
        format!("https://music.163.com/login?codekey={}", self.unikey)
    }
}

/// The payload polled while a QR login is in progress.
///
/// The status code lives on [`ApiResponse::code`]:
/// `800` expired, `801` waiting, `802` scanned, `803` confirmed.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QrLoginBody {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

/// The payload returned by a user's playlist query.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct UserPlaylistsBody {
    #[serde(default)]
    pub playlist: Vec<Playlist>,
}
