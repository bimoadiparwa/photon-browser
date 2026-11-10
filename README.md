# photon-browser

A fast native browser + agent work canvas, modeled on
[Zephium](https://github.com/zephium-browser/Zephium) (Rust, OS web engine,
actor core, SQLite store, Brave adblock-rust blocker) and built for Linux.

See [docs/evidence.md](docs/evidence.md) for how it works and the Zephium
evidence behind each design choice.

## Layout

| Crate | Role |
|---|---|
| `crates/photon-core` | Domain types, nav policy, ports, profiles, Work canvas |
| `crates/photon-app` | Actor shell: applies intents, drives ports |
| `crates/photon-store` | SQLite history + session persistence |
| `crates/photon-blocker` | adblock-rust content filter |
| `crates/photon-engine` | Engine port + headless engine |
| `crates/photon-cli` | Headless CLI over the same actor |
| `desktop/` | Native shell: `tao` window + `wry` WebViews |

## Quick start

```sh
cargo test --workspace
cargo run -p photon-cli -- https://example.com
cargo run -p photon-desktop   # needs WebKitGTK dev headers on Linux
```
