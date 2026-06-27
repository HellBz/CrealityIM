mod api;
mod ws;

use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::{Emitter, Manager};

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

fn app_data_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
    let dir = base.join("CrealityIM");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

// ── Credentials (OS Keystore — Windows Credential Manager / macOS Keychain / libsecret) ──

const KEYRING_SERVICE: &str = "CrealityIM";

#[tauri::command]
fn save_credentials(user_id: String, token: String) -> Result<(), String> {
    let data = json!({"user_id": user_id, "token": token}).to_string();
    keyring::Entry::new(KEYRING_SERVICE, "credentials")
        .map_err(|e| e.to_string())?
        .set_password(&data)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn load_credentials() -> Result<Value, String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, "credentials")
        .map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(data) => {
            let v: Value = serde_json::from_str(&data).unwrap_or(Value::Null);
            if v.get("user_id").and_then(|x| x.as_str()).map(|s| !s.is_empty()).unwrap_or(false)
                && v.get("token").and_then(|x| x.as_str()).map(|s| !s.is_empty()).unwrap_or(false)
            {
                Ok(v)
            } else {
                Ok(Value::Null)
            }
        }
        Err(_) => {
            // Migration: alte login.json in Keystore übertragen
            let path = app_data_dir().join("login.json");
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let v: Value = serde_json::from_str(&content).unwrap_or(Value::Null);
                    if v.get("user_id").and_then(|x| x.as_str()).map(|s| !s.is_empty()).unwrap_or(false)
                        && v.get("token").and_then(|x| x.as_str()).map(|s| !s.is_empty()).unwrap_or(false)
                    {
                        let _ = entry.set_password(&content);
                        let _ = std::fs::remove_file(&path);
                        return Ok(v);
                    }
                }
            }
            Ok(Value::Null)
        }
    }
}

#[tauri::command]
fn delete_credentials() -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, "credentials")
        .map_err(|e| e.to_string())?;
    let _ = entry.delete_credential();
    Ok(())
}

// ── User Cache ──

