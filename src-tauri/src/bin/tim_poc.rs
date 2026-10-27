// PoC: Native ImSDK (ImSDK.dll) -> TIMLogin -> TIMConvGetConvList
// Usage: tim_poc <user_id> <token>   (holt userSig via /api/rest/im/account/login)
use anyhow::Result;
use libloading::{Library, Symbol};
use serde_json::{json, Value};
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::mpsc;
use std::time::Duration;

type TIMCommCallback = extern "C" fn(i32, *const c_char, *const c_char, *const c_void);
type FnInit = unsafe extern "C" fn(u64, *const c_char) -> i32;
type FnUninit = unsafe extern "C" fn() -> i32;
type FnLogin = unsafe extern "C" fn(*const c_char, *const c_char, TIMCommCallback, *const c_void) -> i32;
type FnConvList = unsafe extern "C" fn(TIMCommCallback, *const c_void) -> i32;

extern "C" fn cb(code: i32, desc: *const c_char, json_params: *const c_char, user_data: *const c_void) {
    let d = unsafe { CStr::from_ptr(desc) }.to_string_lossy().into_owned();
    let j = if json_params.is_null() { String::new() } else { unsafe { CStr::from_ptr(json_params) }.to_string_lossy().into_owned() };
    println!("[cb] code={} desc={}", code, d);
    if !j.is_empty() { println!("[cb] json={}", &j[..j.len().min(3000)]); }
    if !user_data.is_null() {
        unsafe { let tx = &*(user_data as *const mpsc::Sender<(i32, String)>); let _ = tx.send((code, j)); }
    }
}

async fn im_rest_login(uid: &str, token: &str) -> Result<(String, u64)> {
    let client = reqwest::Client::new();
    let mut h = reqwest::header::HeaderMap::new();
    for (k, v) in [
        ("__cxy_app_id_", "cxy-gen2"), ("__cxy_app_ver_", "7.3.10"), ("__cxy_platform_", "2"),
        ("__cxy_brand_", "creality"), ("__cxy_token_", token), ("__cxy_uid_", uid),
        ("__cxy_requestid_", &uuid::Uuid::new_v4().to_string()),
        ("__cxy_duid_", &format!("client-{}", uuid::Uuid::new_v4())),
        ("__cxy_os_", "windows"), ("__cxy_os_ver_", "Windows 10"),
        ("__cxy_os_lang_", "0"), ("__cxy_timezone_", "7200"),
        ("Origin", "https://www.crealitycloud.com"),
    ] { h.insert(k, v.parse().unwrap()); }
    h.insert("Content-Type", "application/json".parse().unwrap());
    h.insert("Cookie", format!("model_token={}; model_user_id={}; model_platform_type=2", token, uid).parse().unwrap());

    let r: Value = client.post("https://www.crealitycloud.com/api/rest/im/account/login")
        .headers(h.clone()).json(&json!({})).send().await?.json().await?;
    let sig = r["result"]["userSig"].as_str().unwrap_or("").to_string();
    if sig.is_empty() { anyhow::bail!("login failed: {}", r); }

    let mut app_id: u64 = 1721003041;
    let r2: Value = client.post("https://www.crealitycloud.com/api/rest/im/app/getAppId")
        .headers(h).json(&json!({})).send().await?.json().await.unwrap_or(Value::Null);
    if let Some(n) = r2["result"]["sdkAppId"].as_u64().or_else(|| r2["result"]["appId"].as_u64()) { app_id = n; }
    println!("[poc] userSig ok, sdkAppId={}", app_id);
    Ok((sig, app_id))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let uid = args.get(1).cloned().unwrap_or_else(|| "8749025865".into());
    let token = args.get(2).cloned().unwrap_or_else(|| "1fd10af3fc6d2eaedae63bd719678d66e2328a13d4330d35df7e4e652f6e759d".into());
    let (sig, app_id) = im_rest_login(&uid, &token).await?;

    // SDK-Calls auf eigenem Thread (nicht Main): Callbacks feuern dann
    // auf SDK-internen Threads statt einen Message-Loop zu brauchen.
    let handle = std::thread::spawn(move || -> Result<()> {
    unsafe {
        let lib = Library::new("ImSDK.dll")?;
        let init: Symbol<FnInit> = lib.get(b"TIMInit")?;
        let login: Symbol<FnLogin> = lib.get(b"TIMLogin")?;
        let conv: Symbol<FnConvList> = lib.get(b"TIMConvGetConvList")?;
        let uninit: Symbol<FnUninit> = lib.get(b"TIMUninit")?;

        let cfg = CString::new(json!({
            "sdk_config_log_level": 4,
            "sdk_config_log_file_path": ".",
            "sdk_config_config_file_path": "."
        }).to_string())?;
        let rc = init(app_id, cfg.as_ptr());
        println!("[poc] TIMInit rc={}", rc);

        let (tx, rx) = mpsc::channel::<(i32, String)>();
        let uid_c = CString::new(uid.clone())?;
        let sig_c = CString::new(sig)?;
        let rc = login(uid_c.as_ptr(), sig_c.as_ptr(), cb, &tx as *const _ as *const c_void);
        println!("[poc] TIMLogin call rc={}", rc);
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok((code, _)) => println!("[poc] login cb code={}", code),
            Err(_) => println!("[poc] login TIMEOUT"),
        }

        let (tx2, rx2) = mpsc::channel::<(i32, String)>();
        let rc = conv(cb, &tx2 as *const _ as *const c_void);
        println!("[poc] TIMConvGetConvList call rc={}", rc);
        match rx2.recv_timeout(Duration::from_secs(30)) {
            Ok((code, j)) => {
                println!("[poc] convlist cb code={} items:", code);
                if let Ok(v) = serde_json::from_str::<Value>(&j) {
                    let n = v.as_array().map(|a| a.len()).unwrap_or(0);
                    println!("[poc] conversation count = {}", n);
                    if let Some(a) = v.as_array() {
                        for c in a.iter().take(50) {
                            println!("  {}", serde_json::to_string(c).unwrap_or_default().chars().take(300).collect::<String>());
                        }
                    }
                } else { println!("[poc] raw: {}", &j[..j.len().min(2000)]); }
            }
            Err(_) => println!("[poc] convlist TIMEOUT"),
        }
        let _ = uninit();
    }
    Ok(())
    });
    handle.join().map_err(|_| anyhow::anyhow!("thread panicked"))??;
    std::process::exit(0);
}
