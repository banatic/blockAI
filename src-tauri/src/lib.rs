mod blocklist;
#[cfg(windows)]
mod browser_kill;
mod monitor;
mod window_corner;
#[cfg(windows)]
mod win_titles;
#[cfg(windows)]
mod window_blur;

use monitor::spawn_monitor;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg(target_os = "windows")]
use window_vibrancy::apply_acrylic;

#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;

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
                #[cfg(target_os = "windows")]
                {
                    let _ = apply_acrylic(&w, None);
                    
                    // Windows 11 시스템 라운딩 강제 적용
                    if let Ok(hwnd) = w.hwnd() {
                        unsafe {
                            let preference = DWMWCP_ROUND;
                            let _ = DwmSetWindowAttribute(
                                HWND(hwnd.0 as _),
                                DWMWA_WINDOW_CORNER_PREFERENCE,
                                &preference as *const _ as *const _,
                                std::mem::size_of::<i32>() as u32,
                            );
                        }
                    }
                }
            }
            spawn_monitor(snap_for_thread);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_monitor_snapshot, tick_window_flee])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
