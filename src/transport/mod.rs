//! Reqwest-backed infrastructure adapter.

mod cache;
mod request;
mod response;
mod routes;

use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use cache::Store;
use cookie::Cookie;
use rand::RngExt;
use reqwest::{
    Client, Request, Response as HttpResponse, Url,
    cookie::{CookieStore, Jar},
    header::{
        ACCEPT_LANGUAGE, CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue, ORIGIN, REFERER, SET_COOKIE,
        USER_AGENT,
    },
};
use serde_json::{Value, json};

use crate::{
    Error, TResult,
    crypto::{Crypto, eapi, linuxapi, weapi},
    domain::request_id,
};

pub(crate) use request::{RequestMethod, RequestPlan, RequestPlanBuilder};
pub(crate) use response::Response;
pub(crate) use routes::{
    ACCOUNT, CLOUD_SEARCH, COMMENTS, LIKE, LIKELIST, LYRIC, PERSONAL_FM, PLAYLIST_CREATE,
    PLAYLIST_DELETE, PLAYLIST_DETAIL, PLAYLIST_MANIPULATE, PLAYLIST_UPDATE, QR_LOGIN, QR_UNIKEY,
    RECOMMEND_SONGS, STREAM_URLS, TOPLIST, TRACK_DETAILS, USER_CLOUD, USER_PLAYLISTS, album_detail,
    artist_albums, artist_detail, fm_trash, user_details,
};

/// Side-effect boundary used by endpoint use-cases.
///
/// Production uses [`HttpTransport`]; tests can substitute a deterministic
/// in-memory implementation without opening a socket.
#[async_trait]
pub(crate) trait Transport: Send + Sync {
    async fn execute(&self, request: RequestPlan) -> TResult<Response>;
}

pub(crate) struct HttpTransport {
    config: TransportConfig,
    client: Client,
    store: Store,
    jar: Arc<dyn CookieStore>,
    /// Stable device id used by eapi request headers.
    ///
    /// Persisted when [`HttpTransportBuilder::device_id_path`] is set, so
    /// separate CLI invocations keep presenting the same device.
    device_id: String,
    user_agent: HeaderValue,
}

#[derive(Debug)]
pub(crate) struct HttpTransportBuilder {
    config: TransportConfig,
}

impl HttpTransportBuilder {
    pub(crate) fn new(cookie_path: &Path) -> TResult<Self> {
        let base_url = BASE_URL
            .parse::<Url>()
            .map_err(|error| Error::InvalidUrl(error.to_string()))?;

        Ok(Self {
            config: TransportConfig {
                cache: true,
                cache_ttl: Duration::from_secs(3 * 60),
                base_url,
                preserve_cookies: true,
                cookie_path: cookie_path.to_path_buf(),
                device_id_path: None,
                user_agent: DEFAULT_USER_AGENT.to_owned(),
            },
        })
    }

    /// Persists the anonymous device id so it survives across processes.
    pub(crate) fn device_id_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.device_id_path = Some(path.into());
        self
    }

    /// Overrides the `User-Agent` header sent with every request.
    pub(crate) fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    pub(crate) fn cache(mut self, enabled: bool) -> Self {
        self.config.cache = enabled;
        self
    }

    pub(crate) fn cache_ttl(mut self, ttl: Duration) -> Self {
        self.config.cache_ttl = ttl;
        self
    }

    pub(crate) fn persist_cookies(mut self, enabled: bool) -> Self {
        self.config.preserve_cookies = enabled;
        self
    }

    pub(crate) fn build(self) -> TResult<HttpTransport> {
        let config = self.config;
        let jar = Arc::new(Jar::default());

        match read_cookies(&config.cookie_path) {
            Ok(cookies) if config.preserve_cookies && !cookies.is_empty() => {
                let headers = cookies
                    .split("; ")
                    .map(HeaderValue::from_str)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| Error::Protocol(error.to_string()))?;
                jar.set_cookies(&mut headers.iter(), &config.base_url);
            }
            Ok(_) => {}
            Err(Error::CookiePersistence { source, .. })
                if source.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }

        let user_agent =
            HeaderValue::from_str(&config.user_agent).map_err(|_| Error::InvalidInput {
                field: "user_agent",
                reason: "must be a valid HTTP header value",
            })?;
        let device_id = load_device_id(config.device_id_path.as_deref())?;

        Ok(HttpTransport {
            store: Store::new(config.cache_ttl),
            config,
            client: Client::builder()
                .cookie_store(false)
                .build()
                .map_err(Error::Http)?,
            jar,
            device_id,
            user_agent,
        })
    }
}

