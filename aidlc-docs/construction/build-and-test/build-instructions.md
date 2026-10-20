# Build instructions
Prerequisites: Rust stable, Node 20+, sing-box on PATH (the config tests run the real `sing-box check`; they skip without it), macOS.
```bash
npm ci
npm run check          # svelte-check
npm run build          # vite build
cd src-tauri && cargo clippy --all-targets -- -D warnings
npm run build:app      # full app bundle (src-tauri/target/release/bundle)
```

## Round 2 (per-app controls, branch feature/app-picker-and-usage-limits)
No new dependencies or environment variables. `npm run build:app` was run on this branch and produced `Kage VPN.app` (10.4 MiB). The Apps tab needs `/usr/bin/sips` (part of macOS) to make app icons; without it the tab shows each app's initial instead.
