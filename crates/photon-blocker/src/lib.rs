//! Photon blocker: native content protection.
//! Mirrors Zephium's `zephium-blocker`: Brave `adblock-rust` (EasyList /
//! EasyPrivacy / uBlock filters), requests stopped before a page can make them.

use adblock::request::Request;
use adblock::{Engine, FilterSet};
use photon_core::BlockerPort;

/// Small built-in seed: catches the obvious tracker/ad hosts without a
/// network fetch. Full EasyList/EasyPrivacy/uBlock lists can be loaded
/// via `from_filter_list` at startup.
pub const SEED_RULES: &[&str] = &[
    "||doubleclick.net^",
    "||googlesyndication.com^",
    "||googleadservices.com^",
    "||analytics.google.com^",
    "||facebook.net/tr/",
    "||hotjar.com^",
    "||mixpanel.com^",
    "||segment.com^",
    "||adsrvr.org^",
    "||scorecardresearch.com^",
];

pub struct AdblockFilter {
    engine: Engine,
}

impl AdblockFilter {
    pub fn with_seed() -> Self {
        let mut set = FilterSet::new(false);
        set.add_filters(SEED_RULES.iter().copied(), Default::default());
        Self {
            engine: Engine::from_filter_set(set, true),
        }
    }

    pub fn from_filter_list(list: &str) -> Self {
        let mut set = FilterSet::new(false);
        set.add_filter_list(list, Default::default());
        Self {
            engine: Engine::from_filter_set(set, true),
        }
    }
}

impl BlockerPort for AdblockFilter {
    fn should_block(&self, url: &str, page_url: &str) -> bool {
        let source = if page_url.is_empty() {
            "https://photon.invalid"
        } else {
            page_url
        };
        let Ok(request) = Request::new(url, source, "other") else {
            return false;
        };
        self.engine.check_network_request(&request).matched
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_blocks_trackers() {
        let f = AdblockFilter::with_seed();
        assert!(f.should_block("https://www.doubleclick.net/ads/x", "https://example.com"));
        assert!(f.should_block(
            "https://www.googlesyndication.com/ad.js",
            "https://example.com"
        ));
    }

    #[test]
    fn seed_allows_first_party() {
        let f = AdblockFilter::with_seed();
        assert!(!f.should_block("https://example.com/app.js", "https://example.com"));
        assert!(!f.should_block("https://cdn.example.com/style.css", "https://example.com"));
    }
}