impl HttpTransport {
    async fn execute_request(&self, request: RequestPlan) -> TResult<Response> {
        let id = request.id()?;
        let cacheable = request.cacheable();
        if self.config.cache
            && cacheable
            && let Some(response) = self.store.get(&id)
        {
            return Ok(response);
        }

        let request = self.to_http_request(request)?;
        let response = self.client.execute(request).await.map_err(Error::Http)?;
        self.on_response(id, response, cacheable).await
    }

    async fn on_response(
        &self,
        id: String,
        response: HttpResponse,
        cacheable: bool,
    ) -> TResult<Response> {
        let mut headers = response.headers().get_all(SET_COOKIE).iter().peekable();
        if headers.peek().is_some() {
            self.jar.set_cookies(&mut headers, response.url());
            if self.config.preserve_cookies {
                let header = self.jar.cookies(&self.config.base_url).ok_or_else(|| {
                    Error::Protocol("cookie jar did not yield a persisted header".to_owned())
                })?;
                let serialized = header
                    .to_str()
                    .map_err(|error| Error::Protocol(error.to_string()))?;
                write_cookies(&self.config.cookie_path, serialized)?;
            }
        }

        let response = Response::new(response.bytes().await.map_err(Error::Http)?.to_vec());
        if self.config.cache && cacheable {
            self.store.insert(id, response.clone());
        }
        Ok(response)
    }

