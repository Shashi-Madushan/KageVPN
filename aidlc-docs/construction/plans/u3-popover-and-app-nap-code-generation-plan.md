# U3 Popover switcher and App Nap: Code Generation Plan
Covers FR5 and FR6. No functional design stage (skipped in the execution plan).

## Finding that changes the scope
The menu-bar popover already has a server picker (`routes/tray/+page.svelte`, "Switch server without disconnecting" while connected, and a plain picker while disconnected, including an Auto option). So FR6 reduces to the missing piece from your answer (name **and latency**): show each server's measured latency in the list.

## Steps
- [x] 1. **App Nap, backend** (`appnap.rs`, `Cargo.toml`): add `objc2-foundation` (already in Cargo.lock as 0.3.2 via tauri, no new download) with `NSProcessInfo` and `NSString`; `begin()` takes `beginActivityWithOptions:reason:` with `NSActivityUserInitiatedAllowingIdleSystemSleep` (the Mac may still sleep; only App Nap throttling is prevented); `end()` releases it; idempotent; the activity object is kept in a `Mutex` behind a small `Send` wrapper.
- [x] 2. **Hook** (`session.rs`): `begin()` once connected, `end()` when the connection ends (all endings) and on app quit.
- [x] 3. **Tests** (`tests/appnap.rs`): begin twice holds one activity; end without begin is harmless; held flag toggles.
- [x] 4. **Popover latency** (`routes/tray/+page.svelte`): server option labels get "· 42 ms" when a latency is known; picker order unchanged.
- [x] 5. **Docs**: changelog; `aidlc-docs/construction/u3-popover-and-app-nap/code/summary.md`.
- [ ] 6. **Verify**: `cargo test` (4 threads), `cargo clippy -D warnings`, `svelte-check`.
