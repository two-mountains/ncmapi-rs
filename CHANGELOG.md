# Changelog

All notable changes to this project are documented here.

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
