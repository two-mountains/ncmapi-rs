# Changelog

All notable changes to this project are documented here.

## 1.10.5

### Changed

- Every playlist write drops the response cache itself, so callers cannot
  forget to invalidate it: create, update, add, remove and delete all clear
  after the attempt. `NcmClient::clear_cache()` stays public for other
  write-then-read flows.

## 1.10.4

### Added

- `NcmClient::clear_cache()` drops every cached response. Callers that write
  through the SDK must use it so a read-after-write (for example a playlist
  list right after creating one) sees the change instead of the cached answer.

## 1.10.3

### Fixed

- Only the provider's duplicate-track `502 歌单内歌曲重复` is treated as success
  on add; any other `502` is returned to the caller. Tests for a create response
  without an id and for a non-200 create are included in the published package.

## 1.10.2

### Fixed

- `playlist_update` returns the first failing response instead of the last, so
  a rejected rename is no longer hidden by a successful description change, and
  an update with neither field is `InvalidInput`.
- `playlist_add_tracks` treats the provider's `502 歌单内歌曲重复` as success, so
  adding an already-present track and retrying are harmless as documented.

## 1.10.1

### Fixed

- `playlist_update` now uses the provider's real rename and description
  routes (`/api/playlist/update/name`, `/api/playlist/desc/update`); the
  combined route answered `404 接口未找到`.
- `playlist_delete` posts to `/api/playlist/remove` with `ids` as a JSON array
  string; `/api/playlist/delete` with `id` answered `400 请求参数错误`.

## 1.10.0

### Added

- Playlist writes: `Library::playlist_create`, `playlist_update`,
  `playlist_add_tracks`, `playlist_remove_tracks` and `playlist_delete`, with
  the `PlaylistCreatedBody` response model. Add/remove are idempotent on the
  provider, so retries are safe; create returns the new id.

## 1.9.1

### Added

- `Album::artist` and `Album::description`, so album search and detail map the
  nested artist and description without a second request.

## 1.9.0

### Added

- Album search results: `SearchResults::albums` and `album_count`.
- `Discovery::artist`, `Discovery::artist_albums` and `Discovery::album`
  for public artist profiles, artist album pages and album track lists.
- `Album::publish_time` and `Album::size`; all new fields are null-tolerant.

## 1.8.1

### Fixed

- Track comment payloads tolerate `null` where the provider omits deleted
  content: `Comment::content`, `beReplied[*].content`, user `nickname` and
  `avatarUrl`, plus nullable `total`, `more`, `hotComments` and `comments`.
  Previously a single deleted reply failed the whole comments response with
  `invalid type: null, expected a string`.

## 1.8.0

### Added

- `NcmClientBuilder::device_id_path`: persists one anonymous device id (52 hex
  characters) across processes instead of generating a new device for every
  client. This keeps repeated CLI invocations recognisable to the provider.
- `NcmClientBuilder::user_agent`: overrides the default `ncmapi-rs/1.0` header
  so applications can present a browser or app string.

### Changed

- Requests to `music.163.com` now send `Origin` and `Accept-Language` alongside
  `Referer`, matching a normal web client more closely.

### Fixed

- Cookie and device-id files are created with `0600` permissions on Unix, and
  their parent directory is created when missing.

## 1.7.0

### Added

- `Library::comments(track_id, limit, offset)` and `CommentsBody` /
  `Comment` / `CommentUser` / `RepliedComment` models. Uses a new plain
  read-only GET transport path; public comments need no encryption.

## 1.6.0

### Changed

- Protocol crypto is pure Rust: AES-128-ECB/CBC (`aes` + `cbc`), MD5 (`md-5`)
  and raw RSA (`rsa`) replace `openssl`. Ciphertext fixtures are unchanged, so
  the wire protocol is identical. Removes the system/Homebrew OpenSSL
  requirement and makes cross-compilation (macOS x86_64, Linux arm64) trivial.

## 1.5.0

### Changed

- `ApiResponse.code` is now `i32` and responses expose the provider `message`.
  NetEase returns negative codes such as `-460` for refused actions, which the
  previous `u16` could not represent (the body failed to decode).

## 1.4.1

### Fixed

- `ToplistEntry` is deserialized by hand so duplicate `updateFrequency` keys
  (present in the live chart payload) no longer abort the whole list.

## 1.4.0

### Added

- `library().cloud(limit, offset)` — the account's music cloud (uploaded
  tracks), with `CloudBody`/`CloudTrack` models and matched `simpleSong`
  metadata.

## 1.3.1

### Fixed

- Provider models accept `null` for strings, numbers, vectors and nested
  objects. NetEase returns `null` names on locally-uploaded tracks, which made
  playlist detail fail to deserialize (`invalid type: null, expected a string`).

## 1.3.0

### Changed

- QR login uses the new-client eapi protocol: `/eapi/login/qrcode/*`, `type: 3`,
  `noCheckToken: true`, with a generated device id in the eapi header. The legacy
  weapi `type: 1` flow is rejected by NetEase with `8821` (behavioral
  verification) after the user confirms.

## 1.2.1

### Fixed

- QR polling, FM batches, likes, trashes, liked-id reads and QR key creation are
  now exempt from the short-lived response cache. Polling used to replay the
  first `801 等待扫码` for the whole cache TTL, so confirmation (`803`) was never
  observed, and repeated like/trash calls returned stale results.

## 1.2.0

### Added

- `discovery().daily()` — daily recommended songs with reasons (session).
- `discovery().toplists()` — official chart list (no session).
- `radio().fm_batch()` — personal FM batch (session).
- `radio().trash(id)` — remove a track from personal FM.
- `radio().set_like(id, liked)` — idempotent like/unlike.
- `radio().liked_ids(user)` — liked track ids.
- Models: `DailyBody`, `DailyData`, `RecommendReason`, `FmBody`, `LikeListBody`,
  `ToplistBody`, `ToplistEntry`, `EmptyBody`.

## 1.1.2

### Fixed

- Correct the QR login test stub to assert the unikey route for `qr_start` and
  the check route for `qr_poll`.

## 1.1.1

### Fixed

- QR login now uses the `/weapi/login/qrcode/*` routes. The `/api/*` paths
  return scan state but never observe confirmation, so polling waited forever.

## 1.1.0

### Added

- `library().playlist_detail(id)` — playlist metadata plus its first page of
  tracks and the complete ordered `trackIds` list.
- `library().playlist_tracks(id, limit, offset)` — pages through `trackIds` and
  hydrates the page in batches through the song-detail endpoint.
- `library().lyrics(id)` — LRC text with optional translation and romanisation.
- `auth().qr_start()` / `auth().qr_poll(key)` — QR login; confirmation persists
  the session cookies through the existing transport.
- `QrKeyBody::login_url()` helper for rendering the QR payload.
- Search results now include playlists and artists.
- Playlist models expose `trackCount`, `coverImgUrl` and `creator`.

### Fixed

- `/api/playlist/track/all` returns 404; playlist paging no longer relies on it.
- Enable the `reqwest` `socks` feature so proxy environments work.

## 1.0.0

- Service-based public API (`NcmClient`) with `Discovery`, `Library` and
  `Profile`; stable domain models; discontinued endpoint methods on `NcmApi`.
