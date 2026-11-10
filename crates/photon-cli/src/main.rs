//! Photon CLI: headless driver over the same app actor as the desktop shell.
//! Usage: photon-cli <url> [--private] [--title TITLE] [--history QUERY]
//! Exercises core policy, blocker, engine, and store without native windows.

use photon_app::{App, Intent};
use photon_blocker::AdblockFilter;
use photon_core::StorePort;
use photon_engine::HeadlessEngine;
use photon_store::SqliteStore;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut url: Option<String> = None;
    let mut private = false;
    let mut title: Option<String> = None;
    let mut history_query: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--private" => private = true,
            "--title" => {
                i += 1;
                title = args.get(i).cloned();
            }
            "--history" => {
                i += 1;
                history_query = args.get(i).cloned();
            }
            other if !other.starts_with("--") => url = Some(other.to_string()),
            _ => eprintln!("unknown flag: {}", args[i]),
        }
        i += 1;
    }

    let mut app = App::new(
        HeadlessEngine::default(),
        SqliteStore::open_in_memory().unwrap(),
        AdblockFilter::with_seed(),
    );

    if let Some(query) = history_query {
        let mut store = SqliteStore::open_in_memory().unwrap();
        let _ = store.record_visit("https://example.com", "Example", false);
        let hits = store.search_history(&query, 10).unwrap();
        println!("history matches for {query:?}: {}", hits.len());
        for (u, t) in hits {
            println!("  {u} — {t}");
        }
        return;
    }

    let Some(url) = url else {
        eprintln!("usage: photon-cli <url> [--private] [--title TITLE]");
        std::process::exit(2);
    };

    let intent = if private {
        Intent::OpenPrivateTab { url }
    } else {
        Intent::OpenTab { url }
    };
    if let Err(e) = app.handle(intent) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }

    let state = app.state();
    println!("tabs: {}", state.tabs.len());
    for tab in &state.tabs {
        println!("  {} {} {:?}", tab.id, tab.url, tab.state);
    }
    println!("active tab: {:?}", state.active_tab);
    println!(
        "blocker: doubleclick.net blocked={}",
        app.should_block("https://doubleclick.net/x", "https://example.com")
    );
    if let Some(t) = title {
        let tab = state.tabs[0].id;
        let _ = app.handle(Intent::RecordVisit {
            url: state.tabs[0].url.clone(),
            title: t,
        });
        println!("recorded visit for tab {tab:?}");
    }
}