    fn to_http_request(&self, plan: RequestPlan) -> TResult<Request> {
        let plan = plan.into_parts();
        let mut payload = plan.payload;
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, self.user_agent.clone());
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
        );
        if plan.url.contains("music.163.com") {
            headers.insert(REFERER, HeaderValue::from_static(BASE_URL));
            headers.insert(ORIGIN, HeaderValue::from_static(BASE_URL));
        }

        match plan.encryption {
            Crypto::Weapi => {
                let mut cookies = self
                    .jar
                    .cookies(&self.config.base_url)
                    .map(|header| {
                        header
                            .to_str()
                            .map(str::to_owned)
                            .map_err(|error| Error::Protocol(error.to_string()))
                    })
                    .transpose()?
                    .unwrap_or_default();
                if !plan.cookies.is_empty() {
                    if !cookies.is_empty() {
                        cookies.push_str("; ");
                    }
                    cookies.push_str(&serialize_cookie_map(&plan.cookies));
                }
                headers.insert(
                    COOKIE,
                    HeaderValue::try_from(cookies)
                        .map_err(|error| Error::Protocol(error.to_string()))?,
                );
                let csrf = self
                    .cookie("__csrf", &self.config.base_url)?
                    .map(|cookie| cookie.value().to_owned())
                    .unwrap_or_default();
                payload
                    .as_object_mut()
                    .ok_or_else(|| {
                        Error::Protocol("weapi payload must be a JSON object".to_owned())
                    })?
                    .insert("csrf_token".to_owned(), Value::String(csrf));
            }
            Crypto::Eapi => {
                let mut cookies = self.eapi_header_cookies()?;
                cookies.extend(plan.cookies.clone());
                headers.insert(
                    COOKIE,
                    HeaderValue::try_from(serialize_cookie_map(&cookies))
                        .map_err(|error| Error::Protocol(error.to_string()))?,
                );
                payload
                    .as_object_mut()
                    .ok_or_else(|| {
                        Error::Protocol("eapi payload must be a JSON object".to_owned())
                    })?
                    .insert("header".to_owned(), json!(cookies));
            }
            Crypto::Linuxapi | Crypto::Plain => {
                let cookies = self
                    .jar
                    .cookies(&self.config.base_url)
                    .map(|header| {
                        header
                            .to_str()
                            .map(str::to_owned)
                            .map_err(|error| Error::Protocol(error.to_string()))
                    })
                    .transpose()?
                    .unwrap_or_default();
                headers.insert(
                    COOKIE,
                    HeaderValue::try_from(cookies)
                        .map_err(|error| Error::Protocol(error.to_string()))?,
                );
            }
        }

        let form = match plan.encryption {
            Crypto::Weapi => Some(weapi(payload.to_string().as_bytes())?.into_vec()),
            Crypto::Eapi => {
                let api_path = plan.api_path.ok_or_else(|| {
                    Error::Protocol("eapi request requires an API path".to_owned())
                })?;
                Some(eapi(api_path.as_bytes(), payload.to_string().as_bytes())?.into_vec())
            }
            Crypto::Linuxapi => {
                let data = json!({
                    "method": "POST",
                    "url": adapt_url(&plan.url, plan.encryption),
                    "params": payload,
                });
                Some(linuxapi(data.to_string().as_bytes())?.into_vec())
            }
            Crypto::Plain => None,
        };

        let method = match plan.method {
            RequestMethod::Get => reqwest::Method::GET,
            RequestMethod::Post => reqwest::Method::POST,
        };
        let request = self.client.request(
            method,
            adapt_url(&plan.url, plan.encryption)
                .parse::<Url>()
                .map_err(|error| Error::InvalidUrl(error.to_string()))?,
        );
        match form {
            Some(form) => request
                .headers(headers)
                .form(&form)
                .build()
                .map_err(Error::Http),
            None => request.headers(headers).build().map_err(Error::Http),
        }
    }

    fn cookie(&self, name: &str, url: &Url) -> TResult<Option<Cookie<'static>>> {
        let Some(header) = self.jar.cookies(url) else {
            return Ok(None);
        };
        header
            .to_str()
            .map_err(|error| Error::Protocol(error.to_string()))?
            .split(';')
            .map(|cookie| {
                Cookie::parse(cookie.trim().to_owned())
                    .map_err(|error| Error::Protocol(error.to_string()))
            })
            .collect::<TResult<Vec<_>>>()
            .map(|cookies| cookies.into_iter().find(|cookie| cookie.name() == name))
    }

    fn eapi_header_cookies(&self) -> TResult<BTreeMap<String, String>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| Error::Protocol(error.to_string()))?;
        let defaults = [
            ("osver", "undefined"),
            ("deviceId", self.device_id.as_str()),
            ("appver", "8.0.0"),
            ("versioncode", "140"),
            ("mobilename", self.device_id.as_str()),
            ("resolution", "1920x1080"),
            ("__csrf", ""),
            ("os", "android"),
            ("channel", "undefined"),
        ];
        let mut cookies = defaults
            .into_iter()
            .map(|(name, default)| {
                self.cookie(name, &self.config.base_url).map(|cookie| {
                    (
                        name.to_owned(),
                        cookie.map_or_else(|| default.to_owned(), |c| c.value().to_owned()),
                    )
                })
            })
            .collect::<TResult<BTreeMap<_, _>>>()?;
        cookies.insert(
            "buildver".to_owned(),
            self.cookie("buildver", &self.config.base_url)?.map_or_else(
                || now.as_secs().to_string(),
                |cookie| cookie.value().to_owned(),
            ),
        );
        cookies.insert(
            "requestId".to_owned(),
            request_id(now.as_millis(), rand::rng().random_range(0..1000)),
        );
        for name in ["MUSIC_U", "MUSIC_A"] {
            if let Some(cookie) = self.cookie(name, &self.config.base_url)? {
                cookies.insert(name.to_owned(), cookie.value().to_owned());
            }
        }
        Ok(cookies)
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn execute(&self, request: RequestPlan) -> TResult<Response> {
        self.execute_request(request).await
    }
}

