#![forbid(unsafe_code)]
//! Photon core: domain types + navigation policy. Mirrors Zephium's
//! `zephium-core` ownership: domain authority in Rust, no UI, no I/O.

mod ids;
mod navigation;
mod ports;
mod profiles;
mod session;
mod work;

pub use ids::{PageId, ProfileId, SpaceId, TabId};
pub use navigation::{navigation_decision, NavigationDecision, NavigationInput};
pub use ports::{BlockerPort, EnginePort, StorePort};
pub use profiles::{Profile, ProfileKind};
pub use session::{Tab, TabState};
pub use work::{WorkOutcome, WorkSession};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AppState {
    pub profiles: Vec<Profile>,
    pub active_profile: ProfileId,
    pub tabs: Vec<Tab>,
    pub active_tab: Option<TabId>,
}

impl AppState {
    pub fn new() -> Self {
        let profile = Profile::new("default", ProfileKind::Normal);
        let active_profile = profile.id;
        Self {
            profiles: vec![profile],
            active_profile,
            tabs: Vec::new(),
            active_tab: None,
        }
    }

    pub fn tab(&self, id: TabId) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == id)
    }

    pub fn tab_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