#[tauri::command]
fn save_user_cache(cache: Value) -> Result<(), String> {
    let path = app_data_dir().join("users.json");
    std::fs::write(&path, serde_json::to_string_pretty(&cache).unwrap())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn load_user_cache() -> Result<Value, String> {
    let path = app_data_dir().join("users.json");
    if !path.exists() {
        return Ok(json!({}));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    Ok(serde_json::from_str(&content).unwrap_or(json!({})))
}

// ── Settings ──

#[tauri::command]
fn get_settings() -> Result<Value, String> {
    let path = app_data_dir().join("settings.json");
    let defaults = json!({"notifications": true, "auto_login": true});
    if !path.exists() {
        return Ok(defaults);
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut data: Value = serde_json::from_str(&content).unwrap_or(json!({}));
    if data.get("notifications").is_none() { data["notifications"] = json!(true); }
    if data.get("auto_login").is_none() { data["auto_login"] = json!(true); }
    Ok(data)
}

#[tauri::command]
fn save_settings(settings: Value) -> Result<(), String> {
    let path = app_data_dir().join("settings.json");
    std::fs::write(&path, serde_json::to_string_pretty(&settings).unwrap())
        .map_err(|e| e.to_string())
}

// ── Native Notifications via Tauri Plugin ──

use tauri_plugin_notification::NotificationExt;

#[tauri::command]
fn show_notification(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}

// ── API Commands ──

#[tauri::command]
async fn im_login(token: String, user_id: String, cookie_str: String) -> Result<Value, String> {
    api::im_login(&token, &user_id, &cookie_str).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_user_info(token: String, my_uid: String, target_uid: String) -> Result<Value, String> {
    api::get_user_info(&token, &my_uid, &target_uid).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_users(token: String, uid: String, keyword: String) -> Result<Value, String> {
    api::search_users(&token, &uid, &keyword).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_models(token: String, uid: String, keyword: String) -> Result<Value, String> {
    api::search_models(&token, &uid, &keyword).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_my_models(token: String, uid: String) -> Result<Value, String> {
    api::get_my_models(&token, &uid).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_roam_messages(token: String, uid: String, sig: String, peer: String) -> Result<Value, String> {
    api::get_roam_messages(&token, &uid, &sig, &peer).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_message(token: String, uid: String, sig: String, body: Value) -> Result<Value, String> {
    api::send_message(&token, &uid, &sig, body).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_latest_browse_record(token: String, my_uid: String, other_uid: String) -> Result<Value, String> {
    api::get_latest_browse_record(&token, &my_uid, &other_uid).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn connect_ws(app: tauri::AppHandle, uid: String, sig: String) -> Result<(), String> {
    ws::connect(app, uid, sig).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn ws_send(app: tauri::AppHandle, msg: String) -> Result<bool, String> {
    Ok(ws::send_raw(&app, msg))
}

#[tauri::command]
async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_file(url: String, filename: String) -> Result<String, String> {
    // App-eigener Downloads-Ordner: %APPDATA%\CrealityIM\downloads\
    let downloads = app_data_dir().join("downloads");
    let _ = std::fs::create_dir_all(&downloads);

    // Sicheren Dateinamen erzeugen (keine Pfad-Traversal)
    let safe_name = std::path::Path::new(&filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download")
        .to_string();

    let dest = downloads.join(&safe_name);

    // Bereits vorhanden → direkt öffnen, nicht erneut laden
    let already_exists = dest.exists();
    if !already_exists {
        let client = reqwest::Client::new();
        let bytes = client.get(&url)
            .send().await.map_err(|e| e.to_string())?
            .bytes().await.map_err(|e| e.to_string())?;
        std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
    }

    // Ordner öffnen mit Datei markiert (Windows) oder Ordner direkt (andere)
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer")
        .args(["/select,", &dest.to_string_lossy().to_string()])
        .spawn();
    #[cfg(not(target_os = "windows"))]
    let _ = open::that(&downloads);

    let name = dest.file_name().unwrap_or_default().to_string_lossy().to_string();
    Ok(if already_exists { format!("cached:{}", name) } else { format!("saved:{}", name) })
}

// ── OAuth Login via id.creality.com ──

#[tauri::command]
async fn oauth_token_received(app: tauri::AppHandle, token: String, user_id: String) -> Result<(), String> {
    debug_log!("[oauth] Token received from WebView, userId={}", user_id);
    if let Some(win) = app.get_webview_window("oauth-login") {
        let _ = win.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let js = format!(
            "window._oauthLogin && window._oauthLogin({}, {});",
            serde_json::json!(token),
            serde_json::json!(user_id)
        );
        let _ = main.eval(&js);
        main.emit("oauth_token", serde_json::json!({"token": token, "userId": user_id})).ok();
    }
    Ok(())
}

#[tauri::command]
async fn oauth_login_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::{WebviewWindowBuilder, Manager};

    let login_url = "https://id.creality.com/?lang=en-US&response_type=token&client_id=f9c302ecc29c59a0a6e921ff39a073ca&app_id=creality_model&platform=2&webview=1";

    // Falls ein altes Fenster noch offen ist, schließen
    if let Some(old) = app.get_webview_window("oauth-login") {
        let _ = old.close();
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    // Temporäres Profil-Verzeichnis erstellen (frische Session ohne alter Cookies)
    let tmp_profile = std::env::temp_dir().join("creality-oauth-profile");
    // Altes Profil löschen damit keine Session wiederverwendet wird
    let _ = std::fs::remove_dir_all(&tmp_profile);
    std::fs::create_dir_all(&tmp_profile).ok();

    let win = WebviewWindowBuilder::new(&app, "oauth-login", tauri::WebviewUrl::External(login_url.parse().unwrap()))
        .title("Sign in with Creality")
        .inner_size(500.0, 700.0)
        .center()
        .resizable(true)
        .data_directory(tmp_profile)
        .build()
        .map_err(|e| e.to_string())?;

    // JavaScript injizieren, der auf localStorage Änderungen hört und Token an Rust sendet
    let js_bridge = r#"
        (function() {
            const originalSetItem = localStorage.setItem;
            localStorage.setItem = function(key, value) {
                originalSetItem.call(this, key, value);
                if (key === 'id-application-user') {
                    try {
                        const raw = localStorage.getItem('id-application-user');
                        if (raw) {
                            const arr = JSON.parse(raw);
                            const obj = Array.isArray(arr) ? arr[0] : arr;
                            if (obj && obj.token && obj.token.length > 10) {
                                if (window.__TAURI__) {
                                    window.__TAURI__.invoke('oauth_token_received', {
                                        token: obj.token,
                                        userId: String(obj.userId || '')
                                    }).catch(() => {});
                                }
                            }
                        }
                    } catch(e) {}
                }
            };
            // Auch beim Laden prüfen (falls Token schon vorhanden)
            setTimeout(() => {
                try {
                    const raw = localStorage.getItem('id-application-user');
                    if (raw) {
                        const arr = JSON.parse(raw);
                        const obj = Array.isArray(arr) ? arr[0] : arr;
                        if (obj && obj.token && obj.token.length > 10) {
                            if (window.__TAURI__) {
                                window.__TAURI__.invoke('oauth_token_received', {
                                    token: obj.token,
                                    userId: String(obj.userId || '')
                                }).catch(() => {});
                            }
                        }
                    }
                } catch(e) {}
            }, 1000);
        })();
    "#;
    let _ = win.eval(js_bridge);

    Ok(())
}

#[tauri::command]
async fn upload_file(upload_url: String, file_base64: String, mime_type: String) -> Result<(), String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&file_base64)
        .map_err(|e| e.to_string())?;
    // URL-Teile separat encodieren: Basis-URL + Query-Parameter bleiben, nur Pfad-Teil encodieren
    let encoded_url = if let Some(qmark) = upload_url.find('?') {
        let (path_part, query_part) = upload_url.split_at(qmark);
        // Letzten Pfad-Segment (Dateiname) encodieren
        if let Some(slash) = path_part.rfind('/') {
            let (base, filename) = path_part.split_at(slash + 1);
            format!("{}{}{}", base, urlencoding::encode(filename), query_part)
        } else {
            upload_url.clone()
        }
    } else {
        upload_url.clone()
    };
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .put(&encoded_url)
        .header("Accept", "*/*")
        .header("Content-Type", if mime_type.is_empty() { "application/octet-stream".to_string() } else { mime_type.clone() })
        .body(bytes)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else {
        let body = resp.text().await.unwrap_or_default();
        debug_log!("[upload_file] failed: {} body: {}", status, &body[..body.len().min(200)]);
        Err(format!("Upload failed: HTTP {} - {}", status, &body[..body.len().min(500)]))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ws::WsSender(std::sync::Mutex::new(None)))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            im_login,
            get_user_info,
            search_users,
            search_models,
            get_my_models,
            get_roam_messages,
            send_message,
            get_latest_browse_record,
            connect_ws,
            save_credentials,
            load_credentials,
            delete_credentials,
            save_user_cache,
            load_user_cache,
            get_settings,
            save_settings,
            show_notification,
            ws_send,
            open_url,
            download_file,
            upload_file,
            oauth_login_window,
            oauth_token_received,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
