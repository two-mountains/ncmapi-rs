mod api_request;
mod api_response;
mod route;
mod store;

use std::{
    borrow::Cow,
    convert::TryFrom,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use cookie::Cookie;
use rand::RngExt;
use regex::Regex;
use reqwest::{
    Client, Request, Response, Url,
    cookie::{CookieStore, Jar},
    header::{CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue, REFERER, SET_COOKIE, USER_AGENT},
};
use serde::Serialize;

pub use api_request::{ApiRequest, ApiRequestBuilder};
pub use api_response::ApiResponse;
pub(crate) use route::API_ROUTE;
use serde_json::{Value, json};
use store::Store;

use crate::{
    ApiErr,
    crypto::{Crypto, eapi, linuxapi, weapi},
};
use crate::{TResult, domain::request_id};

use self::api_request::Hm;

pub struct ApiClient {
    config: Config,
    client: Client,
    store: Store,
    // this is a compromise way to sync & retrive cookies, since access to cookie jar
    // is denied by self.client::Afc<ClientRef>.cookie_store;
    jar: Arc<dyn CookieStore>,
}

impl Default for ApiClient {
    fn default() -> Self {
        let cookie_path = "/var/tmp/ncmapi_client_cookies";
        Self::new(cookie_path)
    }
}

#[derive(Debug)]
pub struct ApiClientBuilder {
    config: Config,
}

impl ApiClientBuilder {
    pub fn new(cookie_path: &str) -> Self {
        ApiClientBuilder {
            config: Config {
                cache: true,
                cache_exp: Duration::from_secs(3 * 60),
                base_url: BASE_URL.parse::<Url>().unwrap(),
                preserve_cookies: true,
                cookie_path: String::from(cookie_path),
                log_request: false,
                log_response: false,
            },
        }
    }

    pub fn build(self) -> TResult<ApiClient> {
        let config = self.config;
        let cache_exp = config.cache_exp;
        let jar = Arc::new(Jar::default());

        // Synchronize cookies only when the caller opted into persistence.
        if config.preserve_cookies
            && let Ok(cs) = read_cookies(&config.cookie_path)
            && !cs.is_empty()
        {
            let ch = cs
                .split("; ")
                .map(|cookie| HeaderValue::from_str(cookie).unwrap())
                .collect::<Vec<_>>();
            jar.set_cookies(&mut ch.iter(), &config.base_url);
        }

        Ok(ApiClient {
            config,
            client: Client::builder()
                .cookie_store(false)
                .build()
                .map_err(|_| ApiErr::ReqwestErr)?,
            store: Store::new(cache_exp),
            jar,
        })
    }

    pub fn cache(mut self, enable: bool) -> Self {
        self.config.cache = enable;
        self
    }

    pub fn cache_exp(mut self, exp: Duration) -> Self {
        self.config.cache_exp = exp;
        self
    }

    pub fn preserve_cookies(mut self, enable: bool) -> Self {
        self.config.preserve_cookies = enable;
        self
    }

    #[allow(unused)]
    pub fn log_request(mut self, enable: bool) -> Self {
        self.config.log_request = enable;
        self
    }

    #[allow(unused)]
    pub fn log_response(mut self, enable: bool) -> Self {
        self.config.log_response = enable;
        self
    }
}

impl ApiClient {
    /// cookie_path: file path of cookie cache
    pub fn new(cookie_path: &str) -> ApiClient {
        ApiClientBuilder::new(cookie_path)
            .build()
            .expect("build apiclient fail")
    }

    pub async fn request(&self, req: ApiRequest) -> TResult<ApiResponse> {
        let id = req.id();

        if self.config.cache
            && let Some(response) = self.store.get(&id)
        {
            return Ok(response);
        }

        let request = self.to_http_request(req)?;
        if self.config.log_request {
            println!("{:#?}", request);
        }

        let resp = self
            .client
            .execute(request)
            .await
            .map_err(|_| ApiErr::ReqwestErr)?;
        self.on_response(id, resp).await
    }

    async fn on_response(&self, id: String, resp: Response) -> TResult<ApiResponse> {
        let mut cs = resp.headers().get_all(SET_COOKIE).iter().peekable();
        if cs.peek().is_some() {
            // sync cookie to jar
            self.jar.set_cookies(&mut cs, resp.url());
            // sync cookie to local
            let hv = self.jar.cookies(&self.config.base_url).unwrap();
            if self.config.preserve_cookies {
                write_cookies(&self.config.cookie_path, hv.to_str().unwrap()).unwrap_or_default();
            }
        }

        let body = resp.bytes().await.map_err(|_| ApiErr::ReqwestErr)?;
        let res = ApiResponse::new(body.to_vec());

        // cache response
        if self.config.cache {
            self.store.insert(id, res.clone());
        }

        Ok(res)
    }

    fn to_http_request(&self, req: ApiRequest) -> TResult<Request> {
        let (method, url, data, ua, cookies, crypto, api_url, real_ip) = req.pieces();
        // unwrap or else is lazily evaluated.
        let mut data = data.unwrap_or_else(|| json!({}));

        // basic header
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(fake_ua(ua)));
        if method == api_request::Method::Post {
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            );
        }
        if url.contains("music.163.com") {
            headers.insert(REFERER, HeaderValue::from_static(BASE_URL));
        }
        if let Some(real_ip) = real_ip {
            headers.insert("X-Real-IP", HeaderValue::try_from(real_ip).unwrap());
        }

        // COOKIE header might be overrided by the cookie_store according to
        // reqwest/async_impl/client.rs line: 1232 of version: e6a1a09f0904e06de4ff1317278798c4ed28af66
        //
        // The leading dot means that the cookie is valid for subdomains as well;
        // nevertheless recent HTTP specifications (RFC 6265) changed this rule so modern browsers
        // should notcare about the leading dot. The dot may be needed by old browser implementing the deprecated RFC 2109.
        //
        // so what's sense of adding cookie to the request header which will will be overrided.
        // Another mechanism of preserving cookies might be required. // TODO --> Solved by disable cookies_store & build a new one
        // by simulating.
        match crypto {
            // option cookies + jar cookies
            Crypto::Weapi => {
                let mut cs = String::new();
                // jar cookies
                let jc = self.jar.cookies(self.base_url());
                if let Some(hv) = jc {
                    cs.push_str(hv.to_str().unwrap());
                }

                // option cookies
                if let Some(oc) = &cookies {
                    let oc = oc
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("; ");
                    cs.push_str("; ");
                    cs.push_str(&oc);
                }
                headers.insert(COOKIE, HeaderValue::try_from(cs).unwrap());
            }
            Crypto::Eapi => {
                let mut cs = self.eapi_header_cookies();
                if let Some(ref cookies) = cookies {
                    for (k, v) in cookies {
                        cs.insert(k.to_owned(), v.to_owned());
                    }
                }

                let cs = cs
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("; ");
                headers.insert(COOKIE, HeaderValue::try_from(cs).unwrap());
            }
            Crypto::Linuxapi => {
                let cs = self
                    .jar
                    .cookies(self.base_url())
                    .unwrap_or(HeaderValue::from_static(""));
                headers.insert(COOKIE, HeaderValue::try_from(cs.to_str().unwrap()).unwrap());
            }
        }

        // payload
        // form data
        match crypto {
            Crypto::Weapi => {
                let key = "csrf_token";
                let mut val = String::new();
                if let Some(cookie) = self.cookie("__csrf", &self.config.base_url) {
                    val = cookie.value().to_owned();
                }
                data.as_object_mut()
                    .unwrap()
                    .insert(key.to_owned(), Value::String(val));
            }
            Crypto::Eapi => {
                let mut cs = self.eapi_header_cookies();
                if let Some(ref cookies) = cookies {
                    for (k, v) in cookies {
                        cs.insert(k.to_owned(), v.to_owned());
                    }
                }
                data.as_object_mut()
                    .unwrap()
                    .insert("header".to_owned(), json!(cs));
            }
            Crypto::Linuxapi => {}
        }

        let form_data = {
            match crypto {
                Crypto::Weapi => {
                    let data = data.to_string();
                    weapi(data.as_bytes()).into_vec()
                }
                Crypto::Eapi => {
                    let data = data.to_string();
                    let api_url = api_url.unwrap();
                    eapi(api_url.as_bytes(), data.as_bytes()).into_vec()
                }
                Crypto::Linuxapi => {
                    let data = json!({
                        "method": map_method(method).to_string(),
                        "url": adapt_url(&url, crypto),
                        "params": &data,
                    })
                    .to_string();
                    linuxapi(data.as_bytes()).into_vec()
                }
            }
        };

        // request builder
        let rb = self
            .client
            .request(
                map_method(method),
                adapt_url(&url, crypto)
                    .parse::<Url>()
                    .map_err(|_| ApiErr::ParseUrlErr)?,
            )
            .headers(headers)
            .form(&form_data);

        rb.build().map_err(|_| ApiErr::ReqwestErr)
    }

    fn cookies(&self, url: &Url) -> Vec<Cookie<'static>> {
        let mut cs = Vec::new();
        if let Some(cookies) = self.jar.cookies(url)
            && !cookies.is_empty()
        {
            cookies
                .to_str()
                .unwrap()
                .split(';')
                .map(|s| Cookie::parse(s.to_owned()).unwrap())
                .for_each(|c| cs.push(c));
        }
        cs
    }

    pub fn base_url(&self) -> &Url {
        &self.config.base_url
    }

    pub fn cookie(&self, name: &str, url: &Url) -> Option<Cookie<'static>> {
        self.cookies(url)
            .into_iter()
            .find(|cookie| cookie.name() == name)
    }

    fn cookie_netease_eapi(&self, name: &str) -> Option<String> {
        if let Some(cookie) = self.cookie(name, &self.config.base_url) {
            return Some(cookie.value().to_owned());
        }
        None
    }

    fn eapi_header_cookies(&self) -> Hm {
        let mut hm = Hm::new();
        let mut rng = rand::rng();

        hm.insert(
            "osver".to_owned(),
            self.cookie_netease_eapi("osver")
                .unwrap_or_else(|| "undefined".to_owned()),
        );
        hm.insert(
            "deviceId".to_owned(),
            self.cookie_netease_eapi("deviceId")
                .unwrap_or_else(|| "undefined".to_owned()),
        );
        hm.insert(
            "appver".to_owned(),
            self.cookie_netease_eapi("appver")
                .unwrap_or_else(|| "8.0.0".to_owned()),
        );
        hm.insert(
            "versioncode".to_owned(),
            self.cookie_netease_eapi("versioncode")
                .unwrap_or_else(|| "140".to_owned()),
        );
        hm.insert(
            "mobilename".to_owned(),
            self.cookie_netease_eapi("mobilename")
                .unwrap_or_else(|| "undefined".to_owned()),
        );
        hm.insert(
            "buildver".to_owned(),
            self.cookie_netease_eapi("buildver").unwrap_or_else(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string()
            }),
        );
        hm.insert(
            "resolution".to_owned(),
            self.cookie_netease_eapi("resolution")
                .unwrap_or_else(|| "1920x1080".to_owned()),
        );
        hm.insert(
            "__csrf".to_owned(),
            self.cookie_netease_eapi("__csrf").unwrap_or_default(),
        );
        hm.insert(
            "os".to_owned(),
            self.cookie_netease_eapi("os")
                .unwrap_or_else(|| "android".to_owned()),
        );
        hm.insert(
            "channel".to_owned(),
            self.cookie_netease_eapi("channel")
                .unwrap_or_else(|| "undefined".to_owned()),
        );
        hm.insert(
            "requestId".to_owned(),
            request_id(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
                rng.random_range(0..1000),
            ),
        );

        if let Some(val) = self.cookie_netease_eapi("MUSIC_U") {
            hm.insert("MUSIC_U".to_owned(), val);
        }
        if let Some(val) = self.cookie_netease_eapi("MUSIC_A") {
            hm.insert("MUSIC_A".to_owned(), val);
        }

        hm
    }
}

