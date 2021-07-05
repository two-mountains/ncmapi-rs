use std::{time::Duration, usize};

use openssl::hash::{hash, MessageDigest};
use rand::{Rng, RngCore};
use serde_json::{json, Value};

use crate::{
    client::{ApiClient, ApiClientBuilder, ApiRequestBuilder, ApiResponse, API_ROUTE},
    TResult,
};

pub struct NcmApi {
    client: ApiClient,
}

impl Default for NcmApi {
    fn default() -> Self {
        Self {
            client: ApiClient::default(),
        }
    }
}

impl NcmApi {
    pub fn new(
        enable_cache: bool,
        cache_exp: Duration,
        cache_clean_interval: Duration,
        preserve_cookies: bool,
        cookie_path: &str,
    ) -> Self {
        Self {
            client: ApiClientBuilder::new()
                .cache(enable_cache)
                .cache_exp(cache_exp)
                .cache_clean_interval(cache_clean_interval)
                .preserve_cookies(preserve_cookies)
                .cookie_path(cookie_path)
                .build()
                .unwrap(),
        }
    }
}

// apis
impl NcmApi {
    // 1: 单曲, 10: 专辑, 100: 歌手, 1000: 歌单, 1002: 用户, 1004: MV, 1006: 歌词, 1009: 电台, 1014: 视频
    async fn _search(&self, key: &str, route: &str, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE[route])
            .set_data(limit_offset(30, 0))
            .merge(json!({
                "s": key,
                "type": 1,
            }))
            .merge(opt.unwrap_or_default())
            .build();

