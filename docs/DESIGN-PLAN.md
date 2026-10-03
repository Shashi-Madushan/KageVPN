# Osprey VPN — UI/UX Audit & Design Improvement Plan

Scope: Tauri + SvelteKit desktop VPN manager (main window 780×580 min 680×480, 320×460 tray popover).
Method: design-critique framework (hierarchy, usability, consistency, accessibility) applied to the code in `src/`.

## 1. What works well
- Native macOS feel: vibrancy sidebar, overlay title bar, SF fonts, light/dark via `prefers-color-scheme`.
- Home has a clear focal point (power button + status pill) and good empty/onboarding states.
- Semantic colour use (ok/warn/danger, down/up) and `tabular-nums` for live numbers.
- `:focus-visible` ring, reduced-motion support, `aria-label`s on nav and radiogroups.

## 2. Findings

| # | Area | Finding | Severity |
|---|------|---------|----------|
| 1 | Responsive | Fixed 188px sidebar; at the 680px window minimum content gets ~490px. Only Home has media queries (3 total). | 🔴 |
| 2 | Responsive | Connections table: 7-column grid with fixed widths (~460px minimum) overflows/clips on narrow windows. | 🔴 |
| 3 | Responsive | Wide windows: content stretches edge to edge, lines/cards become unreadably long. | 🟡 |
| 4 | Consistency | Toggle switch CSS copy-pasted in 5 components; button/input styles redefined per page (37 hairline-border rules). | 🟡 |
| 5 | Consistency | Page title "Profiles" vs sidebar "Servers"; "Core Engine" vs "sing-box core"; "Data Usage" vs "Data usage". | 🟡 |
| 6 | Hierarchy | Home shows 6+ competing blocks (banner, hero, 2 stat tiles, chart, today, 3 tool buttons, hint). Primary action should dominate. | 🟡 |
| 7 | Hierarchy | Page header (h1 only) is wasted space; no place for page-level actions (Add server, Close all). | 🟡 |
| 8 | Accessibility | Many 10–11px labels (step cards, captions, badges); muted text on translucent surfaces risks < 4.5:1. | 🟡 |
| 9 | Accessibility | Icon-only controls (`✕`, chips) are < 24px hit area; collapsed nav needs tooltips/labels. | 🟡 |
| 10 | Accessibility | Status conveyed by colour + dot; add text/icon redundancy (already has text on Home — keep for list rows). | 🟢 |
| 11 | UX | Emoji used as icons (🛡️, ⚡); inconsistent with the stroked SVG icon set. | 🟢 |
| 12 | UX | No inline feedback hierarchy for errors vs. info (global `notify` only). | 🟢 |

## 2b. Competitive research (NordVPN, Proton VPN, Mullvad, Surfshark)

