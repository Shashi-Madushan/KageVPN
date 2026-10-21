# U2 Per-app caps: Code Generation Plan
Covers FR2.1 to FR2.7. Design: `construction/u2-per-app-caps/functional-design/` (domain entities, 15 business rules, logic model, frontend components). U2 is the backend plus the pieces the UI needs to talk to it; the cap editor, banner and store are U3. This plan is the single source of truth for U2.

## Where it plugs in (from the code)
- The connection poll is `usage/collector.rs::spawn_connection_poll` (one task per tunnel pid, every 2 seconds). It returns early when recording is off, so the cap tick must run before that skip (rollover) and count only inside the recording branch.
- Usage settings live in `settings.rs::UsageSettings` and `store.rs::usage_settings()`; the usage database is `usage/store.rs` (`Db`, tables created in `SCHEMA`).
- Reconnecting: `session::disconnect_for_restart` then `session::connect`; `session::connect` is also the place to run rollover before the config is built.
- `commands::set_routing` must keep stored cap rules (business rule 13). `config::validate_rules` already refuses blocking Kage or the engine.
- Notifications: `notify::send`. Events: `app.emit(..)` (as `downloads.rs` does).

## Steps
- [x] 1. **Settings** (`settings.rs`): `CapPeriod { Day, Week, Month }`, `AtLimit { Notify, OfferBlock }`, `AppCap { path, limit_bytes, period, at_limit }`; `UsageSettings.app_caps: Vec<AppCap>` (defaults empty; old files load). Test: old JSON loads with no caps; a round trip keeps them.
- [x] 2. **Pure cap logic** (new `usage/caps.rs`, no I/O): `period_start(now, period)` (local midnight; Monday; the 1st), `period_end`, `level(used, limit)` (0, 80, 100), `next_alert(level, alert_level)`, `validate_cap(cap)` (1 MB to 10 TB, not Kage or the engine, path ends `.app`), `apply_cap_rule(rules, path)` and `lift_cap_rule(rules, path)` (add the cap rule on top / remove it, leaving the user's own rules), `keep_cap_rules(stored, incoming)` for `set_routing`. Tests for each, including week boundaries, a month with 31 days and DST-length days, level edges (79.99, 80, 100), and the add and lift rule behaviour.
- [x] 3. **Tally table** (`usage/store.rs`): `app_cap_tally(path PRIMARY KEY, period_start, up, down, alert_level, offer)`; `Db` methods `tally(path)`, `save_tally`, `delete_tally`, `all_tallies`. Not touched by `clear_all` or `clear_details`. Tests with an in-memory database: create, add bytes, rollover reset, delete, survives `clear_all`.
- [x] 4. **Counting and alerts** (`usage/caps.rs`, `collector.rs`): `tick(app, connections, counting)` called from the poll: rollover every cap whose period changed, add each capped app's byte change (only when `counting`), raise 80 and 100 notifications once per period, set `offer = pending` at 100 for `OfferBlock`, emit `caps-changed` on any change. The byte change reuses the poll's per-connection delta so nothing is counted twice. Test the pure part (`accumulate(tally, caps, deltas)`) with several apps, two periods and a rollover.
- [x] 5. **Commands** (`usage/caps.rs`, `commands`, `lib.rs`): `caps_list`, `cap_set`, `cap_remove`, `cap_block`, `cap_not_now`, `cap_unblock`. Block now and Unblock edit the stored routing through the store (cap rules via step 2), then call a new `session::reapply(app)` that reconnects once when the VPN is on. Rollover lifting uses the same path with the "can use the VPN again" notification. Each command checks the preconditions in the business rules (for example Block now needs `offer = pending`).
- [x] 6. **Keep cap rules on Save** (`commands.rs::set_routing`): replace the incoming cap-origin rules with the stored ones before saving. Test: an incoming list without the cap rule keeps it; an incoming cap rule that was not stored is dropped.
- [x] 7. **Start and connect hooks** (`session.rs`, `lib.rs`): run rollover for all caps at app start and at the top of `session::connect` before the config is built (no extra reconnect there). Test the helper that decides which cap rules to lift for a given time.
- [x] 8. **Types and API** (`src/lib/api.ts`): `AppCap`, `CapStatus`, `capsApi` with the commands above; `UsageSettings.appCaps`. No UI beyond what U1 already shows (the U1 Apps tab already renders cap blocks read-only and disables their switch).
- [x] 9. **Docs** (changelog and guide deferred to U3, when the feature has a UI): changelog entry, short note in the user guide's data usage section, `aidlc-docs/construction/u2-per-app-caps/code/summary.md` with the manual checks (set a 1 MB cap, download something, see the 80 and 100 notifications, Block now reconnects, rollover lifts the block, Clear details leaves the tally).
- [x] 10. **Verify**: `cargo test --lib -- --test-threads=4`, `cargo clippy --all-targets -- -D warnings`, `svelte-check`, `vite build`, `npm run test:scripts`.

## Interfaces added
Tauri commands `caps_list`, `cap_set`, `cap_remove`, `cap_block`, `cap_not_now`, `cap_unblock`; event `caps-changed`; `UsageSettings.appCaps`; table `app_cap_tally`.

## Out of scope for U2
The cap editor, Home banner and caps store in the UI (U3); popover alerts and the 90 percent plan alerts (U4).
