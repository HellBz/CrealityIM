use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

fn make_client() -> Client {
    Client::builder().build().unwrap()
}

fn base_headers(token: &str, uid: &str) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert("__cxy_app_id_", "cxy-gen2".parse().unwrap());
    headers.insert("__cxy_app_ver_", "7.3.10".parse().unwrap());
    headers.insert("__cxy_platform_", "2".parse().unwrap());
    headers.insert("__cxy_brand_", "creality".parse().unwrap());
    headers.insert("__cxy_token_", token.parse().unwrap());
    headers.insert("__cxy_uid_", uid.parse().unwrap());
    headers.insert("__cxy_requestid_", Uuid::new_v4().to_string().parse().unwrap());
    headers.insert("__cxy_duid_", format!("client-{}", Uuid::new_v4()).parse().unwrap());
    headers.insert("__cxy_os_", "windows".parse().unwrap());
    headers.insert("__cxy_os_ver_", "Windows 10".parse().unwrap());
    headers.insert("__cxy_os_lang_", "0".parse().unwrap());
    headers.insert("__cxy_timezone_", "7200".parse().unwrap());
    headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)".parse().unwrap());
    headers.insert("Origin", "https://www.crealitycloud.com".parse().unwrap());
    headers.insert("Cookie", format!("model_token={}; model_user_id={}; model_platform_type=2", token, uid).parse().unwrap());
    headers
}

const CF_CLEARANCE: &str = "0LVaBD6stBfzUNvqWCXAMRQmqSk80LmkGxRhaLZVR4Q-1782061734-1.2.1.1-8Xy6gu3JEeM2NFNuf_T.PLH27niHEbpb5iRrDLFSIsSssvTq61iF5KhZtaB_0449Ye3u1FIFUBS1WI_T8OF33CpoCIN4q6y0p4EsLdt_QZu1KY6jWLk_5uLpMv.KyCeyNrsyV6FTRtPJPdXrrZs_zagysi3wXrj5vErWsZvsEFeGR6y.tQRsQ1HeQrJ60koukky21D5PDIdR.yqZyNkur2JuvzexvJSmARysXjSLwNhQ4d6BGVX297uSmsIRsjBSg.cQhyxjpU2IZcfF0eeWxPuVGwYN2yNGCq7mEHEpiyzEPnVSDZPvfLHroUgJF4l8NDU7cZbnAq7eo4Cb1KmL4Vq63OSf9fmBXF4gkDd...n.ejweJHKLTuFP6EISjjM830BJEmqqsAAceuTnHYjxt2DhS0XKqQoJ7JxkjAOFcXWPhNhdT21LgLDToV1zeA68";

pub async fn im_login(token: &str, user_id: &str, _cookie_str: &str) -> anyhow::Result<Value> {
    let client = make_client();
    let duid = format!("client-{}", Uuid::new_v4());
    let cookie = format!(
        "model_device_id={}; model_os_version=Windows%2010; model_platform_type=2; model_lang=0; timeZone=7200; model_token={}; model_user_id={}; cf_clearance={}",
        duid, token, user_id, CF_CLEARANCE
    );
    let mut hdrs = reqwest::header::HeaderMap::new();
    hdrs.insert("accept", "application/json".parse().unwrap());
    hdrs.insert("accept-language", "de-DE,de;q=0.9,en-US;q=0.8,en;q=0.7".parse().unwrap());
    hdrs.insert("content-type", "application/json".parse().unwrap());
    hdrs.insert("origin", "https://www.crealitycloud.com".parse().unwrap());
    hdrs.insert("__cxy_app_ch_", "Chrome 149.0.0.0".parse().unwrap());
    hdrs.insert("__cxy_app_id_", "cxy-gen2".parse().unwrap());
    hdrs.insert("__cxy_app_ver_", "7.3.10".parse().unwrap());
    hdrs.insert("__cxy_brand_", "creality".parse().unwrap());
    hdrs.insert("__cxy_duid_", duid.parse().unwrap());
    hdrs.insert("__cxy_os_lang_", "0".parse().unwrap());
    hdrs.insert("__cxy_os_ver_", "Windows 10".parse().unwrap());
    hdrs.insert("__cxy_platform_", "2".parse().unwrap());
    hdrs.insert("__cxy_requestid_", Uuid::new_v4().to_string().parse().unwrap());
    hdrs.insert("__cxy_timezone_", "7200".parse().unwrap());
    hdrs.insert("__cxy_token_", token.parse().unwrap());
    hdrs.insert("__cxy_uid_", user_id.parse().unwrap());
    hdrs.insert("sec-ch-ua", "\"Google Chrome\";v=\"149\", \"Chromium\";v=\"149\", \"Not)A;Brand\";v=\"24\"".parse().unwrap());
    hdrs.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
    hdrs.insert("sec-ch-ua-platform", "\"Windows\"".parse().unwrap());
    hdrs.insert("sec-fetch-dest", "empty".parse().unwrap());
    hdrs.insert("sec-fetch-mode", "cors".parse().unwrap());
    hdrs.insert("sec-fetch-site", "same-origin".parse().unwrap());
    hdrs.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36".parse().unwrap());
    hdrs.insert("Cookie", cookie.parse().unwrap());
    let raw = client
        .post("https://www.crealitycloud.com/api/rest/im/account/login")
        .headers(hdrs)
        .json(&json!({}))
        .send().await?;
    let resp: Value = raw.json().await?;
    Ok(resp)
}

