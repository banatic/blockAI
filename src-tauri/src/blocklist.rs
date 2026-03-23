use once_cell::sync::Lazy;
use std::sync::RwLock;

// Default internal keywords for initial startup
const DEFAULT_KEYWORDS: &[&str] = &[
    "chatgpt", "openai", "claude", "anthropic", "gemini", "bard", 
    "copilot", "perplexity", "character.ai", "poe.com", "you.com", 
    "deepseek", "huggingface", "mistral",
];

static BLOCKED_KEYWORDS: Lazy<RwLock<Vec<String>>> = Lazy::new(|| {
    RwLock::new(DEFAULT_KEYWORDS.iter().map(|s| s.to_string()).collect())
});

/// Replaces the current blocked keywords with a new list.
pub fn update_keywords(new_list: Vec<String>) {
    if let Ok(mut keywords) = BLOCKED_KEYWORDS.write() {
        *keywords = new_list;
    }
}

/// Returns the first matching keyword from the given title, if any.
pub fn first_matching_keyword(title: &str) -> Option<String> {
    let lower_title = title.to_lowercase();
    let keywords = BLOCKED_KEYWORDS.read().ok()?;
    keywords
        .iter()
        .find(|&kw| lower_title.contains(&kw.to_lowercase()))
        .cloned()
}

/// Checks if the given title matches any blocked keyword.
pub fn title_matches_blocked(title: &str) -> bool {
    first_matching_keyword(title).is_some()
}
