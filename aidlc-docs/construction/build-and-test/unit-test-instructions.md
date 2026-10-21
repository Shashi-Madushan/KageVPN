# Unit tests
```bash
cd src-tauri && cargo test --lib -- --test-threads=4
npm run test:scripts
```
New tests: core (rollback), repair, diagnostics_bundle (needs macOS `unzip`), networks, appnap, config (IPv6). Use `--test-threads=4`: with the default thread count the run stalled twice in this session, cause not found.

## Round 2 additions
- Rust: `usage::caps` (periods, levels, tallies, rule edits, status, tally table, settings merge), `config` (self-block refusal, rule origin), `apps` (icon file name and cache), `subscriptions` (7/3/1-day expiry, check without refresh). Use `cargo test --lib -- --test-threads=4`.
- Scripts: `scripts/tests/app-rules.test.mjs`, `cap-view.test.mjs`, `format.test.mjs` (Node runs the `.ts` modules directly).
