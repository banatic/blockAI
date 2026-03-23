//! 모서리 고정 + 커서가 위젯에 **다가올 때**(창 밖 접근 영역) 다른 모서리로 이동.
//! 창 **안**에 커서가 있을 때는 이동하지 않아 내용을 읽을 수 있습니다.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};

static CORNER_IDX: AtomicU8 = AtomicU8::new(0);
static LAST_FLEE: Mutex<Option<Instant>> = Mutex::new(None);

const EDGE_MARGIN: i32 = 14;
/// 창 바깥으로 이 픽셀만큼 확장한 띠 안에 커서가 들어오면 ‘접근 중’으로 간주합니다.
const APPROACH_PAD: i32 = 76;
const FLEE_COOLDOWN: Duration = Duration::from_millis(420);

#[cfg(windows)]
fn cursor_physical() -> Option<PhysicalPosition<i32>> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut pt = POINT::default();
    if unsafe { GetCursorPos(&mut pt) }.is_err() {
        return None;
    }
    Some(PhysicalPosition::new(pt.x, pt.y))
}

#[cfg(not(windows))]
fn cursor_physical() -> Option<PhysicalPosition<i32>> {
    None
}

fn corner_xy(wx: i32, wy: i32, ww: i32, wh: i32, win_w: i32, win_h: i32, corner: u8) -> (i32, i32) {
    let c = corner % 4;
    match c {
        0 => (wx + EDGE_MARGIN, wy + EDGE_MARGIN),
        1 => (wx + ww - win_w - EDGE_MARGIN, wy + EDGE_MARGIN),
        2 => (wx + EDGE_MARGIN, wy + wh - win_h - EDGE_MARGIN),
        _ => (wx + ww - win_w - EDGE_MARGIN, wy + wh - win_h - EDGE_MARGIN),
    }
}

pub fn place_window_at_corner(window: &WebviewWindow, corner: u8) -> tauri::Result<()> {
    let Some(m) = window.current_monitor().ok().flatten() else {
        return Ok(());
    };
    let wa = m.work_area();
    let wx = wa.position.x;
    let wy = wa.position.y;
    let ww = wa.size.width as i32;
    let wh = wa.size.height as i32;
    let size = window.outer_size()?;
    let w = size.width as i32;
    let h = size.height as i32;
    let c = corner % 4;
    let (x, y) = corner_xy(wx, wy, ww, wh, w, h, c);
    window.set_position(PhysicalPosition::new(x, y))?;
    CORNER_IDX.store(c, Ordering::Relaxed);
    Ok(())
}

/// 커서가 접근 띠에 있고 창 내부는 아니면 다음 모서리로 이동. 새 코너 인덱스(0–3) 또는 None.
pub fn tick_window_flee(app: &AppHandle) -> Option<u8> {
    let window = app.get_webview_window("main")?;
    let cursor = cursor_physical()?;
    let outer = window.outer_position().ok()?;
    let sz = window.outer_size().ok()?;

    let il = outer.x;
    let it = outer.y;
    let ir = outer.x + sz.width as i32;
    let ib = outer.y + sz.height as i32;

    let cx = cursor.x;
    let cy = cursor.y;

    let zl = il - APPROACH_PAD;
    let zt = it - APPROACH_PAD;
    let zr = ir + APPROACH_PAD;
    let zb = ib + APPROACH_PAD;

    let in_approach = cx >= zl && cx <= zr && cy >= zt && cy <= zb;
    let in_window = cx >= il && cx <= ir && cy >= it && cy <= ib;

    if !in_approach || in_window {
        return None;
    }

    let mut last = LAST_FLEE.lock().ok()?;
    let now = Instant::now();
    if let Some(t) = *last {
        if now.duration_since(t) < FLEE_COOLDOWN {
            return None;
        }
    }

    let next = (CORNER_IDX.load(Ordering::Relaxed) + 1) % 4;
    CORNER_IDX.store(next, Ordering::Relaxed);
    *last = Some(now);
    drop(last);

    let _ = place_window_at_corner(&window, next);
    Some(next)
}
