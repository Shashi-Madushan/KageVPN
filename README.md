# SingBox GUI

Minimal macOS app (Tauri 2 + Svelte) that manages [sing-box](https://sing-box.sagernet.org): import a `vless://` link, connect in TUN or proxy mode, view logs, test latency. See [PRD.md](PRD.md) for v1 and [PRD-v2.md](PRD-v2.md) for the v2 plan.

## Requirements
- `brew install sing-box`
- Node 20+, Rust stable, Xcode command line tools

## Develop
```bash
npm install
npm run tauri dev
```

## Test
```bash
cd src-tauri && cargo test
npm run check
```

## Build
```bash
npm run tauri build
```

## Layout
- `src-tauri/src/profile` VLESS link parsing
- `src-tauri/src/config.rs` sing-box config generation
- `src-tauri/src/runner` process supervision (TUN uses one admin prompt)
- `src-tauri/src/session.rs` connect/disconnect state machine
- `src-tauri/src/logs.rs` log tailing and error hints
- `src/lib` Svelte state (`app.svelte.ts`), API wrappers, components