#[derive(Debug)]
pub(crate) struct TransportConfig {
    cache: bool,
    cache_ttl: Duration,
    preserve_cookies: bool,
    cookie_path: PathBuf,
    device_id_path: Option<PathBuf>,
    user_agent: String,
    base_url: Url,
}

/// Opens `path` for a full rewrite and creates missing parent directories.
///
/// Session material is private to the user: new files start at `0600` on Unix
/// instead of inheriting the process umask.
fn open_private(path: &Path) -> std::io::Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

fn write_cookies(path: &Path, cookies: &str) -> TResult<()> {
    let mut file = open_private(path).map_err(|source| Error::CookiePersistence {
        operation: "opening the cookie file for writing",
        source,
    })?;
    file.write_all(cookies.as_bytes())
        .map_err(|source| Error::CookiePersistence {
            operation: "writing the cookie file",
            source,
        })
}

/// Returns the persisted device id, creating it on first use.
///
/// The id is an opaque 52-character hex token, matching the shape the mobile
/// client sends. Reusing one value per installation avoids presenting every
/// process as a brand-new device to the provider.
fn load_device_id(path: Option<&Path>) -> TResult<String> {
    let Some(path) = path else {
        return Ok(generate_device_id());
    };

    match fs::read_to_string(path) {
        Ok(raw) if !raw.trim().is_empty() => Ok(raw.trim().to_owned()),
        Ok(_) => {
            let id = generate_device_id();
            write_device_id(path, &id)?;
            Ok(id)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let id = generate_device_id();
            write_device_id(path, &id)?;
            Ok(id)
        }
        Err(source) => Err(Error::DeviceIdPersistence {
            operation: "reading the device id file",
            source,
        }),
    }
}

fn write_device_id(path: &Path, id: &str) -> TResult<()> {
    let mut file = open_private(path).map_err(|source| Error::DeviceIdPersistence {
        operation: "opening the device id file for writing",
        source,
    })?;
    file.write_all(id.as_bytes())
        .map_err(|source| Error::DeviceIdPersistence {
            operation: "writing the device id file",
            source,
        })
}

fn generate_device_id() -> String {
    (0..52)
        .map(|_| {
            const HEX: &[u8] = b"0123456789ABCDEF";
            HEX[rand::rng().random_range(0..16)] as char
        })
        .collect()
}

fn read_cookies(path: &Path) -> TResult<String> {
    let mut file = File::open(path).map_err(|source| Error::CookiePersistence {
        operation: "opening the cookie file",
        source,
    })?;
    let mut cookies = String::new();
    file.read_to_string(&mut cookies)
        .map_err(|source| Error::CookiePersistence {
            operation: "reading the cookie file",
            source,
        })?;
    Ok(cookies)
}

