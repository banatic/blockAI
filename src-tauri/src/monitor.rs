use crate::MonitorSnapshot;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde::Deserialize;

const SCAN_INTERVAL_SECS: u64 = 2;
const FETCH_INTERVAL_SECS: u64 = 60; // Update keywords every 1 minute
const REMOTE_TOML_URL: &str = "https://raw.githubusercontent.com/banatic/blockAI/main/keyword.toml";

#[derive(Deserialize)]
struct KeywordConfig {
    keywords: Vec<String>,
}

#[cfg(windows)]
fn scan_once(state: &Arc<Mutex<MonitorSnapshot>>) {
    use crate::blocklist::{first_matching_keyword, title_matches_blocked};
    use crate::browser_kill::kill_chrome_and_edge;
    use crate::win_titles::visible_window_titles;

    let titles = visible_window_titles();
    let mut trigger: Option<(String, String)> = None;
    for t in &titles {
        if title_matches_blocked(t) {
            let kw = first_matching_keyword(t).unwrap_or_else(|| "?".to_string());
            trigger = Some((t.clone(), kw));
            break;
        }
    }

    let kill_needed = trigger.is_some();
    {
        let mut s = state.lock().unwrap();
        s.scans = s.scans.saturating_add(1);
        if let Some((ref window_title, ref keyword)) = trigger {
            s.last_matched_window_title = Some(window_title.clone());
            s.last_matched_keyword = Some(keyword.clone());
        }
    }
    if kill_needed {
        let killed = kill_chrome_and_edge();
        let mut s = state.lock().unwrap();
        s.last_kill_count = killed;
        s.total_kill_attempts = s.total_kill_attempts.saturating_add(killed as u64);
    }
}

#[cfg(not(windows))]
fn scan_once(state: &Arc<Mutex<MonitorSnapshot>>) {
    let mut s = state.lock().unwrap();
    s.scans = s.scans.saturating_add(1);
    s.last_error = Some("Windows Required: Monitoring and termination features are only available on Windows systems.".into());
}

fn fetch_remote_keywords() {
    use crate::blocklist::update_keywords;
    
    std::thread::spawn(|| loop {
        match reqwest::blocking::get(REMOTE_TOML_URL) {
            Ok(resp) => {
                if let Ok(text) = resp.text() {
                    if let Ok(config) = toml::from_str::<KeywordConfig>(&text) {
                        println!("Successfully updated keywords from remote: {} items", config.keywords.len());
                        // Convert all keywords to lowercase for efficient matching
                        let lower_keywords: Vec<String> = config.keywords.into_iter().map(|kw| kw.to_lowercase()).collect();
                        update_keywords(lower_keywords);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch remote keywords: {}", e);
            }
        }
        std::thread::sleep(Duration::from_secs(FETCH_INTERVAL_SECS));
    });
}

pub fn spawn_monitor(state: Arc<Mutex<MonitorSnapshot>>) {
    // Start background fetch
    fetch_remote_keywords();

    // Start monitor loop
    std::thread::spawn(move || loop {
        scan_once(&state);
        std::thread::sleep(Duration::from_secs(SCAN_INTERVAL_SECS));
    });
}
