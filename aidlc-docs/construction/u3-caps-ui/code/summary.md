# U3 caps UI: code summary
- `usage/commands.rs`: `merge_usage_settings` keeps stored caps when the Usage page saves (closes a U2 gap); test in `tests/usage_caps.rs`.
- `src/lib/capView.ts`: limit parsing and display, reset text, tones, `canBlock`. `appRules.ts`: `mergeCapRules`.
- `src/lib/caps.svelte.ts`: store (5 s refresh and `caps-changed`, only while used); `app.reloadRouting()` in `app.svelte.ts`.
- `CapEditor.svelte`, `AppsTab.svelte` (Cap button or usage pill on each row, Unblock for cap blocks, caps of uninstalled apps), `HomePage.svelte` (Block now cards), `RoutingPage.svelte` (draft follows changes made behind the page).
- Tests: 22 script tests (new: `cap-view.test.mjs`, one more in `app-rules.test.mjs`), 1 more Rust test.
- Dropped from the design: the "block lifted" line on Home (the notification says it).

## Manual check (not run here)
1. Routing, Apps: press Cap on an app, set 1 MB, Day, Offer to block, Set cap. The row shows a usage pill.
2. Use the app through the VPN: 80% and 100% notifications arrive once each; Home shows a Block now card.
3. Block now: the VPN reconnects; the row says blocked by a data cap; the app has no network.
4. Edit something in Routing before pressing Block now, then press it: the page keeps your edit and shows the cap block.
5. Unblock: the block goes; the offer does not come back this period.
6. Turn app usage off in Settings: the cap editor says nothing is counted and offers Turn on.
7. Save Usage settings: the cap is still there.
