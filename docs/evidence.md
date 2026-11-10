# Photon — How it works, with Zephium evidence

Photon is a **similar application** to [Zephium](https://github.com/zephium-browser/Zephium)
(MPL-2.0, Rust, 1018 stars at time of writing), built for this repo.
Zephium targets macOS/Windows only; Photon runs wherever Rust + a system
web engine exist, and is verified here on **Linux x86_64**.

## Evidence: what Zephium actually is

| Claim | Evidence |
|---|---|
| Native Rust browser, OS web engine (WebKit macOS, WebView2 Windows), not Chromium/Firefox fork | [README](https://github.com/zephium-browser/Zephium) + [docs/architecture.md](https://github.com/zephium-browser/Zephium/blob/main/docs/architecture.md) §3 |
| Workspace: `crates/*` + `desktop` + `xtask`, pinned vendored Tauri/Wry | [Cargo.toml](https://github.com/zephium-browser/Zephium/blob/main/Cargo.toml) |
| Domain authority in `zephium-core` (types/reducers/ports, `#![forbid(unsafe_code)]`), actor in `zephium-app`, SQLite in `zephium-store`, engine in `zephium-engine` | [architecture.md §1–2](https://github.com/zephium-browser/Zephium/blob/main/docs/architecture.md) |
| Brave `adblock-rust` blocker, EasyList/EasyPrivacy/uBlock filters | README "Fast and light" + `assets/blocker-seed/v1/` |
| Profiles with private/non-private boundary; private state excluded from durable history/favicon storage | architecture.md §5 |
| Work: agent canvas where helpers browse live pages, results stay as artifacts, changes wait for OK | README "Work" + [docs/work.md](https://github.com/zephium-browser/Zephium/blob/main/docs/work.md) |
| Bring your own model keys (Anthropic/OpenAI/Gemini/…/Ollama); keys in system keychain | README "Work" |
| Linux unsupported | [AGENTS.md](https://github.com/zephium-browser/Zephium/blob/main/AGENTS.md): "Linux is not supported" |

## How Photon works (same shape, Linux-portable)

```
┌─────────────┐   typed Intent    ┌──────────┐  port calls   ┌────────────┐
│ photon-cli /│ ────────────────► │ photon-  │ ────────────► │ photon-    │
│ photon (wry)│ ◄──────────────── │ app      │               │ engine     │
│             │   projection      │ (actor)  │ ────────────► │ (Headless /│
│             │                   │          │               │  wry view) │
│             │                   │  owns    │ ────────────► │ photon-    │
│             │                   │  state,  │               │ store      │
│             │                   │  policy  │ ────────────► │ (SQLite)   │
└─────────────┘                   └────┬─────┘               │ photon-    │
                                       │ pure policy         │ blocker    │
                                       ▼                     └────────────┘
                              ┌──────────────────┐
                              │ photon-core      │
                              │ types · reducers │
                              │ nav policy ·     │
                              │ ports · profiles │
                              │ Work canvas      │
                              └──────────────────┘
```

1. **Core first** (`photon-core`): `Tab`, `Profile`, `AppState`, `WorkSession`,
   and the single `navigation_decision` predicate. No I/O, no unsafe.
2. **Actor** (`photon-app`): intents (`OpenTab`, `Navigate`, `CloseTab`, …)
   are applied to state; the actor consults nav policy, then drives
   `EnginePort` / `StorePort` / `BlockerPort`. `Deferred`-style calls return
   after the engine accepts, never claim durable success early.
3. **Engine** (`photon-engine` + `desktop/`): `HeadlessEngine` records tabs
   for tests/CLI. The desktop shell builds real WebViews with `wry 0.55` on
   `tao` — WebKitGTK on Linux, WKWebView on macOS, WebView2 on Windows.
   Remote pages are raw WebViews; chrome/overlays are Rust-owned.
4. **Store** (`photon-store`): SQLite via `rusqlite`, forward migrations
   in `migrate()`. Private visits are refused before they reach the DB.
5. **Blocker** (`photon-blocker`): Brave `adblock-rust` (same crate Zephium
   uses) with a built-in seed; `from_filter_list` loads full EasyList text.
6. **Work** (`photon-core::work`): `WorkSession` carries prompt + source tab
   and accumulates artifacts; `finish` returns a `WorkOutcome` that lives
   on the canvas, not a chat scroll.

## Differences from Zephium (deliberate)

| Zephium | Photon | Why |
|---|---|---|
| Tauri 2 + Svelte frame, generated TS bindings | `tao` + `wry` directly, Rust projections | No native dev headers here; same Wry layer Tauri wraps |
| Vendored Tauri/Wry forks | Upstream `wry =0.55.1`, `tao 0.35` | Reproducible without fork maintenance |
| 25 agent skills, MCP, multi-provider runtime | `WorkSession` seam only | Model/provider runtime is a later slice; policy + artifacts are the contract |
| macOS/Windows only | Linux-verified, macOS/Windows via same wry build | This host is Linux x86_64 |

## Build & run

```sh
cargo build --workspace        # library crates + headless CLI
cargo test --workspace         # nav policy, store, blocker, actor, engine
cargo run -p photon-cli -- https://example.com
cargo run -p photon-cli -- https://example.com --private
cargo run -p photon-desktop    # native window (needs webkit2gtk dev headers on Linux)
```

Native desktop needs system WebKitGTK dev packages on Linux:
`libwebkit2gtk-4.0-dev libgtk-3-dev libsoup-3.0-dev` (absent in this
container, so `cargo check` of `photon-desktop` fails on `webkit2gtk-sys`
until installed — the library crates build and test green without it).
