//! Navigation policy: the single predicate every navigation path consults.
//! Mirrors Zephium's `zephium-core::navigation`: domain decides, engine obeys.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NavigationDecision {
    Allow,
    Deny { reason: &'static str },
}

#[derive(Debug, Clone)]
pub struct NavigationInput<'a> {
    pub url: &'a str,
    pub is_private: bool,
    pub is_user_gesture: bool,
}

const BLOCKED_SCHEMES: &[&str] = &["javascript:", "file:", "vbscript:", "data:"];

pub fn navigation_decision(input: &NavigationInput<'_>) -> NavigationDecision {
    let lower = input.url.trim_start().to_ascii_lowercase();
    if lower.is_empty() {
        return NavigationDecision::Deny {
            reason: "empty-url",
        };
    }
    for scheme in BLOCKED_SCHEMES {
        if lower.starts_with(scheme) {
            return NavigationDecision::Deny {
                reason: "blocked-scheme",
            };
        }
    }
    if lower.starts_with("about:") || lower.starts_with("photon:") {
        return NavigationDecision::Allow;
    }
    match url::Url::parse(input.url) {
        Ok(parsed) => match parsed.scheme() {
            "http" | "https" => NavigationDecision::Allow,
            _ => NavigationDecision::Deny {
                reason: "unsupported-scheme",
            },
        },
        Err(_) => {
            // Bare host or search-ish input: treat as search, allow to engine.
            if input.url.contains('.') && !input.url.contains(char::is_whitespace) {
                NavigationDecision::Allow
            } else {
                NavigationDecision::Deny {
                    reason: "invalid-url",
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decide(url: &str) -> NavigationDecision {
        navigation_decision(&NavigationInput {
            url,
            is_private: false,
            is_user_gesture: true,
        })
    }

    #[test]
    fn allows_http_and_https() {
        assert_eq!(decide("https://example.com"), NavigationDecision::Allow);
        assert_eq!(
            decide("http://example.com/a?b=1"),
            NavigationDecision::Allow
        );
    }

    #[test]
    fn denies_script_and_local_schemes() {
        assert_eq!(
            decide("javascript:alert(1)"),
            NavigationDecision::Deny {
                reason: "blocked-scheme"
            }
        );
        assert_eq!(
            decide("file:///etc/passwd"),
            NavigationDecision::Deny {
                reason: "blocked-scheme"
            }
        );
    }

    #[test]
    fn internal_pages_allowed() {
        assert_eq!(decide("about:newtab"), NavigationDecision::Allow);
        assert_eq!(decide("photon:home"), NavigationDecision::Allow);
    }

    #[test]
    fn rejects_empty_and_garbage() {
        assert_eq!(
            decide(""),
            NavigationDecision::Deny {
                reason: "empty-url"
            }
        );
        assert_eq!(
            decide("not a url"),
            NavigationDecision::Deny {
                reason: "invalid-url"
            }
        );
    }
}
