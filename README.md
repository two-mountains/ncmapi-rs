<h1 align=center>ncmapi-rs</h1>

NetEase Cloud Music API for Rust. Pure-Rust crypto (no OpenSSL), rustls for
TLS, and a transport that never caches mutations.

`NcmClient` is the sole public SDK entry point. `NcmApi` remains only as a
deprecated alias for a short migration window.


### Usage

```toml
[dependencies]
ncmapi = "1"
tokio = { version = "1", features = ["full"] }
```

```rust
use ncmapi::{NcmClient, SearchKind, SearchQuery};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error:Error>> {
    let api = NcmClient::builder().build()?;
    let response = api
        .discovery()
        .search(SearchQuery::new("mota")?.kind(SearchKind::Artist))
        .await?;

    for song in response.body.result.songs {
        println!("{} — {}", song.id, song.name);
    }

    Ok(())
}
```


### Architecture

The public surface is intentionally small:

- `client.discovery()` searches the catalogue (tracks, playlists, artists).
- `client.library()` reads tracks, stream URLs, playlists and lyrics.
- `client.profile()` reads account and user information.
- `client.auth()` starts and polls QR logins.

Each service accepts value types and returns an `ApiResponse<T>` whose body is
a stable domain model. Services create immutable request plans; `NcmClient`
delegates them to an internal transport that owns protocol encryption, cookies,
caching, and HTTP I/O. This keeps the domain layer deterministic and lets the
test suite exercise it without accessing the external music service.

`1.0` removes the former `NcmApi` endpoint methods. Migrate calls to a focused
service, for example `client.discovery().search(SearchQuery::new("mota")?)`.

### Playlists, lyrics and QR login

```rust
let detail = api.library().playlist_detail(24381616).await?;
let tracks = api.library().playlist_tracks(24381616, 500, 0).await?;
let lyrics = api.library().lyrics(track_id).await?;

let daily = api.discovery().daily().await?;       // session
let charts = api.discovery().toplists().await?;   // no session
let fm = api.radio().fm_batch().await?;           // session
api.radio().set_like(song_id, true).await?;
api.radio().trash(song_id).await?;
let liked = api.radio().liked_ids(user_id).await?;

let qr = api.auth().qr_start().await?;
println!("{}", qr.body.login_url());          // encode this as a QR code
let status = api.auth().qr_poll(&qr.body.unikey).await?; // code 803 = confirmed
```

### Contribute

If you think this package useful, please do make pull requests.

Run the deterministic test suite with `cargo test`. The SDK deliberately does
not run external-service checks in its normal development workflow.

### License

[MIT](LICENSE)
