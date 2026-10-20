# U2 Per-app caps: Functional Design Plan
Covers FR2.1 to FR2.7 (and the unblock side of FR3.2). Technology-agnostic business logic; code comes after approval.

## What the code showed (inputs to the design)
- The connection poll already gives every live connection's bytes and its **app bundle path** (`Connection.app_path`), every 2 seconds. Connections that close between two polls are missed (already true for the usage page).
- Usage detail rows (`usage_dim`) are keyed by **process name**, per hour, and are deleted after the "keep detail" setting (default 90 days) or when the user presses Clear details. A cap counted from them would be wrong after either, and process names do not map cleanly to apps.
- Usage recording can be off or paused; the poll then skips everything.
- A system notification is plain text with no buttons in this build, so **Block now** lives on the Apps row and a Home banner.
- A routing change (block or unblock) reconnects the VPN today.

## Steps
- [x] 1. Answer the questions below.
- [x] 2. Check the answers for contradictions or vague wording; ask follow-ups if needed.
- [x] 3. Write `aidlc-docs/construction/u2-per-app-caps/functional-design/domain-entities.md` (cap, period, tally, cap state).
- [x] 4. Write `business-rules.md` (period boundaries, counting, thresholds, when a block is offered, when it is lifted, edge cases).
- [x] 5. Write `business-logic-model.md` (the poll-to-alert flow, Block now, Unblock, period reset, settings changes).
- [x] 6. Write `frontend-components.md` (cap editor on the Apps row, banner, states; the API each uses).
- [ ] 7. Present for approval.

## Proposed design in short (so the questions make sense)
A cap is set per installed app (by bundle path): a limit in bytes and a period (daily, weekly, monthly). The app keeps its **own running tally per app and period** (a new small table, independent of the usage detail and its retention). Each poll adds the bytes of that app's live connections to the tally, if recording is on. At 80 percent and 100 percent one notification is sent per period. At 100 percent the Apps row and a Home banner show **Block now**; clicking it adds a block rule marked as a cap block and reconnects. When the period rolls over, the tally restarts and a cap block is lifted.

## Questions

## Question 1
Where should the per-app tally come from?

A) A new dedicated tally per app and period, counted from the poll by bundle path (unaffected by "Clear details" or the retention setting, exact to the app)

B) Reuse the existing usage detail rows and match them to the app by name (no new table, but wrong after Clear details or the retention limit, and names can clash)

C) Other (please describe after [Answer]: tag below)

[Answer]: A (not answered by the user; chosen by the assistant as recommended: a dedicated tally is exact and survives Clear details)

## Question 2
Programs without an app bundle (command-line tools such as `curl`, processes shown only by name) cannot be identified by bundle path. Can they have a cap?

A) No, caps are for installed apps only (the Apps tab rows)

B) Yes, also for process names, set from the Other app rules list

C) Other (please describe after [Answer]: tag below)

[Answer]: A (not answered by the user; chosen by the assistant as recommended: caps are for installed apps only)

## Question 3
When does each period start?

A) Daily at local midnight, weekly on Monday at midnight, monthly on the 1st at midnight

B) Daily at local midnight, weekly on Sunday, monthly on the same billing-cycle day as the global data cap

C) Other (please describe after [Answer]: tag below)

[Answer]: A (not answered by the user; chosen by the assistant as recommended: calendar boundaries are the simplest to explain)

## Question 4
If you change an app's limit or period after some usage was counted, what happens to the tally?

A) Keep what was counted in the current period (a bigger limit may remove an alert state; a smaller one may trigger it at once)

B) Restart the count from zero as if the cap were just created

C) Other (please describe after [Answer]: tag below)

[Answer]: A (not answered by the user; chosen by the assistant as recommended: keep what was counted)

## Question 5
When a period ends while an app is blocked by its cap, the block is lifted. How should that reconnect be handled?

A) Remove the cap block and reconnect automatically, with a notification ("Slack can use the VPN again")

B) Remove the cap block now but let it take effect at the next reconnect, with a notification; no interruption

C) Ask first with a banner, like Block now

D) Other (please describe after [Answer]: tag below)

[Answer]: A (not answered by the user; chosen by the assistant as recommended: lift the block and reconnect automatically)

## Question 6
If the VPN is off when a period ends, or the app is quit, what should happen to a cap block?

A) The block is lifted the next time the app starts or connects, before the VPN comes up

B) Stay blocked until the user clicks Unblock

C) Other (please describe after [Answer]: tag below)

[Answer]: A (not answered by the user; chosen by the assistant as recommended: lift the block before the VPN comes up)

## Question 7
The 80 percent and 100 percent alerts: what should the wording and the number of repeats be?

A) One notification at each level per period, naming the app and its usage ("Slack used 80% of its 2 GB daily cap")

B) Same, and also repeat the 100 percent one each time the app is opened while over the cap

C) Other (please describe after [Answer]: tag below)

[Answer]: A (not answered by the user; chosen by the assistant as recommended: one alert per level per period)

## Question 8
What if usage recording is off or paused (so no bytes are counted)?

A) Caps show "not counting" with a button to turn recording on; no alerts or blocks happen meanwhile

B) Counting for caps always runs from the poll even if the usage history is off (caps then work independently of the usage page settings)

C) Other (please describe after [Answer]: tag below)

[Answer]: A (not answered by the user; chosen by the assistant as recommended: respect the privacy choice: no counting while recording is off)

## Question 9
How many apps may have a cap, and what limits are allowed?

A) Any number of apps; limits from 1 MB to 10 TB, entered in MB or GB

B) At most 20 apps, same limits

C) Other (please describe after [Answer]: tag below)

[Answer]: A (not answered by the user; chosen by the assistant as recommended: no artificial limit on the number of apps)

## Question 10
The tally misses bytes of connections that opened and closed between two polls (up to a few seconds of traffic, and short downloads that finish fast). Is that acceptable, with a note in the UI?

A) Yes, show "approximate" next to figures and explain it in the docs

B) No, improve it: also read the final byte counts when a connection disappears from the list (more work, still not exact)

C) Other (please describe after [Answer]: tag below)

[Answer]: A (not answered by the user; chosen by the assistant as recommended: approximate, stated in the UI)