        self.client.request(r).await
    }

    pub async fn search(&self, key: &str, opt: Option<Value>) -> TResult<ApiResponse> {
        self._search(key, "search", opt).await
    }

    pub async fn cloud_search(&self, key: &str, opt: Option<Value>) -> TResult<ApiResponse> {
        self._search(key, "cloudsearch", opt).await
    }

    pub async fn album_sub(&self, id: usize, op: u8) -> TResult<ApiResponse> {
        let op = if op == 1 { "sub" } else { "unsub" };
        let u = replace_all_route_params(API_ROUTE["album_sub"], op);
        let r = ApiRequestBuilder::post(&u)
            .set_data(json!({
                "id": id,
            }))
            .build();

        self.client.request(r).await
    }

    pub async fn album_sublist(&self, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["album_sublist"])
            .set_data(limit_offset(25, 0))
            .insert("total", Value::Bool(true))
            .merge(opt.unwrap_or_default())
            .build();

        self.client.request(r).await
    }

    pub async fn album(&self, id: usize) -> TResult<ApiResponse> {
        let u = replace_all_route_params(API_ROUTE["album"], &id.to_string());
        let r = ApiRequestBuilder::post(&u).build();

        self.client.request(r).await
    }

    pub async fn artist_songs(&self, id: usize, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["artist_songs"])
            .set_data(json!({
                "id": id,
                "private_cloud": true,
                "work_type":     1,
                "order":         "hot",
                "offset":        0,
                "limit":         100,
            }))
            .merge(opt.unwrap_or_default())
            .add_cookie("os", "pc")
            .build();

        self.client.request(r).await
    }

    pub async fn artist_sub(&self, id: usize, sub: u8) -> TResult<ApiResponse> {
        let mut opt = "sub";
        if sub != 1 {
            opt = "unsub";
        }

        let u = replace_all_route_params(API_ROUTE["artist_sub"], opt);
        let r = ApiRequestBuilder::post(&u)
            .set_data(json!({
                "artistId": id,
                "artistIds": [id]
            }))
            .build();

        self.client.request(r).await
    }

    pub async fn artist_sublist(&self, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["artist_sublist"])
            .set_data(limit_offset(25, 0))
            .merge(opt.unwrap_or_default())
            .insert("total", Value::Bool(true))
            .build();

        self.client.request(r).await
    }

    pub async fn artist_top_song(&self, id: usize) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["artist_top_song"])
            .set_data(json!({ "id": id }))
            .build();

        self.client.request(r).await
    }

    pub async fn check_music(&self, id: usize, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["check_music"])
            .set_data(json!({"br": 999000}))
            .merge(opt.unwrap_or_default())
            .merge(json!({ "ids": [id] }))
            .build();

        self.client.request(r).await
    }

    pub async fn comment_hot(
        &self,
        id: usize,
        resouce_type: ResourceType,
        opt: Option<Value>,
    ) -> TResult<ApiResponse> {
        let u = replace_all_route_params(API_ROUTE["comment_hot"], "");
        let u = format!("{}{}{}", u, map_resource_code(resouce_type), id);

        let r = ApiRequestBuilder::post(&u)
            .add_cookie("os", "pc")
            .set_data(limit_offset(20, 0))
            .merge(opt.unwrap_or_default())
            .merge(json!({
                "beforeTime": 0,
                "rid": id
            }))
            .build();

        self.client.request(r).await
    }

    pub async fn comment_new(
        &self,
        id: usize,
        resource_type: ResourceType,
        page_size: usize,
        page_no: usize,
        sort_type: usize,
        cursor: usize,
        show_inner: bool,
    ) -> TResult<ApiResponse> {
        let mut cursor = cursor;
        if sort_type != 3 {
            cursor = (page_no - 1) * page_size;
        }

        let r = ApiRequestBuilder::post(API_ROUTE["comment_new"])
            .set_crypto(crate::crypto::Crypto::Eapi)
            .add_cookie("os", "pc")
            .set_api_url("/api/v2/resource/comments")
            .set_data(json!({
                "pageSize":  page_size,
                "pageNo":    page_no,
                "sortType":  sort_type,
                "cursor":    cursor,
                "showInner": show_inner,
            }))
            .insert(
                "threadId",
                Value::String(format!("{}{}", map_resource_code(resource_type), id)),
            )
            .build();

        self.client.request(r).await
    }

    pub async fn comment_create(
        &self,
        rid: usize,
        rt: ResourceType,
        cmt: &str,
    ) -> TResult<ApiResponse> {
        let thread_id = format!("{}{}", map_resource_code(rt), rid);

        let u = replace_all_route_params(API_ROUTE["comment"], "add");
        let r = ApiRequestBuilder::post(&u)
            .add_cookie("os", "pc")
            .set_data(json!({"threadId": thread_id, "content": cmt}))
            .build();

        self.client.request(r).await
    }

    pub async fn comment_re(
        &self,
        rid: usize,
        rt: ResourceType,
        re_id: usize,
        cmt: &str,
    ) -> TResult<ApiResponse> {
        let thread_id = format!("{}{}", map_resource_code(rt), rid);

        let u = replace_all_route_params(API_ROUTE["comment"], "reply");
        let r = ApiRequestBuilder::post(&u)
            .add_cookie("os", "pc")
            .set_data(json!({"threadId": thread_id, "content": cmt, "commentId": re_id}))
            .build();

        self.client.request(r).await
    }

    pub async fn comment_del(
        &self,
        rid: usize,
        rt: ResourceType,
        cmt_id: usize,
    ) -> TResult<ApiResponse> {
        let thread_id = format!("{}{}", map_resource_code(rt), rid);

        let u = replace_all_route_params(API_ROUTE["comment"], "delete");
        let r = ApiRequestBuilder::post(&u)
            .add_cookie("os", "pc")
            .set_data(json!({"threadId": thread_id, "commentId": cmt_id}))
            .build();

        self.client.request(r).await
    }

    pub async fn daily_signin(&self, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["daily_signin"])
            .set_data(json!({"type": 0}))
            .merge(opt.unwrap_or_default())
            .build();

        self.client.request(r).await
    }

    pub async fn fm_trash(&self, id: usize) -> TResult<ApiResponse> {
        let mut rng = rand::thread_rng();
        let u = format!(
            "https://music.163.com/weapi/radio/trash/add?alg=RT&songId={}&time={}",
            id,
            rng.gen_range(10..20)
        );
        let r = ApiRequestBuilder::post(&u)
            .set_data(json!({ "songId": id }))
            .build();

        self.client.request(r).await
    }

    pub async fn like(&self, id: usize, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["like"])
            .add_cookie("os", "pc")
            .add_cookie("appver", "2.7.1.198277")
            .set_real_ip("118.88.88.88")
            .set_data(json!({"alg": "itembased", "time": 3, "like": true, "trackId": id}))
            .merge(opt.unwrap_or_default())
            .build();

        self.client.request(r).await
    }

    // fav_list
    pub async fn likelist(&self, uid: usize) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["likelist"])
            .set_data(json!({ "uid": uid }))
            .build();

        self.client.request(r).await
    }

    pub async fn login_phone(&self, phone: &str, password: &str) -> TResult<ApiResponse> {
        let password = md5_hex(password.as_bytes());
        let r = ApiRequestBuilder::post(API_ROUTE["login_cellphone"])
            .add_cookie("os", "pc")
            .set_data(json!({
                "countrycode":   "86",
                "rememberLogin": true,
                "phone": phone,
                "password": password,
            }))
            .build();

        self.client.request(r).await
    }

    pub async fn login_refresh(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["login_refresh"]).build();

        self.client.request(r).await
    }

    pub async fn login_status(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["login_status"]).build();

        self.client.request(r).await
    }

    pub async fn logout(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["logout"]).build();

        self.client.request(r).await
    }

    pub async fn lyric(&self, id: usize) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["lyric"])
            .add_cookie("os", "pc")
            .set_data(json!({
                "id": id,
                "lv": -1,
                "kv": -1,
                "tv": -1,
            }))
            .build();

        self.client.request(r).await
    }

    pub async fn personal_fm(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["personal_fm"]).build();

        self.client.request(r).await
    }

    pub async fn playlist_detail(&self, id: usize, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["playlist_detail"])
            .set_data(json!({"n": 100000, "s": 8, "id": id}))
            .merge(opt.unwrap_or_default())
            .build();

        self.client.request(r).await
    }

    pub async fn playlist_tracks(
        &self,
        pid: usize,
        op: u8,
        tracks: Vec<usize>,
    ) -> TResult<ApiResponse> {
        let op = if op == 1 { "add" } else { "del" };
        let r = ApiRequestBuilder::post(API_ROUTE["playlist_tracks"])
            .add_cookie("os", "pc")
            .set_data(json!({"op": op, "pid": pid, "trackIds": tracks, "imme": true}))
            .build();

        self.client.request(r).await
    }

    pub async fn playlist_update(
        &self,
        pid: usize,
        name: &str,
        desc: &str,
        tags: Vec<&str>,
    ) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["playlist_update"])
            .add_cookie("os", "pc")
            .set_data(json!({
                "/api/playlist/update/name": {"id": pid, "name": name},
                "/api/playlist/desc/update": {"id": pid, "desc": desc},
                "/api/playlist/tags/update": {"id": pid, "tags": tags.join(";")},
            }))
            .build();

        self.client.request(r).await
    }

    pub async fn recommend_resource(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["recommend_resource"]).build();

        self.client.request(r).await
    }

    pub async fn recommend_songs(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["recommend_songs"])
            .add_cookie("os", "ios")
            .build();

        self.client.request(r).await
    }

    pub async fn scrobble(&self, id: usize, source_id: usize) -> TResult<ApiResponse> {
        let mut rng = rand::thread_rng();
        let r = ApiRequestBuilder::post(API_ROUTE["scrobble"])
            .set_data(json!({
                "logs": [{
                        "action": "play",
                        "json": {
                            "download": 0,
                            "end":      "playend",
                            "id":       id,
                            "sourceId": source_id,
                            "time":     rng.gen_range(20..30),
                            "type":     "song",
                            "wifi":     0,
                        }
                }]
            }))
            .build();

        self.client.request(r).await
    }

    pub async fn search_default(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["search_default"])
            .set_crypto(crate::crypto::Crypto::Eapi)
            .set_api_url("/api/search/defaultkeyword/get")
            .build();

        self.client.request(r).await
    }

    pub async fn search_hot_detail(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["search_hot_detail"]).build();

        self.client.request(r).await
    }

    pub async fn search_hot(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["search_hot"])
            .set_data(json!({"type": 1111}))
            .set_ua(crate::client::UA::IPhone)
            .build();

        self.client.request(r).await
    }

    pub async fn search_suggest(&self, keyword: &str, opt: Option<Value>) -> TResult<ApiResponse> {
        let mut device = "web";
        if let Some(val) = opt {
            if val["type"] == "mobile" {
                device = "mobile"
            }
        }

        let u = format!("{}{}", API_ROUTE["search_suggest"], device);
        let r = ApiRequestBuilder::post(&u)
            .set_data(json!({ "s": keyword }))
            .build();

        self.client.request(r).await
    }

    pub async fn simi_artist(&self, artist_id: usize) -> TResult<ApiResponse> {
        let mut r = ApiRequestBuilder::post(API_ROUTE["simi_artist"])
            .set_data(json!({ "artistid": artist_id }));
        if self
            .client
            .cookie("MUSIC_U", self.client.base_url())
            .is_none()
        {
            r = r.add_cookie("MUSIC_A", ANONYMOUS_TOKEN);
        }

        self.client.request(r.build()).await
    }

    pub async fn simi_playlist(&self, id: usize, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["simi_playlist"])
            .set_data(limit_offset(50, 0))
            .merge(opt.unwrap_or_default())
            .insert("songid", json!(id))
            .build();

        self.client.request(r).await
    }

    pub async fn simi_song(&self, id: usize, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["simi_song"])
            .set_data(limit_offset(50, 0))
            .merge(opt.unwrap_or_default())
            .insert("songid", json!(id))
            .build();

        self.client.request(r).await
    }

    pub async fn song_detail(&self, ids: &Vec<usize>) -> TResult<ApiResponse> {
        let list = ids
            .iter()
            .map(|id| json!({ "id": id }).to_string())
            .collect::<Vec<_>>();
        let r = ApiRequestBuilder::post(API_ROUTE["song_detail"])
            .set_data(json!({ "c": list }))
            .build();

        self.client.request(r).await
    }

    pub async fn song_url(&self, ids: &Vec<usize>) -> TResult<ApiResponse> {
        let mut rb = ApiRequestBuilder::post(API_ROUTE["song_url"])
            .set_crypto(crate::crypto::Crypto::Eapi)
            .add_cookie("os", "pc")
            .set_api_url("/api/song/enhance/player/url")
            .set_data(json!({"ids": ids, "br": 999000}));

        if self
            .client
            .cookie("MUSIC_U", self.client.base_url())
            .is_none()
        {
            let mut rng = rand::thread_rng();
            let mut token = [0u8; 16];
            rng.fill_bytes(&mut token);
            rb = rb.add_cookie("_ntes_nuid", &hex::encode(token));
        }

        self.client.request(rb.build()).await
    }

    pub async fn user_account(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["user_account"]).build();
        self.client.request(r).await
    }

    pub async fn user_cloud_detail(&self, ids: &Vec<usize>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["user_cloud_detail"])
            .set_data(json!({ "songIds": ids }))
            .build();
        self.client.request(r).await
    }

    pub async fn user_cloud(&self, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["user_cloud"])
            .set_data(limit_offset(30, 0))
            .merge(opt.unwrap_or_default())
            .build();
        self.client.request(r).await
    }

    pub async fn user_comment_history(
        &self,
        uid: usize,
        opt: Option<Value>,
    ) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["user_comment_history"])
            .set_data(json!({
                "compose_reminder":    true,
                "compose_hot_comment": true,
                "limit":               10,
                "time":                0,
                "user_id":             uid,
            }))
            .merge(opt.unwrap_or_default())
            .build();
        self.client.request(r).await
    }

    pub async fn user_detail(&self, uid: usize) -> TResult<ApiResponse> {
        let u = replace_all_route_params(API_ROUTE["user_detail"], &uid.to_string());
        let r = ApiRequestBuilder::post(&u).build();
        self.client.request(r).await
    }

    pub async fn user_dj(&self, uid: usize, opt: Option<Value>) -> TResult<ApiResponse> {
        let u = replace_all_route_params(API_ROUTE["user_dj"], &uid.to_string());
        let r = ApiRequestBuilder::post(&u)
            .set_data(limit_offset(30, 0))
            .merge(opt.unwrap_or_default())
            .build();
        self.client.request(r).await
    }

    pub async fn user_level(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["user_level"]).build();
        self.client.request(r).await
    }

    pub async fn user_playlist(&self, uid: usize, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["user_playlist"])
            .set_data(limit_offset(30, 0))
            .merge(opt.unwrap_or_default())
            .merge(json!({"includeVideo": true, "uid": uid}))
            .build();
        self.client.request(r).await
    }

    pub async fn user_record(&self, uid: usize, opt: Option<Value>) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["user_record"])
            .set_data(json!({"type": 1, "uid": uid}))
            .merge(opt.unwrap_or_default())
            .build();
        self.client.request(r).await
    }

    pub async fn user_subcount(&self) -> TResult<ApiResponse> {
        let r = ApiRequestBuilder::post(API_ROUTE["user_subcount"]).build();
        self.client.request(r).await
    }
}