#[derive(Debug)]
pub(crate) struct Config {
    cache: bool,
    cache_exp: Duration,
    preserve_cookies: bool,
    cookie_path: String,
    base_url: Url,

    log_request: bool,
    log_response: bool,
}

#[derive(Serialize, Debug, Clone, Copy)]
#[allow(unused)]
pub enum UA {
    Chrome,
    Edge,
    Firefox,
    Safari,
    Android,
    IPhone,
    Linux,
}

fn write_cookies(path: &str, cs: &str) -> TResult<()> {
    if !Path::new(path).exists() {
        File::create(path).map_err(|_| ApiErr::WriteCookieErr)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|_| ApiErr::WriteCookieErr)?;

    file.write_all(cs.as_bytes())
        .map_err(|_| ApiErr::WriteCookieErr)?;
    Ok(())
}

fn read_cookies(path: &str) -> TResult<String> {
    let mut file = File::open(path).map_err(|_| ApiErr::WriteCookieErr)?;
    let mut cs = String::new();
    file.read_to_string(&mut cs)
        .map_err(|_| ApiErr::WriteCookieErr)?;

    Ok(cs)
}

#[allow(unused)]
fn serialize_cookies(cookies: &[Cookie]) -> String {
    cookies
        .iter()
        .map(|cookie| cookie.to_string())
        .collect::<Vec<String>>()
        .join("; ")
}