Sources: [Tom's Guide on NordVPN's redesign](https://www.tomsguide.com/computing/vpns/nordvpn-redesigns-its-desktop-app-heres-whats-new), [TechRadar](https://www.techradar.com/vpn/vpn-services/nordvpns-desktop-apps-just-got-a-new-look-heres-all-you-need-to-know), [NordVPN blog](https://nordvpn.com/blog/desktop-app-redesign-2026/), [Comparitech Proton vs Mullvad](https://comparitech.com/blog/vpn-privacy/proton-vpn-vs-mullvad-vpn), [CyberInsider Surfshark vs Mullvad](https://cyberinsider.com/vpn/comparison/surfshark-vs-mullvad/).

| App | Pattern | Takeaway for Osprey VPN |
|-----|---------|--------------------------|
| NordVPN | Search-first server picker replaced the long country list; recommended location shown on launch; map confirms location; stats dashboard; guided first minutes. | Status-first Home with the recommended/active server surfaced; search in the server picker; keep onboarding. |
| Proton VPN | Map or list server selection; tabs keep UI uncluttered, but kill switch / protections were buried. | Surface protections (kill switch, auto-reconnect, mode) as chips on Home, not only in Settings. |
| Mullvad | Minimal, same design on every platform, all settings one click away, strong status colour. | Large status headline in green/neutral, minimal chrome, consistent components. |
| Surfshark | Minimal connect pane plus feature access on the side; toggles and dropdowns for quick changes. | Keep one primary pane + quick toggles; use consistent switch/dropdown primitives. |

Conclusions applied: (1) status headline ("You're protected") is the hero, power button supports it; (2) protections visible at a glance; (3) quick tools demoted to a compact row; (4) consistent primitives; (5) searchable, latency-aware server list (already present in Servers; kept and polished).

## 3. Design principles
1. **One primary action per screen** — Connect on Home, Add on Servers.
2. **Tokens over literals** — all spacing, type, radii, elevation from CSS variables.
3. **Content-width, not window-width** — layouts respond to the container (container queries), not just viewport.
4. **Progressive disclosure** — secondary tools collapse before primary ones as space shrinks.
5. **Native first** — respect macOS vibrancy, system accent where possible, 28px min hit targets.

## 4. Design system foundation (Phase 1 — implemented)
- Tokens: spacing `--space-1..6`, type `--text-xs..xl`, radii, elevation, `--sidebar-w`, `--content-max`, focus ring.
- Shared primitives in `app.css`: `.btn`, `.btn.primary`, `.btn.danger`, `.input`, `.switch`.
- Raise minimum text size to 11px for body-level labels; `--fg-muted` kept ≥ 4.5:1.

## 5. Responsive strategy (Phase 1 — implemented)
| Window width | Layout |
|--------------|--------|
| ≥ 1100 | Sidebar 200px, content centred, max 960px |
| 760–1099 | Sidebar 188px, content fluid |
| 560–759 | **Icon rail** sidebar (56px), labels become tooltips, badges become dots |
| < 560 | Rail + single-column everywhere; tables become stacked rows |

Components:
- Shell: rail via media query; main content `max-width` + auto margins; reduced paddings when narrow.
- Home: container query on `.home` (2-col → 1-col at 560px of *content* width, quick tools 3→1).
- Connections: columns drop (App, Route, Age) at narrow container widths; destination always visible.
- Dialogs: form grids collapse to 1 column; dialogs `max-width: calc(100vw - 32px)`.
- Tauri `minWidth` 680 → 520 so the compact layouts are reachable.

## 6. Roadmap & progress tracker

Legend: ✅ done · 🟡 partial / needs humans · ⬜ not started

### Phase 1 — Foundation & responsiveness
- ✅ Tokens, `.btn`/`.input` primitives
- ✅ Adaptive sidebar (icon rail < 760px), centred content max-width
- ✅ Home container queries, Connections column dropping, dialog single-column
- ✅ Tauri `minWidth` 680 → 520

### Phase 2 — Consistency
- ✅ One global toggle switch (removed 4 duplicated scoped copies)
- ✅ Unified naming: Servers / Data usage / Engine (sidebar + titles + copy)
- ✅ SVG icons (shield, refresh) replace emoji on Home chips; emoji removed from Servers sort; Select auto glyph is now plain text
- ✅ `.btn` / `.input` adopted on Servers, Connections and Settings (scoped duplicates removed); remaining scoped button rules (Routing, Core, Usage, Diagnostics, dialogs) are cosmetic leftovers

### Phase 3 — Hierarchy & UX
- ✅ Home: status headline hero, protection chips (kill switch, auto-reconnect, mode), compact quick-tools row
- ✅ Page header subtitles (context under each title)
- ✅ Servers: search, sort by ping, test all, grouping (pre-existing, kept)
- ✅ Page-header action slot (`HeaderActions`), used by Servers and Connections
- ✅ Compact/comfortable density toggle on Servers (persisted)
- ✅ Inline recovery actions: Connect now / Retry on Connections

### Phase 4 — Polish & accessibility
- ✅ Muted text contrast raised (light `#636366`, dark `#a1a1a8`)
- ✅ 10px text raised to 11px
- ✅ 28px minimum hit areas on star / more / close controls
- ✅ ⌘1–⌘9 page switching, shown in rail tooltips
- ✅ Arrow-key navigation inside sidebar and server list

### Phase 3b — Page redesigns (Logs, Data usage)
- ✅ Logs: level tags (not colour-only), error row tint, Copy/Clear in header, level segmented filter, line counts, Following/Paused status with "Jump to latest", contextual empty states
- ✅ Data usage: one summary card (total, delta, download/upload split, avg/peak/connected, cap), range as one dropdown, filters tucked in a popover, group-by and compare moved to the chart, single tabbed breakdown (Apps / Sites / Servers), settings collapsed under "Limits and privacy" with destructive actions separated

### Phase 3c — Page redesigns (Routing, Diagnostics, Engine)
- ✅ Routing: icon mode cards (radiogroup), one settings list (LAN bypass, default action, DNS), rules as rows with value + kind, preset chips instead of emoji buttons, 28px row actions, empty state; fixed a stray `input { border-top }` rule that styled every input
- ✅ Diagnostics: verdict-first status card with last-run time and inline TUN remediation, 3-up result cells, speed test with progress and inline Connect, Copy report moved to the header
- ✅ Engine: active-engine hero with health colour, one unified Versions list (system, managed, available) with channel switch, "New" badge, inline progress, locked-while-connected notice, Check for updates in the header

### Phase 3d — De-templating Routing and Engine
Critique: stacked identical translucent cards, icon-in-tinted-tile hero, pill badges on every row, card-style single-choice picker, controls inside cards, lists where tables belong.
- ✅ New shared "grouped list" primitives in `app.css` (`.section`, `.section-head`, `.group`, `.item`, `.state` dot status, `.linkbtn`): section labels outside, key/value rows, hairline separators
- ✅ Engine: key/value "Active engine" group, real table for Versions (Version / Released · size / Status / action), plain-text status with a dot instead of pills, container-query column hiding
- ✅ Routing: segmented Mode control with one-line explanation, General settings group, rules as compact rows (type · match · action) with hover-revealed reorder/delete, text-link presets

### Phase 3e — Tray popover (320×460)
Audit: server name and flag shown twice, picker boxed inside a card, dead blank graph when disconnected, dead disabled controls with no servers, no headline matching Home, ~20px hit targets.
- ✅ Centred hero with power button + "You're protected" headline; one sub line (server · time · ping) that truncates; exit IP below
- ✅ Server picker unboxed; traffic card only shown when connected (disconnected shows just Today usage)
- ✅ No-servers state: single "Add a server…" action, usage/mode hidden
- ✅ 28px close and footer targets, 12px text floor, "Disconnect to change…" note when locked
- ✅ `--ok-text` token (4.5:1+ on light) for green headlines on Home and tray
- 🟡 Native right-click menu (Open / Connect / Quit) reviewed in code, unchanged

### Phase 3f — Server management and dialogs
- ✅ Servers: single primary "Add server" (+ Test all) in the header; flat grouped list with Server / Protocol / Latency columns, accent bar on the active server, colour-coded latency, flag shown once, compact density, row menu with grouped actions
- ✅ Subscriptions collapsed to rows with an inline "+ Add subscription…" form; hidden in the empty state, which has its own two clear actions
- ✅ Shared `Dialog` (header, scrolling body, footer, backdrop/Esc close) and `ConfirmDialog`; Add server, New/Edit server, QR and delete confirmations rebuilt on them
- ✅ Add server: auto-fills from clipboard, drop a file, live "N links ready", "Enter details manually" fallback
- ✅ Edit server: Connection / Security / Transport sections, Advanced options collapsed, inline UUID validation, Save disabled until valid
- ✅ Fixed: segmented options inside a form submitted it (added `type="button"`)

### Phase 3g — Onboarding tour
- ✅ Removed the "Welcome" card from Home; Home keeps its engine banners and a small "Take the quick tour" link when there are no servers
- ✅ 6-step tour dialog with live demos built from the real components (PowerButton, Select, Segmented, Sparkline): welcome, add a server, choose a mode, connect, menu bar/shortcut, setup checklist
- ✅ Opens once on first launch with no servers; existing users are marked as seen; replay from Settings → Help
- ✅ Last step adapts: "Install the engine first" when missing, primary becomes "Add a server" and opens that dialog on Servers
- ✅ Arrow-key navigation, step dots, Skip; fixed dialog height between steps; opaque `--dialog-bg` for all dialogs

### Phase 3h — Speed test gauge
- ✅ Backend streams live readings (`speed-progress`, ~6/s): ping + jitter first, then time-boxed download (4 parallel streams) and upload (3 streams), 8 s each, with a cancel command
- ✅ Ookla-style dial: 270° non-linear scale (0–1000 Mbps), animated needle and arc, big live number, phase label, live graph per phase
- ✅ Ping / Download / Upload result tiles fill in as each phase finishes; the finished dial shows the download result
- ✅ Cancel, inline error, light and dark themes, 520px layout

### Phase 3i — Theme toggle
- ✅ Icon-only System / Light / Dark selector at the bottom of the sidebar (arrow-key support); the narrow icon rail uses one cycling button
- ✅ Saved per device, applied before first paint, shared with the tray popover, and passed to the native window (`setTheme`) so the frosted sidebar matches
- ✅ CSS tokens follow the OS only while "System" is selected

### Phase 5 — Validation (needs real users)
- 🟡 Test script prepared below; sessions not run
  1. "Connect to the VPN" (target < 5 s, 0 misclicks)
  2. "Add a server from a vless:// link"
  3. "Find out why the VPN disconnected" (logs/diagnostics)
  4. "Switch to the fastest server"
  5. Resize window to minimum and find Settings

## 7. Success metrics
- No horizontal overflow or clipped content from 520px to 1600px wide.
- All text ≥ 4.5:1 contrast; all interactive targets ≥ 28px.
- Zero page-local re-definitions of switch styles (done); button styles (in progress).
