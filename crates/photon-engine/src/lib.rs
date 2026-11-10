//! Photon engine: native browser views behind a port.
//! Mirrors Zephium's `zephium-engine`: Rust owns policy + view lifetimes;
//! the system web engine (WebKitGTK on Linux via `wry`, WKWebView on macOS,
//! WebView2 on Windows) supplies renderer/network processes. A logical tab
//! is application state, not a live WebView.

use photon_core::EnginePort;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("native: {0}")]
    Native(String),
}

/// Headless engine: records tabs without native views. Used for tests,
/// the CLI, and hosts where no native window exists.
#[derive(Debug, Default)]
pub struct HeadlessEngine {
    tabs: HashMap<u64, String>,
    titles: HashMap<u64, String>,
}

impl HeadlessEngine {
    pub fn tabs(&self) -> &HashMap<u64, String> {
        &self.tabs
    }

    pub fn set_title(&mut self, tab_id: u64, title: impl Into<String>) {
        self.titles.insert(tab_id, title.into());
    }
}

fn validate_url(url: &str) -> Result<(), EngineError> {
    if url.starts_with("about:") || url.starts_with("photon:") {
        return Ok(());
    }
    url::Url::parse(url).map_err(|e| EngineError::InvalidUrl(e.to_string()))?;
    Ok(())
}

impl EnginePort for HeadlessEngine {
    type Error = EngineError;

    fn open_tab(&mut self, tab_id: u64, url: &str) -> Result<(), EngineError> {
        validate_url(url)?;
        self.tabs.insert(tab_id, url.to_string());
        Ok(())
    }

    fn navigate(&mut self, tab_id: u64, url: &str) -> Result<(), EngineError> {
        validate_url(url)?;
        if !self.tabs.contains_key(&tab_id) {
            return Err(EngineError::Native(format!("unknown tab {tab_id}")));
        }
        self.tabs.insert(tab_id, url.to_string());
        Ok(())
    }

    fn close_tab(&mut self, tab_id: u64) -> Result<(), EngineError> {
        self.tabs.remove(&tab_id);
        self.titles.remove(&tab_id);
        Ok(())
    }

    fn title_of(&self, tab_id: u64) -> Option<String> {
        self.titles.get(&tab_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_lifecycle() {
        let mut e = HeadlessEngine::default();
        e.open_tab(1, "https://example.com").unwrap();
        e.navigate(1, "https://example.com/2").unwrap();
        assert_eq!(e.tabs().get(&1).unwrap(), "https://example.com/2");
        e.close_tab(1).unwrap();
        assert!(e.tabs().is_empty());
    }

    #[test]
    fn rejects_garbage_urls() {
        let mut e = HeadlessEngine::default();
        assert!(e.open_tab(1, "not a url").is_err());
    }
}
