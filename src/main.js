const { invoke } = window.__TAURI__.core;

/**
 * Updates an element's text content with a fallback for empty values.
 * @param {HTMLElement} el 
 * @param {string|number|null} v 
 */
function updateText(el, v) {
  if (!el) return;
  const content = (v == null || v === "") ? "—" : String(v);
  if (el.textContent !== content) {
    el.textContent = content;
  }
}

async function tickSnapshot() {
  try {
    const s = await invoke("get_monitor_snapshot");
    
    updateText(document.getElementById("scans"), s.scans);
    updateText(document.getElementById("keyword"), s.lastMatchedKeyword);
    updateText(document.getElementById("windowTitle"), s.lastMatchedWindowTitle);
    updateText(document.getElementById("lastKill"), s.lastKillCount ?? 0);
    updateText(document.getElementById("totalKill"), s.totalKillAttempts ?? 0);

    const warn = document.getElementById("warn");
    if (warn) {
      if (!s.windowsSupported) {
        warn.hidden = false;
        warn.textContent = s.lastError || "Windows 전용: 창 감시 및 브라우저 종료 기능은 Windows 시스템에서만 사용할 수 있습니다.";
      } else {
        warn.hidden = true;
      }
    }
  } catch (err) {
    console.error("Failed to fetch monitor snapshot:", err);
  }
}

async function tickFlee() {
  try {
    await invoke("tick_window_flee");
  } catch {
    /* ignore intentionally - window might be moving or closed */
  }
}

window.addEventListener("DOMContentLoaded", () => {
  // Initial sync
  tickSnapshot();
  tickFlee();
  
  // Real-time updates
  setInterval(tickSnapshot, 1000);
  setInterval(tickFlee, 90);
});