fn replace_all_route_params(u: &str, rep: &str) -> String {
    let re = regex::Regex::new(r"\$\{.*\}").unwrap();
    re.replace_all(u, rep).to_string()
}

fn limit_offset(limit: usize, offset: usize) -> Value {
    json!({
        "limit": limit,
        "offset": offset
    })
}

// id : 资源 id
// type: 数字 , 资源类型 , 对应歌曲 , mv, 专辑 , 歌单 , 电台, 视频对应以下类型
// 0: 歌曲
// 1: mv
// 2: 歌单
// 3: 专辑
// 4: 电台
// 5: 视频
#[derive(Copy, Clone)]
pub enum ResourceType {
    Song = 0,
    MV = 1,
    Collection = 2,
    Album = 3,
    Podcast = 4,
    Video = 5,
    Moment = 6,
}

// pub enum ResourceTypeCode {
//     R_SO_4_,
//     R_MV_5_,
//     A_PL_0_,
//     R_AL_3_,
//     A_DJ_1_,
//     R_VI_62_,
//     A_EV_2_,
// }

fn map_resource_code(t: ResourceType) -> String {
    match t {
        ResourceType::Song => String::from("R_SO_4_"),
        ResourceType::MV => String::from("R_MV_5_"),
        ResourceType::Collection => String::from("A_PL_0_"),
        ResourceType::Album => String::from("R_AL_3_"),
        ResourceType::Podcast => String::from("A_DJ_1_"),
        ResourceType::Video => String::from("R_VI_62_"),
        ResourceType::Moment => String::from("A_EV_2_"),
    }
}