fn fake_ua(ua: UA) -> &'static str {
    match ua {
        UA::Chrome => UA_CHROME,
        UA::Firefox => UA_FIREFOX,
        UA::Safari => UA_SAFARI,
        UA::Android => UA_ANDROID,
        UA::IPhone => UA_IPHONE,
        UA::Edge => UA_CHROME,
        UA::Linux => UA_LINUX,
    }
}

fn adapt_url(url: &str, crypto: Crypto) -> String {
    let re = Regex::new(r"\w*api").unwrap();
    let u = match crypto {
        Crypto::Weapi => re.replace_all(url, "weapi"),
        Crypto::Eapi => re.replace_all(url, "eapi"),
        Crypto::Linuxapi => Cow::from("https://music.163.com/api/linux/forward"),
    };
    u.to_string()
}

// The reason why directly use Method in reqwest is that i can't find a simple way to
// get a unique id for a api_request, and serialize to json is a compromize way and Method in reqwest
// is not serializable.
fn map_method(method: api_request::Method) -> reqwest::Method {
    match method {
        api_request::Method::Get => reqwest::Method::GET,
        api_request::Method::Head => reqwest::Method::HEAD,
        api_request::Method::Post => reqwest::Method::POST,
        api_request::Method::Options => reqwest::Method::OPTIONS,
        api_request::Method::Connect => reqwest::Method::CONNECT,
        api_request::Method::Trace => reqwest::Method::TRACE,
        api_request::Method::Delete => reqwest::Method::DELETE,
        api_request::Method::Put => reqwest::Method::PUT,
        api_request::Method::Patch => reqwest::Method::PATCH,
    }
}

