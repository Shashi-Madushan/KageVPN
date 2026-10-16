# Build instructions
Prerequisites: Rust stable, Node 20+, sing-box on PATH (the config tests run the real `sing-box check`; they skip without it), macOS.
```bash
npm ci
npm run check          # svelte-check
npm run build          # vite build
cd src-tauri && cargo clippy --all-targets -- -D warnings
npm run build:app      # full app bundle (src-tauri/target/release/bundle)
```
