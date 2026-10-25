# Requirements: per-app controls

## Intent analysis
- **Request:** a per-app picker for bypass and proxy rules, per-app data caps, and a way to disable chosen apps' network access (like the DataEye app); plus low-data and expiry alerts in the menu bar.
- **Type:** New feature and enhancement (brownfield). **Clarity:** clear after the questions. **Scope:** multiple components (routing, usage, session, UI, tray). **Complexity:** moderate. **Depth:** standard.
- Answers: `app-controls-questions.md`. Q1 and Q2 were not answered by the user and were chosen by the assistant (C and C). Answers checked for contradictions: none found.

## What exists and is reused
Routing rules by app (`apps.rs`, `AppPicker.svelte`, RoutingPage), Route buttons on the Connections page, per-app usage rows in the usage database, a global cap with 80% and 100% alerts (`usage/quota.rs`), plan and expiry parsing (`format.ts` `planSummary`), the Networks feature.

## Functional requirements

### FR1. Apps tab (Routing page)
- FR1.1 A searchable list of installed apps with icon, name and a three-way switch: Use the VPN, Go direct, Block. A choice is an override; apps without one follow the routing default.
- FR1.2 Presets at the top: "Everything through the VPN, except the apps I pick" and "Only the apps I pick through the VPN". A preset sets the routing default; per-app overrides stay.
- FR1.3 Each row shows the app's current usage (this cycle) and its cap progress when a cap is set.
- FR1.4 Rules made by process name (no `.app`) stay in the rule editor and also appear in the Apps tab as rows labelled with the process name.
- FR1.5 Changing a choice edits the existing routing rules (same storage as today) and reconnects the VPN the way routing changes do now, after the user confirms once per batch of changes.

### FR2. Per-app data caps
- FR2.1 Each app can have a cap in bytes with its own period: daily, weekly or monthly.
- FR2.2 Usage counted is what the app moved through the VPN (existing usage data); direct traffic is not counted.
- FR2.3 Notifications at 80% and 100% of an app's cap, once per period.
- FR2.4 Per app action at the cap: **notify only**, or **block the app until the period resets**.
- FR2.5 Blocking is asked for, not automatic: at 100% the user is told and gets a **Block now** action (on the Apps row, in the Home banner, and in the notification where the system allows an action). Nothing changes without a click.
- FR2.6 A blocked-by-cap app is unblocked automatically when its period resets, and any time by an **Unblock** button on its row.
- FR2.7 Caps are soft: counting uses the connections poll (about every 2 seconds), so an app can overshoot by a few seconds of traffic. The UI and docs say so.

### FR3. Disable usage for selected apps
- FR3.1 A per-app switch that blocks the app's network access completely (the existing Block action), switchable on and off from the Apps tab, like a firewall toggle in a data-monitor app.
- FR3.2 Blocked apps are listed together and clearly marked; the cap block and the manual block are shown as different reasons.

### FR4. Low-data and expiry alerts (round 2 of this request)
- FR4.1 A warning at 90% of the provider's data allowance and when the plan has 7, 3 or 1 days left (the 90% value replaces nothing existing; the app's own 80% and 100% cap alerts stay).
- FR4.2 Shown in the menu-bar popover (remaining days and traffic) and as a notification, once per threshold per plan period.

## Non-functional requirements
- NFR1 Existing routing rules and settings keep loading (new settings fields default; no migration step).
- NFR2 A cap or block never leaves the app unable to connect: if a rule would block the VPN's own traffic the change is refused.
- NFR3 Counting and cap checks run off the existing poll; no new background process; CPU cost is negligible.
- NFR4 Apps list loads from the cached installed-apps scan; the tab stays responsive with several hundred apps (render only the visible rows or limit shown rows).
- NFR5 Works in light and dark themes and at narrow window widths, like the other pages.
- NFR6 Tests: pure functions for period boundaries, cap decisions and rule edits are unit-tested; the existing 300+ tests, clippy and svelte-check stay clean.
- NFR7 Extension rules (security baseline, property-based testing, resiliency baseline) stay disabled for this work (answer Q13 A).

## Out of scope
- Strict, immediate enforcement of caps.
- Counting direct (non-VPN) traffic.
- Per-network or scheduled app blocks (Q7 chose the plain block).
- The universal Intel and Apple Silicon build (Q12 chose A and B only).

## Open points to settle in design
- Whether a rule change can apply without a full reconnect (a spike; falls back to reconnecting with confirmation).
- Whether the system notification can carry a Block now button through the notification plugin; if not, the Home banner and Apps row carry it.