const BASE_URL: &str = "https://music.163.com";

const UA_CHROME: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/42.0.2311.135 Safari/537.36 Edge/13.10586";
const UA_FIREFOX: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:46.0) Gecko/20100101 Firefox/46.0";
const UA_SAFARI: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.103 Safari/537.36";
const UA_ANDROID: &str = "Mozilla/5.0 (Linux; Android 9; PCT-AL10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/70.0.3538.64 HuaweiBrowser/10.0.3.311 Mobile Safari/537.36";
const UA_IPHONE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 13_5_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.1.1 Mobile/15E148 Safari/604.1";
const UA_LINUX: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36";

#[cfg(test)]
mod contract_tests {
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use super::{ApiClientBuilder, ApiRequestBuilder};

    #[tokio::test]
    async fn encrypted_request_is_sent_to_a_local_http_contract() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/weapi/contract"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code": 200})))
            .mount(&server)
            .await;

        let client = ApiClientBuilder::new("/unused/ncmapi-cookie")
            .cache(false)
            .preserve_cookies(false)
            .build()
            .expect("a local HTTP client can be built");
        let request = ApiRequestBuilder::post(&format!("{}/api/contract", server.uri()))
            .set_data(json!({"ping": "pong"}))
            .build();

        let response = client
            .request(request)
            .await
            .expect("local contract response");
        assert_eq!(response.deserialize_to_implict().code, 200);
    }
}

