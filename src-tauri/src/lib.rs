mod blocklist;
#[cfg(windows)]
mod browser_kill;
mod monitor;
mod window_corner;
#[cfg(windows)]
mod win_titles;

use monitor::spawn_monitor;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[derive(Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorSnapshot {
    /// UI용: 이 빌드가 Windows에서 전체 기능을 제공하는지
    #[serde(default)]
    pub windows_supported: bool,
    pub scans: u64,
    pub last_matched_window_title: Option<String>,
    pub last_matched_keyword: Option<String>,
    pub last_kill_count: u32,
    pub total_kill_attempts: u64,
    pub last_error: Option<String>,
}

#[derive(Clone)]
pub struct MonitorState(pub Arc<Mutex<MonitorSnapshot>>);

#[tauri::command]
fn get_monitor_snapshot(state: tauri::State<MonitorState>) -> MonitorSnapshot {
    let mut snap = state.0.lock().unwrap().clone();
    snap.windows_supported = cfg!(windows);
    snap
}

/// 커서가 창에 가까이 오면(창 밖 접근 띠) 다른 화면 모서리로 옮깁니다.
#[tauri::command]
fn tick_window_flee(app: tauri::AppHandle) -> Option<u8> {
    window_corner::tick_window_flee(&app)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let snapshot = MonitorState(Arc::new(Mutex::new(MonitorSnapshot::default())));
    let snap_for_thread = snapshot.0.clone();

    tauri::Builder::default()
        .manage(snapshot)
        .setup(|app| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = window_corner::place_window_at_corner(&w, 0);
            }
            spawn_monitor(snap_for_thread);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_monitor_snapshot, tick_window_flee])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
