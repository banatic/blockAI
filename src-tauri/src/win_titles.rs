use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
};

unsafe extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let out = &mut *(lparam.0 as *mut Vec<String>);
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL::from(true);
    }
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len <= 0 {
        return BOOL::from(true);
    }
    let mut buf = vec![0u16; len as usize + 1];
    let n = unsafe { GetWindowTextW(hwnd, &mut buf) };
    if n == 0 {
        return BOOL::from(true);
    }
    let title = String::from_utf16_lossy(&buf[..n as usize]);
    if !title.is_empty() {
        out.push(title);
    }
    BOOL::from(true)
}

/// 현재 사용자 세션에서 보이는 최상위 창의 제목 목록(빈 제목 제외).
pub fn visible_window_titles() -> Vec<String> {
    let mut titles = Vec::new();
    unsafe {
        let ptr = &mut titles as *mut Vec<String>;
        let _ = EnumWindows(Some(enum_window), LPARAM(ptr as isize));
    }
    titles
}
