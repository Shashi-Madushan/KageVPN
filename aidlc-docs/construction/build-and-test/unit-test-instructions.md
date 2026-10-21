# Unit tests
```bash
cd src-tauri && cargo test --lib -- --test-threads=4
npm run test:scripts
```
New tests: core (rollback), repair, diagnostics_bundle (needs macOS `unzip`), networks, appnap, config (IPv6). Use `--test-threads=4`: with the default thread count the run stalled twice in this session, cause not found.

## Round 2 additions
- Rust: `usage::caps` (periods, levels, tallies, rule edits, status, tally table, settings merge), `config` (self-block refusal, rule origin), `apps` (icon file name and cache), `subscriptions` (7/3/1-day expiry, check without refresh). Use `cargo test --lib -- --test-threads=4`.
- Scripts: `scripts/tests/app-rules.test.mjs`, `cap-view.test.mjs`, `format.test.mjs` (Node runs the `.ts` modules directly).

## Round 4, pull request 1 additions
- Rust: `profile_tuic`, `profile_hysteria`, `profile_ssh`, `profile_wireguard` (each protocol's links, refusals with their messages, round trips, outbound or endpoint shape); the shared conformance suite now covers nine protocols, endpoints and the build-feature declarations; `profile_formats` (sing-box `endpoints`, old and new WireGuard forms, Clash entries for every new type); `profile_links` (import notes, `.conf` files named after the file); `config` (endpoints placement, engine-feature filtering, real `sing-box check` of an SSH server with a pasted key and of WireGuard in both modes); `health` (UDP protocols skip the TCP test).
- `src-tauri/src/tests/fixtures/test_ssh_key` is a throwaway key made for the tests; it opens nothing.
- Scripts: `format.test.mjs` (import notes in the toast, WireGuard file detection).
- Expected: `cargo test --lib -- --test-threads=4` 409 passed; `npm run test:scripts` 39 passed.

## Round 4, pull request 2 additions
- Rust: `sysproxy` (13 tests against a pretend macOS: parsing, bypass list, set then restore, a user change kept, rollback after a failure, new services and ports, a proxy with a login left alone, a failed restore kept for Repair, the restore script's quoting and guards, when it applies, old settings); `runner` (the supervisor runs the restore script only when a run ends without a stop request).
- One live test, ignored by default because it changes a real network service ("iPhone USB" only; it cannot see others): `cargo test --lib live_system_proxy_round_trip -- --ignored --nocapture`.
- Scripts: `format.test.mjs` (bypass entries from the Settings box).
- Expected: `cargo test --lib -- --test-threads=4` 424 passed, 1 ignored; `npm run test:scripts` 40 passed.
