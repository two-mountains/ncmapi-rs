# Changelog

All notable changes to this project are documented here.

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
