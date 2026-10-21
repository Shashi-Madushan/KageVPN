# U3 Caps in the UI: Code Generation Plan
Covers FR1.3, FR2.5, FR2.6 and the user-facing side of FR2.1 to FR2.4. Design: `construction/u2-per-app-caps/functional-design/frontend-components.md`. Backend commands exist from U2. This plan is the single source of truth for U3.

## Findings that shape the work
- **A U2 gap to close first.** `usage_set_settings` replaces the whole usage settings, so the Usage page saving any setting (it sends the whole object) would overwrite `appCaps` with whatever the page last loaded. Caps are managed only by the cap commands, so it must keep the stored ones.
- A cap block changes the stored routing behind the Routing page. Its draft only syncs once (`if (app.routing && !draft)`), and `dirty` compares against `app.routing`, so after a cap block the page would look edited and a later Save would send a stale list (the backend already keeps cap rules, but the screen must show the truth).
- `app.reloadSnapshot()` does not refresh `app.routing`; the UI needs a way to re-read it when `caps-changed` arrives.
- A system notification has no buttons, so Block now lives on the Apps row and a Home banner (as designed). The design's "cap block lifted" banner line is dropped: the notification already says it, and a banner for a past event adds state for little value.

## Steps
- [x] 1. **Keep caps when usage settings are saved** (`usage/commands.rs`): `usage_set_settings` copies `app_caps` from the stored settings. Extract `merge_usage_settings(current, next)` (also holds the existing alert reset and `cycle_day` clamp) and test it: caps survive, alert level resets when the cap changes, `cycle_day` clamps.
- [x] 2. **Pure view helpers** (`src/lib/capView.ts`): `toBytes(value, unit)` (MB or GB, rejects non-numbers, below 1 MB or above 10 TB), `splitBytes(bytes)` (the value and unit to show), `periodWord(period)`, `resetText(periodEnd, period, now)` ("resets at midnight", "resets Monday", "resets 1 Nov"), `barTone(state)`, `canBlock(status)` (offer pending and not blocked). Tests in `scripts/tests/cap-view.test.mjs`, and a `mergeCapRules(draftRules, storedRules)` test added to `app-rules.test.mjs` (see step 6).
- [x] 3. **Caps store** (`src/lib/caps.svelte.ts`, `app.svelte.ts`): holds the `CapStatus` list; `use()` starts a 5-second refresh and a `caps-changed` listener, reference-counted so it runs only while a consumer is mounted; actions `save`, `remove`, `block`, `notNow`, `unblock` call `capsApi` and show errors with `app.notify`. `app.reloadRouting()` re-reads `routing` from the snapshot; every cap action and `caps-changed` call it. Derived `pending` (offers waiting for a decision) and `byPath`.
- [x] 4. **Cap editor** (`CapEditor.svelte`): opens under an Apps row. Limit number plus MB or GB, period (Day, Week, Month), at-limit (Notify only, Offer to block); Save disabled with the reason shown when the limit is invalid; Remove cap. With a cap: progress bar, "1.4 GB of 2 GB today, approximate" and the reset text, the 80 percent and over colours. State buttons: **Block now** and **Not now** (over, pending), **Unblock** (blocked), "App usage is off" with **Turn on** (not counting, enables recording through `usageApi`), "Not installed" with **Remove**.
- [x] 5. **Apps tab** (`AppsTab.svelte`): each row gets a small **Cap** button, or a thin usage bar (and the percentage) when a cap exists; clicking it opens the editor under the row. The Blocked strip's cap blocks get an **Unblock** button (`caps.unblock`). Caps for apps no longer installed are listed in "Other app rules" with Remove. Row text shows "over its data cap" for over and blocked states.
- [x] 6. **Routing page stays truthful** (`RoutingPage.svelte`, `appRules.ts`): track the settings last synced into the draft; when `app.routing` changes, resync fully if the draft is unedited, otherwise replace only the draft's cap-origin rules with the stored ones (`mergeCapRules`). `dirty` then reflects only the user's own edits.
- [x] 7. **Home banner** (`CapBanner.svelte`, `HomePage.svelte`): one card per pending offer, "Slack reached its 2 GB daily cap. Block it?", with **Block now** (confirmation text says the VPN reconnects) and **Not now**. Hidden when none.
- [x] 8. **Docs**: user guide section "Data caps for apps" (set a cap, what 80 and 100 percent do, Block now, reset, approximate counting, recording must be on, Kage cannot be capped), changelog entry for U2 and U3 together, `aidlc-docs/construction/u3-caps-ui/code/summary.md` with the manual checks.
- [x] 9. **Verify**: `cargo test --lib -- --test-threads=4`, `cargo clippy --all-targets -- -D warnings`, `svelte-check`, `vite build`, `npm run test:scripts`.

## Interfaces added
`merge_usage_settings` (internal), the TS modules above, `app.reloadRouting()`. No new Tauri commands.

## Out of scope for U3
Popover alerts and the 90 percent plan and expiry warnings (U4); a cap column on the Usage page.
