# Technology Stack
Rust 2021 (Tauri 2, tokio, reqwest, rusqlite, serde, qrcode, chrono); TypeScript + Svelte 5 / SvelteKit + Vite; shell helpers run as root; sing-box >= 1.12 (1.14.2 locally); CI in .github/workflows (cargo clippy -D warnings, cargo test, svelte-check, build); docs-site for documentation. Tests: ~282 cargo tests including real `sing-box check`.
