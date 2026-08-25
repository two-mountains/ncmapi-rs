<h1 align=center>ncmapi-rs</h1>

NetEase Cloud Music API for Rust.

`NcmClient` is the public client. `NcmApi` remains temporarily as a deprecated
type alias for migration only.


### Usage

```toml
[dependencies]
ncmapi = "0.1"
tokio = { version = "1", features = ["full"] }
```

```rust
use ncmapi::NcmClient;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error:Error>> {
    let api = NcmClient::default();
    let resp = api.cloud_search("mota", None).await;

    let res = resp.unwrap().deserialize_to_implict();
    println!("{:#?}", res);

    Ok(())
}
```


### Document

Most of the functions are self documented. If there is some confusion about the params of a funtion requires, figure out [here](https://neteasecloudmusicapi.vercel.app)



### How it works

* api: export api functions.
* client:
    * takes an ApiRequst, process it into a Request by presenting it with header and encrypt the payload etc. And then send requests to the server, takes the response and then returns the ApiResponse back.
    * cache

### Contribute

If you think this package useful, please do make pull requests.

Run the deterministic test suite with `cargo test`. Network checks are opt-in:
`cargo test --features live-tests`.

### License

[MIT](LICENSE)
