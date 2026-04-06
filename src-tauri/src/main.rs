// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(windows)]
    let _guard = {
        use windows::core::PCWSTR;
        use windows::Win32::System::Threading::CreateMutexW;
        use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};

        // "BlockAI_Monotone_Guard" matches the user's "Monotone" request
        let name: Vec<u16> = "Global\\BlockAI_Monotone_Guard\0".encode_utf16().collect();
        let handle = unsafe { CreateMutexW(None, true, PCWSTR(name.as_ptr())).unwrap() };
        
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            // Already running
            return;
        }
        handle
    };

    blockai_lib::run();
}