fn md5_hex(pt: &[u8]) -> String {
    hex::encode(hash(MessageDigest::md5(), pt).unwrap())
}

const ANONYMOUS_TOKEN: &str = "8aae43f148f990410b9a2af38324af24e87ab9227c9265627ddd10145db744295fcd8701dc45b1ab8985e142f491516295dd965bae848761274a577a62b0fdc54a50284d1e434dcc04ca6d1a52333c9a";

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use tokio::fs;

    use crate::NcmApi;

    const ALBUM_ID: usize = 34808483;
    const SONG_ID: usize = 32977061;
    const COLLECTION_ID: usize = 2484967117;
    const ARTIST_ID: usize = 5771;
    const USER_ID: usize = 49668844;

    #[derive(Deserialize)]
    struct Auth {
        phone: String,
        password: String,
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_search() {
        let api = NcmApi::default();
        let resp = api.search("mota", None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cloud_search() {
        let api = NcmApi::default();
        let resp = api.cloud_search("mota", None).await;
        assert!(resp.is_ok());
        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_album_sub() {
        let api = NcmApi::default();
        let resp = api.album_sub(ALBUM_ID, 1).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_album_sublist() {
        let api = NcmApi::default();
        let resp = api.album_sublist(None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_album() {
        let api = NcmApi::default();
        let resp = api.album(ALBUM_ID).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artist_songs() {
        let api = NcmApi::default();
        let resp = api.artist_songs(ARTIST_ID, None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artist_sub() {
        let api = NcmApi::default();
        let resp = api.artist_sub(ARTIST_ID, 1).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artist_sublist() {
        let api = NcmApi::default();
        let resp = api.artist_sublist(None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artist_top_song() {
        let api = NcmApi::default();
        let resp = api.artist_top_song(ARTIST_ID).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_check_music() {
        let api = NcmApi::default();
        let resp = api.check_music(SONG_ID, None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_comment_hot() {
        let api = NcmApi::default();
        let resp = api
            .comment_hot(SONG_ID, crate::api::ResourceType::Song, None)
            .await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_comment_new() {
        let api = NcmApi::default();
        let resp = api
            .comment_new(SONG_ID, crate::api::ResourceType::Song, 1, 1, 1, 0, true)
            .await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_comment_create() {
        let api = NcmApi::default();
        let resp = api
            .comment_create(SONG_ID, crate::api::ResourceType::Song, "喜欢")
            .await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_comment_re() {}

    #[tokio::test(flavor = "multi_thread")]
    async fn test_comment_del() {}

    #[tokio::test(flavor = "multi_thread")]
    async fn test_daily_signin() {
        let api = NcmApi::default();
        let resp = api.daily_signin(None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_fm_trash() {
        let api = NcmApi::default();
        let resp = api.fm_trash(347230).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_like() {
        let api = NcmApi::default();
        let resp = api.like(SONG_ID, None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_likelist() {
        let api = NcmApi::default();
        let resp = api.likelist(USER_ID).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_login_phone() {
        let f = fs::read_to_string("test_data/auth.json")
            .await
            .expect("no auth file");
        let auth: Auth = serde_json::from_str(&f).unwrap();

        let api = NcmApi::default();
        let resp = api.login_phone(&auth.phone, &auth.password).await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_login_refresh() {
        let api = NcmApi::default();
        let resp = api.login_refresh().await;
        assert!(resp.is_ok());

        let res = resp.unwrap().deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_login_status() {
        let api = NcmApi::default();
        let resp = api.login_status().await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_logout() {
        let api = NcmApi::default();
        let resp = api.logout().await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lyric() {
        let api = NcmApi::default();
        let resp = api.lyric(SONG_ID).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_personal_fm() {
        let api = NcmApi::default();
        let resp = api.personal_fm().await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_playlist_detail() {
        let api = NcmApi::default();
        let resp = api.playlist_detail(COLLECTION_ID, None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_playlist_tracks() {}

    #[tokio::test(flavor = "multi_thread")]
    async fn test_playlist_update() {}

    #[tokio::test(flavor = "multi_thread")]
    async fn test_recommend_resource() {
        let api = NcmApi::default();
        let resp = api.recommend_resource().await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_recommend_songs() {
        let api = NcmApi::default();
        let resp = api.recommend_songs().await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_scrobble() {
        let api = NcmApi::default();
        let resp = api.scrobble(29106885, COLLECTION_ID).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_search_default() {
        let api = NcmApi::default();
        let resp = api.search_default().await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_search_hot_detail() {
        let api = NcmApi::default();
        let resp = api.search_hot_detail().await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_search_hot() {
        let api = NcmApi::default();
        let resp = api.search_hot().await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_search_suggest() {
        let api = NcmApi::default();
        let resp = api.search_suggest("mota", None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_simi_artist() {
        let api = NcmApi::default();
        let resp = api.simi_artist(ARTIST_ID).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_simi_playlist() {
        let api = NcmApi::default();
        let resp = api.simi_playlist(SONG_ID, None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_simi_song() {
        let api = NcmApi::default();
        let resp = api.simi_song(SONG_ID, None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_song_detail() {
        let api = NcmApi::default();
        let resp = api.song_detail(&vec![SONG_ID]).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_song_url() {
        let api = NcmApi::default();
        let resp = api.song_url(&vec![SONG_ID]).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_user_account() {
        let api = NcmApi::default();
        let resp = api.user_account().await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_user_cloud_detail() {
        // let api = NcmApi::default();
        // let resp = api.user_cloud_detail().await;
        // assert!(resp.is_ok());

        // let res = resp.unwrap();
        // let res = res.deserialize_to_implict();
        // assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_user_cloud() {
        let api = NcmApi::default();
        let resp = api.user_cloud(None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_user_comment_history() {
        let api = NcmApi::default();
        let resp = api.user_comment_history(USER_ID, None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_user_detail() {
        let api = NcmApi::default();
        let resp = api.user_detail(USER_ID).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_dj() {
        let api = NcmApi::default();
        let resp = api.user_dj(USER_ID, None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_user_level() {
        let api = NcmApi::default();
        let resp = api.user_level().await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_user_playlist() {
        let api = NcmApi::default();
        let resp = api.user_playlist(USER_ID, None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_user_record() {
        let api = NcmApi::default();
        let resp = api.user_record(USER_ID, None).await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_user_subcount() {
        let api = NcmApi::default();
        let resp = api.user_subcount().await;
        assert!(resp.is_ok());

        let res = resp.unwrap();
        let res = res.deserialize_to_implict();
        assert_eq!(res.code, 200);
    }
}