#[cfg(all(test, feature = "live-tests"))]
mod tests {
    use crate::client::ApiClientBuilder;
    use crate::client::route::API_ROUTE;

    use super::*;
    use serde_json::json;

    const COOKIE_PATH: &str = "/var/tmp/ncmapi_client_cookies";

    type Rb = api_request::ApiRequestBuilder;

    fn create_search_req() -> ApiRequest {
        Rb::post(API_ROUTE["cloudsearch"])
            .set_data(json!({
                "s": "mota",
                "type": 1,
            }))
            .insert("offset", json!(0))
            .merge(json!({"limit": 1}))
            .build()
    }

    #[test]
    fn test_client() {
        let cb = ApiClientBuilder::new(COOKIE_PATH)
            .cache(true)
            .preserve_cookies(true)
            .log_request(true);

        let res = cb.build();

        assert!(res.is_ok());
    }

    #[test]
    fn test_to_http_request() {
        let r = Rb::post(API_ROUTE["cloudsearch"])
            .set_data(json!({
                "s": "mota",
                "type": 1,
            }))
            .insert("offset", json!(0))
            .merge(json!({"limit": 3}))
            .set_api_url("/api/url")
            .set_real_ip("real_ip")
            .set_ua(UA::IPhone)
            .set_cookies(Hm::new())
            .add_cookie("sid", "f1h82fg191fh9")
            .build();

        let c = ApiClientBuilder::new(COOKIE_PATH).build().unwrap();
        let http_req = c.to_http_request(r);

        assert!(http_req.is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_request() {
        let c = ApiClientBuilder::new(COOKIE_PATH)
            .log_request(true)
            .build()
            .unwrap();
        let r = create_search_req();

        let resp = c.request(r).await;
        assert!(resp.is_ok());
        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cache() {
        let c = ApiClientBuilder::new(COOKIE_PATH).build().unwrap();

        let r = create_search_req();
        let resp = c.request(r).await;
        assert!(resp.is_ok());
        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
        std::thread::sleep(std::time::Duration::from_secs(10));

        let r = create_search_req();
        let resp = c.request(r).await;
        assert!(resp.is_ok());
        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
        std::thread::sleep(std::time::Duration::from_secs(10));

        let r = create_search_req();
        let resp = c.request(r).await;
        assert!(resp.is_ok());
        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    #[test]
    fn test_read_cookies() {
        let res = read_cookies(COOKIE_PATH);
        assert!(res.is_ok());
    }

    #[test]
    fn test_write_cookies() {
        let res = write_cookies(COOKIE_PATH, "name=alex; age=19");
        assert!(res.is_ok())
    }

    #[test]
    fn test_eapi_headers() {
        let c = ApiClientBuilder::new(COOKIE_PATH).build().unwrap();

        let c = c.eapi_header_cookies();
        println!("{}", c.get("requestId").unwrap());
    }
}
