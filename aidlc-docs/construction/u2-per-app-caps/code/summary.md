# U2 per-app caps: code summary
- `settings.rs`: `CapPeriod`, `AtLimit`, `AppCap`; `UsageSettings.app_caps` (defaults empty, old files load).
- `usage/caps.rs` (new): pure period, level, tally, validation and rule helpers; `tick` (counting, alerts, rollover) called from the poll; `rollover_all` at app start and before each connect; commands `caps_list`, `cap_set`, `cap_remove`, `cap_block`, `cap_not_now`, `cap_unblock`; event `caps-changed`.
- `usage/store.rs`: table `app_cap_tally` (untouched by Clear details and Clear all usage).
- `usage/collector.rs`: collects each capped app's byte change by bundle path; runs a rollover-only tick when recording is off or paused.
- `commands.rs::set_routing`: keeps stored cap blocks and drops cap rules from the incoming list.
- `session.rs::connect`: rollover before the configuration is built.
- `src/lib/api.ts`: `AppCap`, `CapStatus`, `capsApi`, `UsageSettings.appCaps`.
- Tests: 19 in `tests/usage_caps.rs`.
- No changelog or guide entry yet: the feature has no UI until U3.

## Manual check (not run here; needs the U3 editor or a settings edit)
1. Give an app a 1 MB daily cap that offers a block; use the app through the VPN; the 80% and 100% notifications arrive once each.
2. Press Block now (U3): the VPN reconnects, the app has no network, Routing, Apps shows it as blocked by a data cap.
3. Change the Mac's date past midnight (or wait): the block lifts and the VPN reconnects with a notification.
4. Clear details in Settings: the cap's usage does not change.
5. Save the Routing page with an old draft open: the cap block is still there.
