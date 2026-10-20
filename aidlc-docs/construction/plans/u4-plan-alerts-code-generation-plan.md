# U4 Low-data and expiry alerts: Code Generation Plan
Covers FR4.1 and FR4.2. No functional design stage (skipped in the execution plan). This plan is the single source of truth for U4.

## Finding that shrinks the scope
Most of FR4 already exists (`subscriptions.rs`, `format.ts`, `routes/tray/+page.svelte`):
- A data warning at **90 percent** and another when the allowance is **used up**, each shown once per period (`due_data_notice`, `notified_quota`).
- An expiry warning **3 days** before the end and an **expired** notice, once per expiry date (`due_expiry_notice`, `notified_expire`, `notified_stage`).
- The popover already shows "Plan: 12.4 GB left · 9 days left" for the active server, in a warning colour near the end (`planSummary`).

So only two real gaps remain against the requirement:
1. **Lead times.** The requirement asked for 7, 3 and 1 days; only 3 exists.
2. **Expiry is only checked when a subscription refreshes** (every 12 hours by default, and not at all while refreshes fail). A date is a matter of time, not of a download, so the expiry check should also run on the scheduler's own 30-minute tick, from the figures already stored.

Also, `planSummary` calls the plan "low" only at 3 days; it should match the first warning (7 days).

## Steps
- [ ] 1. **Four expiry stages** (`subscriptions.rs`): `Expiry` becomes `Week` (7 days), `Soon` (3 days), `LastDay` (1 day), `Expired`. Stage numbers stored in `notified_stage` become 1 to 4 in that order; only the highest stage reached is announced, once per expiry date, so a subscription first seen at 2 days announces once, not three times. Notice wording: "X expires in 7 days.", "X expires in 3 days.", "X expires in a day.", "X has expired." One-time effect for existing saved values (old 1 was "soon", old 2 was "expired"): at most one repeat of the notice on the next check, then normal. Tests: each threshold, only-highest-announced, renewal starts over, no expiry means none, the migration case does not panic.
- [ ] 2. **Check on the scheduler tick** (`subscriptions.rs`): a pure `expiry_pass(subscription, now) -> Option<Notice>` that updates the notified fields; `record_refresh` uses it instead of its inline copy, and `spawn_scheduler` runs `check_expiry(app)` at the start of each tick for every subscription with stored figures, saves the notified fields and sends the notices. Test: a subscription whose refresh keeps failing still gets its 3-day notice from the stored date; a second pass sends nothing.
- [ ] 3. **Popover "low" matches** (`format.ts`): `planSummary` flags the plan as low at 7 days or less (data stays at 10 percent left). Tests in `scripts/tests/format.test.mjs` for data left, days left, both, unlimited data, expired, and the low flag.
- [ ] 4. **Docs**: user guide (the subscriptions section mentions notices), changelog entry, `aidlc-docs/construction/u4-plan-alerts/code/summary.md` with manual checks.
- [ ] 5. **Verify**: `cargo test --lib -- --test-threads=4`, `cargo clippy --all-targets -- -D warnings`, `svelte-check`, `vite build`, `npm run test:scripts`.

## Interfaces added
None (internal functions and a changed `Expiry` enum). No new settings: the lead times are fixed at 7, 3 and 1 days.

## Out of scope
A separate data-warning threshold (the 90 percent one stays), notifications for apps' caps (done in U2), a plan line on Home.
