// 전역 키보드 훅: 교사가 어떤 창에서든 비밀번호를 타이핑하면 앱 즉시 종료.
// 비밀번호는 GitHub keyword.toml의 teacher_password 필드에서 로드됨.
use once_cell::sync::Lazy;
use std::sync::Mutex;

const MAX_BUFFER: usize = 64;

static KEY_BUFFER: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static EXIT_CODE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

pub fn set_exit_code(code: String) {
    if let Ok(mut c) = EXIT_CODE.lock() {
        *c = code.to_uppercase();
    }
}

pub fn spawn_keyboard_watcher() {
    std::thread::spawn(|| unsafe {
        use windows::Win32::Foundation::HINSTANCE;
        use windows::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
            WH_KEYBOARD_LL, MSG,
        };

        let hook = match SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_hook_proc),
            HINSTANCE::default(),
            0,
        ) {
            Ok(h) => h,
            Err(_) => return,
        };

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            DispatchMessageW(&msg);
        }

        let _ = UnhookWindowsHookEx(hook);
    });
}

unsafe extern "system" fn keyboard_hook_proc(
    ncode: i32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::{CallNextHookEx, KBDLLHOOKSTRUCT, HHOOK};

    // ncode >= 0 이고 WM_KEYDOWN(0x0100) 일 때만 처리
    if ncode >= 0 && wparam.0 as u32 == 0x0100 {
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        if let Some(ch) = vk_to_char(kb.vkCode) {
            // try_lock: 훅 콜백 내에서 블로킹 방지
            if let Ok(mut buf) = KEY_BUFFER.try_lock() {
                buf.push(ch);
                if buf.len() > MAX_BUFFER {
                    let excess = buf.len() - MAX_BUFFER;
                    buf.drain(..excess);
                }
                if let Ok(code) = EXIT_CODE.try_lock() {
                    if !code.is_empty() && buf.ends_with(code.as_str()) {
                        std::process::exit(0);
                    }
                }
            }
        }
    }

    CallNextHookEx(HHOOK::default(), ncode, wparam, lparam)
}

// A-Z (0x41–0x5A), 0-9 (0x30–0x39) 만 감시
fn vk_to_char(vk: u32) -> Option<char> {
    match vk {
        0x41..=0x5A => Some((b'A' + (vk - 0x41) as u8) as char),
        0x30..=0x39 => Some((b'0' + (vk - 0x30) as u8) as char),
        _ => None,
    }
}
