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
- The SSH key for the real `sing-box check` is made by `ssh-keygen` when the test runs (the test is skipped without it); earlier commits kept one in `tests/fixtures/test_ssh_key`, removed after GitGuardian flagged it.
- Scripts: `format.test.mjs` (import notes in the toast, WireGuard file detection).
- Expected: `cargo test --lib -- --test-threads=4` 409 passed; `npm run test:scripts` 39 passed.
