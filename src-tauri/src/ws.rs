use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::Message, Connector};
const SDKAPPID: u64 = 1721003041;
const WEBSDKAPPID: u64 = 537048168;
const PLATFORM: u32 = 7;
const SDKVERSION: &str = "1.7.3";

// Debug-Only Logging Macro - nur im Debug-Build aktiv
#[cfg(debug_assertions)]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

pub struct WsSender(pub Mutex<Option<mpsc::UnboundedSender<String>>>);

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn make_msg(cmd: &str, uid: &str, sig: &str, seq: u64, body: serde_json::Value) -> String {
    json!({
        "head": {
            "ver": "v4",
            "platform": PLATFORM,
            "websdkappid": WEBSDKAPPID,
            "websdkversion": SDKVERSION,
            "status_instid": 0,
            "sdkappid": SDKAPPID,
            "contenttype": "json",
            "reqtime": now_secs(),
            "identifier": uid,
            "usersig": sig,
            "sdkability": 192371,
            "tjgID": "",
            "servcmd": cmd,
            "seq": seq
        },
        "body": body
    }).to_string()
}

pub async fn connect(app: AppHandle, uid: String, sig: String) -> anyhow::Result<()> {
    let ts = now_secs();
    let instance_id = ts % 1_000_000_000;
    let random = (ts * 1000003) % 1_000_000_000;
    let url = format!(
        "wss://wsssgp.im.qcloud.com/binfo?sdkappid={}&instanceid={}&random={}&platform={}&host=windows&version=-1&sdkversion={}&compress=none",
        SDKAPPID, instance_id, random, PLATFORM, SDKVERSION
    );

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    {
        let state = app.state::<WsSender>();
        let mut guard = state.0.lock().unwrap();
        *guard = Some(tx.clone());
    }

    tokio::spawn(async move {
        let app_clone = app.clone();
        let tls = native_tls::TlsConnector::new().expect("TLS init failed");
        let connector = Connector::NativeTls(tls);

        debug_log!("[ws] connecting...");
        match connect_async_tls_with_config(&url, None, false, Some(connector)).await {
            Ok((ws_stream, _)) => {
                debug_log!("[ws] connected OK");
                let (mut write, mut read) = ws_stream.split();

                // wslogin direkt senden — kein Race Condition mit Frontend
                let login = make_msg("im_open_status.wslogin", &uid, &sig, now_secs(), json!({"State":"Online","is_web_uniapp":0,"InstType":0}));
                if let Err(e) = write.send(Message::Binary(login.into_bytes().into())).await {
                    debug_log!("[ws] wslogin send failed: {}", e);
                    let _ = app_clone.emit("ws_close", e.to_string());
                    return;
                }
                debug_log!("[ws] wslogin sent");

                // Frontend informieren (für UI-Status)
                let _ = app_clone.emit("ws_open", ());

                // Heartbeat-Task
                let tx_hb = tx.clone();
                let uid_hb = uid.clone();
                let sig_hb = sig.clone();
                tokio::spawn(async move {
                    let mut seq: u64 = 9_000_000;
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(25)).await;
                        let hb = make_msg("heartbeat.alive", &uid_hb, &sig_hb, seq, json!({}));
                        seq += 1;
                        if tx_hb.send(hb).is_err() { break; }
                        debug_log!("[ws] heartbeat sent");
                    }
                });

                // Sende-Task für ws_send vom Frontend
                let app_send = app_clone.clone();
                tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        debug_log!("[ws] sending msg len={}", msg.len());
                        if write.send(Message::Binary(msg.into_bytes().into())).await.is_err() {
                            let _ = app_send.emit("ws_close", "send failed");
                            break;
                        }
                    }
                });

                // Empfangen
                while let Some(item) = read.next().await {
                    match item {
                        Ok(Message::Text(txt)) => {
                            debug_log!("[ws] recv: {}", &txt[..txt.len().min(200)]);
                            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                                let _ = app_clone.emit("ws_message", val);
                            }
                        }
                        Ok(Message::Binary(bin)) => {
                            if let Ok(s) = std::str::from_utf8(&bin) {
                                debug_log!("[ws] recv bin: {}", &s[..s.len().min(200)]);
                                if let Ok(val) = serde_json::from_str::<Value>(s) {
                                    let _ = app_clone.emit("ws_message", val);
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            debug_log!("[ws] server closed");
                            let _ = app_clone.emit("ws_close", "closed");
                            break;
                        }
                        Err(e) => {
                            debug_log!("[ws] recv error: {}", e);
                            let _ = app_clone.emit("ws_close", e.to_string());
                            break;
                        }
                        _ => {}
                    }
                }

                if let Some(state) = app_clone.try_state::<WsSender>() {
                    let mut guard = state.0.lock().unwrap();
                    *guard = None;
                }
            }
            Err(e) => {
                debug_log!("[ws] connect FAILED: {}", e);
                let _ = app.emit("ws_close", e.to_string());
                if let Some(state) = app.try_state::<WsSender>() {
                    let mut guard = state.0.lock().unwrap();
                    *guard = None;
                }
            }
        }
    });

    Ok(())
}

pub fn send_raw(app: &AppHandle, msg: String) -> bool {
    if let Some(state) = app.try_state::<WsSender>() {
        let guard = state.0.lock().unwrap();
        if let Some(tx) = guard.as_ref() {
            return tx.send(msg).is_ok();
        }
    }
    false
}
