<h1 align=center>ncmapi-rs</h1>

NetEase Cloud Music API for Rust.

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

- `client.discovery()` searches the catalogue.
- `client.library()` reads tracks and stream URLs.
- `client.profile()` reads account and user information.

Each service accepts value types and returns an `ApiResponse<T>` whose body is
a stable domain model. Services create immutable request plans; `NcmClient`
delegates them to an internal transport that owns protocol encryption, cookies,
caching, and HTTP I/O. This keeps the domain layer deterministic and lets the
test suite exercise it without accessing the external music service.

`1.0` removes the former `NcmApi` endpoint methods. Migrate calls to a focused
service, for example `client.discovery().search(SearchQuery::new("mota")?)`.

### Contribute

If you think this package useful, please do make pull requests.

Run the deterministic test suite with `cargo test`. The SDK deliberately does
not run external-service checks in its normal development workflow.

### License

[MIT](LICENSE)