fn serialize_cookie_map(cookies: &BTreeMap<String, String>) -> String {
    cookies
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

fn adapt_url(url: &str, crypto: Crypto) -> String {
    match crypto {
        Crypto::Weapi => url.replacen("/api/", "/weapi/", 1),
        Crypto::Eapi => url.replacen("/api/", "/eapi/", 1),
        Crypto::Linuxapi => "https://music.163.com/api/linux/forward".to_owned(),
        Crypto::Plain => url.to_owned(),
    }
}

const BASE_URL: &str = "https://music.163.com";
const DEFAULT_USER_AGENT: &str = "ncmapi-rs/1.0";

#[cfg(test)]
mod tests {
    use std::{path::Path, time::Duration};

    use serde::Deserialize;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use super::{Crypto, HttpTransportBuilder, RequestPlanBuilder, adapt_url, write_cookies};
    use crate::Error;

    #[derive(Deserialize)]
    struct ContractResponse {
        code: u16,
    }

    #[test]
    fn device_id_is_created_once_and_reused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/device.id");
        let cookie_path = dir.path().join("cookies");

        let first = HttpTransportBuilder::new(&cookie_path)
            .unwrap()
            .device_id_path(&path)
            .build()
            .unwrap()
            .device_id;
        let second = HttpTransportBuilder::new(&cookie_path)
            .unwrap()
            .device_id_path(&path)
            .build()
            .unwrap()
            .device_id;

        assert_eq!(first.len(), 52);
        assert_eq!(first, second);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), first);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn cookie_file_is_created_with_private_permissions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/cookies");

        write_cookies(&path, "MUSIC_U=abc").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "MUSIC_U=abc");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn configured_user_agent_and_origin_reach_the_request() {
        let transport = HttpTransportBuilder::new(Path::new("/unused/ncmapi-cookie"))
            .unwrap()
            .persist_cookies(false)
            .user_agent("Mozilla/5.0 (Macintosh) Chrome/155.0.0.0")
            .build()
            .unwrap();
        let plan = RequestPlanBuilder::post("https://music.163.com/api/test")
            .payload(json!({}))
            .build();

        let request = transport.to_http_request(plan).unwrap();

        assert_eq!(
            request.headers()[reqwest::header::USER_AGENT],
            "Mozilla/5.0 (Macintosh) Chrome/155.0.0.0"
        );
        assert_eq!(
            request.headers()[reqwest::header::ORIGIN],
            "https://music.163.com"
        );
        assert_eq!(
            request.headers()[reqwest::header::REFERER],
            "https://music.163.com"
        );
    }

    #[test]
    fn invalid_user_agent_is_rejected() {
        let result = HttpTransportBuilder::new(Path::new("/unused/ncmapi-cookie"))
            .unwrap()
            .user_agent("bad header\nvalue")
            .build();

        assert!(matches!(
            result.err(),
            Some(Error::InvalidInput {
                field: "user_agent",
                ..
            })
        ));
    }

    #[test]
    fn encryption_selects_the_expected_protocol_path() {
        assert_eq!(
            adapt_url("https://example.test/api/search", Crypto::Weapi),
            "https://example.test/weapi/search"
        );
        assert_eq!(
            adapt_url("https://example.test/api/search", Crypto::Eapi),
            "https://example.test/eapi/search"
        );
    }

    #[tokio::test]
    async fn uncacheable_plans_always_reach_the_server() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/weapi/no-cache"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code": 200})))
            .expect(2)
            .mount(&server)
            .await;

        let transport = HttpTransportBuilder::new(Path::new("/unused/ncmapi-cookie"))
            .unwrap()
            .cache(true)
            .cache_ttl(Duration::from_secs(60))
            .persist_cookies(false)
            .build()
            .unwrap();

        for _ in 0..2 {
            let request = RequestPlanBuilder::post(format!("{}/api/no-cache", server.uri()))
                .payload(json!({"poll": true}))
                .uncacheable()
                .build();
            transport.execute_request(request).await.unwrap();
        }
        server.verify().await;
    }

    #[tokio::test]
    async fn cacheable_plans_hit_the_server_once() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/weapi/cached"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code": 200})))
            .expect(1)
            .mount(&server)
            .await;

        let transport = HttpTransportBuilder::new(Path::new("/unused/ncmapi-cookie"))
            .unwrap()
            .cache(true)
            .cache_ttl(Duration::from_secs(60))
            .persist_cookies(false)
            .build()
            .unwrap();

        for _ in 0..2 {
            let request = RequestPlanBuilder::post(format!("{}/api/cached", server.uri()))
                .payload(json!({"read": true}))
                .build();
            transport.execute_request(request).await.unwrap();
        }
        server.verify().await;
    }

    #[tokio::test]
    async fn encrypted_request_is_sent_to_a_local_http_contract() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/weapi/contract"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code": 200})))
            .mount(&server)
            .await;

        let transport = HttpTransportBuilder::new(Path::new("/unused/ncmapi-cookie"))
            .unwrap()
            .cache(false)
            .persist_cookies(false)
            .build()
            .unwrap();
        let request = RequestPlanBuilder::post(format!("{}/api/contract", server.uri()))
            .payload(json!({"ping": "pong"}))
            .build();

        let response = transport.execute_request(request).await.unwrap();
        assert_eq!(response.decode::<ContractResponse>().unwrap().code, 200);
    }
}