pub async fn get_user_info(token: &str, my_uid: &str, target_uid: &str) -> anyhow::Result<Value> {
    let client = make_client();
    let uid_int: i64 = target_uid.parse().unwrap_or(0);
    let hdrs = base_headers(token, my_uid);
    let resp = client
        .post("https://www.crealitycloud.com/api/rest/im/account/userInfo")
        .headers(hdrs)
        .json(&json!({"cxyUserId": uid_int}))
        .send().await?
        .json::<Value>().await?;
    Ok(resp)
}

pub async fn search_users(token: &str, uid: &str, keyword: &str) -> anyhow::Result<Value> {
    let client = make_client();
    let duid = format!("client-{}", Uuid::new_v4());
    let cookie = format!("model_token={}; model_user_id={}; model_platform_type=2", token, uid);
    let mut hdrs = reqwest::header::HeaderMap::new();
    hdrs.insert("accept", "application/json, text/plain, */*".parse().unwrap());
    hdrs.insert("content-type", "application/json".parse().unwrap());
    hdrs.insert("origin", "https://www.crealitycloud.com".parse().unwrap());
    hdrs.insert("__cxy_app_ch_", "Chrome 149.0.0.0".parse().unwrap());
    hdrs.insert("__cxy_app_id_", "creality_model".parse().unwrap());
    hdrs.insert("__cxy_app_ver_", "7.3.10".parse().unwrap());
    hdrs.insert("__cxy_brand_", "creality".parse().unwrap());
    hdrs.insert("__cxy_duid_", duid.parse().unwrap());
    hdrs.insert("__cxy_os_lang_", "0".parse().unwrap());
    hdrs.insert("__cxy_os_ver_", "Windows 10".parse().unwrap());
    hdrs.insert("__cxy_platform_", "2".parse().unwrap());
    hdrs.insert("__cxy_requestid_", Uuid::new_v4().to_string().parse().unwrap());
    hdrs.insert("__cxy_timezone_", "7200".parse().unwrap());
    hdrs.insert("__cxy_token_", token.parse().unwrap());
    hdrs.insert("__cxy_uid_", uid.parse().unwrap());
    hdrs.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36".parse().unwrap());
    hdrs.insert("Cookie", cookie.parse().unwrap());
    let resp = client
        .post("https://www.crealitycloud.com/api/cxy/search/userSearch")
        .headers(hdrs)
        .json(&json!({"keyword": keyword, "page": 1, "pageSize": 20, "isModeler": false}))
        .send().await?
        .json::<Value>().await?;
    Ok(resp)
}

pub async fn search_models(token: &str, uid: &str, keyword: &str) -> anyhow::Result<Value> {
    let client = make_client();
    let resp = client
        .post("https://www.crealitycloud.com/api/cxy/smart_search/v1/model")
        .headers(base_headers(token, uid))
        .json(&json!({"keyword": keyword, "page": 1, "pageSize": 20}))
        .send().await?
        .json::<Value>().await?;
    Ok(resp)
}

pub async fn get_my_models(token: &str, uid: &str) -> anyhow::Result<Value> {
    let client = make_client();
    let resp = client
        .post("https://www.crealitycloud.com/api/cxy/v3/folder/folderList")
        .headers(base_headers(token, uid))
        .json(&json!({"page": 1, "pageSize": 20, "nodeType": 1}))
        .send().await?
        .json::<Value>().await?;
    Ok(resp)
}

pub async fn get_roam_messages(token: &str, uid: &str, _sig: &str, peer: &str) -> anyhow::Result<Value> {
    let client = make_client();
    let resp = client
        .post("https://www.crealitycloud.com/api/rest/im/msg/getRoamMsg")
        .headers(base_headers(token, uid))
        .json(&json!({
            "Peer_Account": peer,
            "MaxCnt": 20,
            "MinTime": 0,
            "MaxTime": 0,
            "LastMsgTime": 0,
            "LastMsgKey": 0
        }))
        .send().await?
        .json::<Value>().await?;
    Ok(resp)
}

pub async fn send_message(token: &str, uid: &str, _sig: &str, body: Value) -> anyhow::Result<Value> {
    let client = make_client();
    let resp = client
        .post("https://www.crealitycloud.com/api/rest/im/msg/sendmsg")
        .headers(base_headers(token, uid))
        .json(&body)
        .send().await?
        .json::<Value>().await?;
    Ok(resp)
}

pub async fn get_latest_browse_record(token: &str, my_uid: &str, other_uid: &str) -> anyhow::Result<Value> {
    let client = make_client();
    let hdrs = base_headers(token, my_uid);
    let resp = client
        .post("https://www.crealitycloud.com/api/rest/im/account/latestBrowseRecord")
        .headers(hdrs)
        .json(&json!({"cxyUserId": other_uid}))
        .send().await?
        .json::<Value>().await?;
    Ok(resp)
}
