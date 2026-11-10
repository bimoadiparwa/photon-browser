//! Ports: the boundaries `photon-app` applies policy through.
//! Mirrors Zephium's `zephium-core::ports`.

/// Engine: owns native WebViews / tab view lifetimes.
pub trait EnginePort {
    type Error: std::fmt::Display;
    fn open_tab(&mut self, tab_id: u64, url: &str) -> Result<(), Self::Error>;
    fn navigate(&mut self, tab_id: u64, url: &str) -> Result<(), Self::Error>;
    fn close_tab(&mut self, tab_id: u64) -> Result<(), Self::Error>;
    fn title_of(&self, tab_id: u64) -> Option<String>;
}

/// Store: owns durable records (history, sessions, settings).
pub trait StorePort {
    type Error: std::fmt::Display;
    fn record_visit(&mut self, url: &str, title: &str, is_private: bool)
        -> Result<(), Self::Error>;
    fn search_history(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(String, String)>, Self::Error>;
    fn save_session(&mut self, snapshot_json: &str) -> Result<(), Self::Error>;
    fn load_session(&self) -> Result<Option<String>, Self::Error>;
}

/// Blocker: owns content protection decisions.
pub trait BlockerPort {
    fn should_block(&self, url: &str, page_url: &str) -> bool;
}
