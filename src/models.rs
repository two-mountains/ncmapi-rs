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

/// A playlist projection.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct Playlist {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    pub description: Option<String>,
}

/// The payload returned by a user's playlist query.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct UserPlaylistsBody {
    #[serde(default)]
    pub playlist: Vec<Playlist>,
}
