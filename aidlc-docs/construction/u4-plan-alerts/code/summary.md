# U4 plan alerts: code summary
- `subscriptions.rs`: `Expiry` has `Week`, `Soon`, `LastDay`, `Expired` (stored as 1 to 4); `due_expiry_notice` announces only the highest stage reached; `expiry_pass` applies and remembers it; `record_refresh` uses it; `check_expiry` runs on every scheduler tick from stored figures.
- `format.ts`: `planSummary` is low at 7 days or 10 percent of data.
- Already existing and kept: the 90 percent and used-up data notices, and the popover plan line.
- Tests: 5 new Rust tests in `tests/subscriptions.rs` (existing expiry tests rewritten for the new stages); `scripts/tests/format.test.mjs` (4 tests, also covering the rounded speeds fix).
- One-time effect for saved data: old stage 1 (soon) and 2 (expired) are read as week and soon, so one extra notice may appear once.

## Manual check (not run here)
1. With a test subscription whose expire header is 6 days away: a "expires in 6 days" notice within half an hour, then none until 3 days.
2. Block the provider (or use an unreachable URL) and confirm the same notice still arrives.
3. The popover plan line is in the warning colour at 7 days or less.
