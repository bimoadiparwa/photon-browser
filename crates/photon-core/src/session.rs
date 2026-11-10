use crate::{ProfileId, TabId};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TabState {
    Idle,
    Loading { url: String },
    Loaded { url: String, title: String },
    Failed { url: String, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tab {
    pub id: TabId,
    pub profile_id: ProfileId,
    pub url: String,
    pub title: String,
    pub state: TabState,
    pub pinned: bool,
    pub audible: bool,
    pub muted: bool,
}

impl Tab {
    pub fn new(profile_id: ProfileId, url: impl Into<String>) -> Self {
        let url = url.into();
        Self {
            id: TabId::next(),
            profile_id,
            title: String::new(),
            state: TabState::Idle,
            url,
            pinned: false,
            audible: false,
            muted: false,
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.state, TabState::Loading { .. })
    }
}
