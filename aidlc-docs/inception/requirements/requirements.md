# Requirements

## Intent Analysis
- **User request**: install AI-DLC v1; then build the not-yet-done items from the feature list.
- **Request type**: New features / enhancements (brownfield). **Scope**: multiple components (backend + settings UI + tray popover). **Complexity**: Moderate. **Depth**: Standard.
- **Interpretation note**: Q1=A ("all except NetworkExtension") conflicts with later skips (Q6=none, Q7=skip, Q8=skip QR). The specific answers win: the round is auto-connect by network (3), repair/rollback/diagnostics (4), App Nap (10) and the popover server switcher (11, switcher only). TUIC/WireGuard (5), system-proxy mode (8), QR scanning and NetworkExtension (9) are out of scope. Already shipped: items 1, 2, 6, 7.

## Functional requirements
- **FR1 Auto-connect by network**: identify the network by the default gateway's hardware address (no Location permission). Per-network action: connect, disconnect, or ignore. Configurable in Settings; remembered networks listed with a name the user can edit. Acts on network change events in the existing network watcher; manual connect/disconnect stands the automation down (as F6 does for reconnect).
- **FR2 Diagnostics bundle**: export a .zip to a user-chosen place with redacted app log, sing-box log tail, config with secrets and server addresses removed, versions, settings summary; also a short summary copied to the clipboard.
- **FR3 Engine rollback**: keep the previous engine version on disk; mark a version "good" after a verified connection; one-click rollback on the Core page; option to remove kept old versions.
- **FR4 Repair VPN service**: reinstall the root service/helper (single password prompt), clear stale start/stop/pf state, then re-check status. Available as a button (Core page / Diagnostics).
- **FR5 App Nap**: hold a macOS activity assertion while connected so health and reconnect timers are not throttled; release on disconnect.
- **FR6 Popover server switcher**: list servers (name, latency) of the active group in the menu-bar popover; selecting one switches (without a full reconnect when possible). Also works while disconnected (it changes the active server).

## Non-functional requirements
- No new permission prompts except the existing admin prompt for Repair. Redaction must never leak keys, passwords, UUIDs, subscription URLs or server hostnames/IPs.
- Failures never leave the VPN worse than before (rollback and repair are non-destructive until verified).
- New logic is covered by unit tests; `cargo clippy -D warnings`, `cargo test`, `svelte-check`, build stay green. Root-service actions cannot be exercised in tests; stub-tested and listed for manual check (as in the audit).
- Docs and changelog updated. Delivered on a feature branch with a PR to `dev`.

## Extensions
- Security Baseline: **No**. Property-Based Testing: **No**. Resiliency Baseline: **No**.
