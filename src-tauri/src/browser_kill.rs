use sysinfo::{ProcessesToUpdate, System};

/// 프로세스 이름이 `chrome.exe`, `msedge.exe`인 항목을 종료 시도합니다.
/// 반환값은 `kill()`에 성공한 프로세스 개수(대략적인 지표)입니다.
pub fn kill_chrome_and_edge() -> u32 {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut n = 0u32;
    for (_pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        if name == "chrome.exe" || name == "msedge.exe" {
            if process.kill() {
                n += 1;
            }
        }
    }
    n
}
