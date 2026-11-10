//! Photon app: actor shell around photon-core.
//! Mirrors Zephium's `zephium-app`: UI intents enter a queue; the actor owns
//! state, applies domain transitions, invokes ports, publishes projections.
//! Blocking I/O belongs in adapters/workers, never in a reducer.

use photon_core::{
    navigation_decision, AppState, BlockerPort, EnginePort, NavigationDecision, NavigationInput,
    ProfileKind, StorePort, Tab, TabId, TabState,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("navigation denied: {0}")]
    NavigationDenied(&'static str),
    #[error("unknown tab {0:?}")]
    UnknownTab(TabId),
    #[error("no profile")]
    NoProfile,
    #[error("engine: {0}")]
    Engine(String),
    #[error("store: {0}")]
    Store(String),
}

pub enum Intent {
    OpenTab { url: String },
    Navigate { tab: TabId, url: String },
    CloseTab(TabId),
    ActivateTab(TabId),
    RecordVisit { url: String, title: String },
    OpenPrivateTab { url: String },
    NewPrivateProfile { name: String },
}

pub struct App<E, S, B>
where
    E: EnginePort,
    S: StorePort,
    B: BlockerPort,
{
    state: AppState,
    engine: E,
    store: S,
    blocker: B,
}

impl<E, S, B> App<E, S, B>
where
    E: EnginePort,
    S: StorePort,
    B: BlockerPort,
{
    pub fn new(engine: E, store: S, blocker: B) -> Self {
        Self {
            state: AppState::new(),
            engine,
            store,
            blocker,
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn handle(&mut self, intent: Intent) -> Result<(), AppError> {
        match intent {
            Intent::OpenTab { url } => self.open_tab(url, false),
            Intent::OpenPrivateTab { url } => self.open_tab(url, true),
            Intent::Navigate { tab, url } => self.navigate(tab, url),
            Intent::CloseTab(tab) => self.close_tab(tab),
            Intent::ActivateTab(tab) => {
                if self.state.tab(tab).is_some() {
                    self.state.active_tab = Some(tab);
                    Ok(())
                } else {
                    Err(AppError::UnknownTab(tab))
                }
            }
            Intent::RecordVisit { url, title } => {
                let is_private = self
                    .state
                    .profiles
                    .iter()
                    .find(|p| p.id == self.state.active_profile)
                    .map(|p| p.is_private())
                    .unwrap_or(false);
                self.store
                    .record_visit(&url, &title, is_private)
                    .map_err(|e| AppError::Store(e.to_string()))
            }
            Intent::NewPrivateProfile { name } => {
                let profile = photon_core::Profile::new(name, ProfileKind::Private);
                self.state.active_profile = profile.id;
                self.state.profiles.push(profile);
                Ok(())
            }
        }
    }

    fn is_private(&self) -> bool {
        self.state
            .profiles
            .iter()
            .find(|p| p.id == self.state.active_profile)
            .map(|p| p.is_private())
            .unwrap_or(false)
    }

    fn open_tab(&mut self, url: String, private: bool) -> Result<(), AppError> {
        let decision = navigation_decision(&NavigationInput {
            url: &url,
            is_private: private,
            is_user_gesture: true,
        });
        if let NavigationDecision::Deny { reason } = decision {
            return Err(AppError::NavigationDenied(reason));
        }
        let mut tab = Tab::new(self.state.active_profile, url.clone());
        tab.state = TabState::Loading { url: url.clone() };
        let id = tab.id;
        self.state.tabs.push(tab);
        self.state.active_tab = Some(id);
        self.engine
            .open_tab(id.get(), &url)
            .map_err(|e| AppError::Engine(e.to_string()))?;
        Ok(())
    }

    fn navigate(&mut self, tab_id: TabId, url: String) -> Result<(), AppError> {
        let decision = navigation_decision(&NavigationInput {
            url: &url,
            is_private: self.is_private(),
            is_user_gesture: true,
        });
        if let NavigationDecision::Deny { reason } = decision {
            return Err(AppError::NavigationDenied(reason));
        }
        let tab = self
            .state
            .tab_mut(tab_id)
            .ok_or(AppError::UnknownTab(tab_id))?;
        tab.url = url.clone();
        tab.state = TabState::Loading { url: url.clone() };
        let _ = tab;
        self.engine
            .navigate(tab_id.get(), &url)
            .map_err(|e| AppError::Engine(e.to_string()))
    }

    fn close_tab(&mut self, tab_id: TabId) -> Result<(), AppError> {
        let pos = self
            .state
            .tabs
            .iter()
            .position(|t| t.id == tab_id)
            .ok_or(AppError::UnknownTab(tab_id))?;
        self.state.tabs.remove(pos);
        if self.state.active_tab == Some(tab_id) {
            self.state.active_tab = self.state.tabs.last().map(|t| t.id);
        }
        self.engine
            .close_tab(tab_id.get())
            .map_err(|e| AppError::Engine(e.to_string()))
    }

    /// Ask the blocker whether a subresource should be cancelled.
    /// Pure policy query: safe to call from engine callbacks.
    pub fn should_block(&self, url: &str, page_url: &str) -> bool {
        self.blocker.should_block(url, page_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_blocker::AdblockFilter;
    use photon_engine::HeadlessEngine;
    use photon_store::SqliteStore;

    type TestApp = App<HeadlessEngine, SqliteStore, AdblockFilter>;

    fn test_app() -> TestApp {
        App::new(
            HeadlessEngine::default(),
            SqliteStore::open_in_memory().unwrap(),
            AdblockFilter::with_seed(),
        )
    }

    #[test]
    fn open_and_navigate() {
        let mut app = test_app();
        app.handle(Intent::OpenTab {
            url: "https://example.com".into(),
        })
        .unwrap();
        assert_eq!(app.state().tabs.len(), 1);
        let tab = app.state().tabs[0].id;
        app.handle(Intent::Navigate {
            tab,
            url: "https://example.com/2".into(),
        })
        .unwrap();
        assert_eq!(app.state().tab(tab).unwrap().url, "https://example.com/2");
    }

    #[test]
    fn denies_blocked_schemes() {
        let mut app = test_app();
        let err = app
            .handle(Intent::OpenTab {
                url: "javascript:alert(1)".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::NavigationDenied("blocked-scheme")));
        assert!(app.state().tabs.is_empty());
    }

    #[test]
    fn close_removes_tab_and_reassigns_active() {
        let mut app = test_app();
        app.handle(Intent::OpenTab {
            url: "https://a.example".into(),
        })
        .unwrap();
        app.handle(Intent::OpenTab {
            url: "https://b.example".into(),
        })
        .unwrap();
        let first = app.state().tabs[0].id;
        app.handle(Intent::CloseTab(first)).unwrap();
        assert_eq!(app.state().tabs.len(), 1);
    }

    #[test]
    fn blocker_query() {
        let app = test_app();
        assert!(app.should_block("https://doubleclick.net/x", "https://example.com"));
        assert!(!app.should_block("https://example.com/x", "https://example.com"));
    }
}
