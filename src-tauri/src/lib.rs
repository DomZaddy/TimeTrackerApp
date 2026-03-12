mod sheets;
mod oauth;

use tauri::{
    AppHandle, Manager, Runtime,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// App state shared across commands
pub struct AppState {
    pub oauth_tokens: Mutex<Option<oauth::OAuthTokens>>,
    pub live_status_row: Mutex<Option<i64>>,
    pub live_status_tab: Mutex<Option<String>>,
    pub badge_count: Mutex<u32>,
    pub reminder_active: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            oauth_tokens: Mutex::new(None),
            live_status_row: Mutex::new(None),
            live_status_tab: Mutex::new(None),
            badge_count: Mutex::new(0),
            reminder_active: Mutex::new(false),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PushResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_count: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WeekHoursResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub week_ms: f64,
    #[serde(default)]
    pub monthly_pay: f64,
    #[serde(default)]
    pub monthly_hours: f64,
    #[serde(default)]
    pub week_start: f64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionBlock {
    pub date: String,
    pub task: String,
    pub check_in: String,
    pub check_out: String,
    pub break_hours: f64,
    pub notes: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LiveStatusData {
    pub task: String,
    pub start_time: f64,
    #[serde(default)]
    pub elapsed: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NudgeData {
    pub title: String,
    pub body: String,
}

// Reminder messages
const REMINDER_MESSAGES: &[&str] = &[
    "Take a sec to jot down what you've been working on.",
    "Hey! It's been a while. Drop some notes before you forget.",
    "Time to log your progress!",
    "What have you been crushing? Add a note.",
    "Don't lose track — note down your current work.",
    "Nice grind! Take a moment to log what you've done so far.",
];

#[tauri::command]
async fn sheets_auth(app: AppHandle) -> AuthResult {
    match oauth::start_oauth_flow(&app).await {
        Ok(tokens) => {
            let state = app.state::<AppState>();
            *state.oauth_tokens.lock().unwrap() = Some(tokens);
            AuthResult { success: true, error: None }
        }
        Err(e) => AuthResult { success: false, error: Some(e.to_string()) }
    }
}

#[tauri::command]
async fn sheets_sign_out(app: AppHandle) -> AuthResult {
    let state = app.state::<AppState>();
    *state.oauth_tokens.lock().unwrap() = None;
    // Also clear from persistent store
    if let Ok(store) = app.store("timetracker-store.json") {
        let _ = store.delete("oauth-tokens");
        let _ = store.save();
    }
    AuthResult { success: true, error: None }
}

#[tauri::command]
async fn sheets_push(app: AppHandle, rows: Vec<SessionBlock>) -> PushResult {
    match sheets::push_session(&app, rows).await {
        Ok(count) => PushResult { success: true, error: None, row_count: Some(count) },
        Err(e) => PushResult { success: false, error: Some(e.to_string()), row_count: None },
    }
}

#[tauri::command]
async fn sheets_status_start(app: AppHandle, data: LiveStatusData) -> PushResult {
    match sheets::status_start(&app, data).await {
        Ok(row) => PushResult { success: true, error: None, row_count: Some(row as usize) },
        Err(e) => PushResult { success: false, error: Some(e.to_string()), row_count: None },
    }
}

#[tauri::command]
async fn sheets_status_update(app: AppHandle, data: LiveStatusData) -> PushResult {
    match sheets::status_update(&app, data).await {
        Ok(_) => PushResult { success: true, error: None, row_count: None },
        Err(e) => PushResult { success: false, error: Some(e.to_string()), row_count: None },
    }
}

#[tauri::command]
async fn sheets_status_clear(app: AppHandle) -> PushResult {
    match sheets::status_clear(&app).await {
        Ok(_) => PushResult { success: true, error: None, row_count: None },
        Err(e) => PushResult { success: false, error: Some(e.to_string()), row_count: None },
    }
}

#[tauri::command]
async fn sheets_fetch_week_hours(app: AppHandle) -> WeekHoursResult {
    match sheets::fetch_week_hours(&app).await {
        Ok(result) => result,
        Err(e) => WeekHoursResult {
            success: false,
            error: Some(e.to_string()),
            week_ms: 0.0,
            monthly_pay: 0.0,
            monthly_hours: 0.0,
            week_start: 0.0,
        },
    }
}

#[tauri::command]
async fn send_nudge<R: Runtime>(app: AppHandle<R>, data: NudgeData) {
    let _ = app.notification()
        .builder()
        .title(&data.title)
        .body(&data.body)
        .show();
}

#[tauri::command]
async fn send_reminder<R: Runtime>(app: AppHandle<R>) {
    let msg = REMINDER_MESSAGES[rand_index(REMINDER_MESSAGES.len())];
    let _ = app.notification()
        .builder()
        .title("Check-In Reminder")
        .body(msg)
        .show();
}

fn rand_index(max: usize) -> usize {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as usize;
    nanos % max
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::default())
        .setup(|app| {
            // Load saved OAuth tokens from store
            let store = app.store("timetracker-store.json")
                .expect("Failed to create store");
            if let Some(tokens_val) = store.get("oauth-tokens") {
                if let Ok(tokens) = serde_json::from_value::<oauth::OAuthTokens>(tokens_val.clone()) {
                    let state = app.state::<AppState>();
                    *state.oauth_tokens.lock().unwrap() = Some(tokens);
                }
            }

            // Restore live status row/tab so status_update works after restart
            {
                let state = app.state::<AppState>();
                if let Some(row_val) = store.get("live-status-row") {
                    if let Some(row) = row_val.as_i64() {
                        *state.live_status_row.lock().unwrap() = Some(row);
                    }
                }
                if let Some(tab_val) = store.get("live-status-tab") {
                    if let Some(tab) = tab_val.as_str() {
                        *state.live_status_tab.lock().unwrap() = Some(tab.to_string());
                    }
                }
            }

            // Build tray
            let show = MenuItemBuilder::with_id("show", "Open TimeTracker").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show)
                .separator()
                .item(&quit)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&menu)
                .tooltip("TimeTracker")
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sheets_auth,
            sheets_sign_out,
            sheets_push,
            sheets_status_start,
            sheets_status_update,
            sheets_status_clear,
            sheets_fetch_week_hours,
            send_nudge,
            send_reminder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
