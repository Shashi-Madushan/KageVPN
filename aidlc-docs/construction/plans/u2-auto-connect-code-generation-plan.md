# U2 Auto-connect by network: Code Generation Plan
Single source of truth. Covers FR1; design in construction/u2-auto-connect/functional-design/.

## Deviation to confirm
Q1=B asked for a notification with Connect/Ignore buttons. The notification plugin cannot show buttons on macOS desktop. The plan asks with a small macOS dialog (`osascript display dialog`, buttons Ignore / Connect, gives up after 60 s and counts as Ignore). It appears only for a never-seen network while the feature is on.

## Steps
- [x] 1. **Settings** (`settings.rs`, `store.rs`): `NetworkAction`, `KnownNetwork`, `NetworkSettings` (default off, empty); stored in `Data` with serde defaults so old files load; store getters/setters; schema stays at 4 (additive).
- [x] 2. **Pure logic** (`networks.rs`): `decide(settings, current_id, state, manual_override, waiting_reconnect) -> Decision`; `parse_gateway(route_output) -> Option<String>`; `parse_mac(arp_output) -> Option<String>`; `default_name(mac)`; name validation.
- [x] 3. **Gateway reading** (`networks.rs`): run `/sbin/route -n get default` (scoped to a physical interface when the tunnel is up) and `/usr/sbin/arp -n <gateway>`; short timeouts; failures give `None`.
- [x] 4. **Hook** (`network_watch.rs`, `lib.rs`): evaluate at app start, after wake and on every network change after the settle delay; `manual_override` flag in `AppState` set by manual connect/disconnect paths and cleared on the next network change; disconnect action releases the kill switch (existing disconnect path with `UserStop`).
- [x] 5. **Commands** (`commands.rs`, `lib.rs`): `network_settings`, `set_network_settings` (validates names), `current_network`, `network_remember(action)`.
- [x] 6. **Tests** (`tests/networks.rs`): decide() for every row of business-rules.md (disabled, unknown, connect/disconnect/ignore, manual override, reconnect loop active, already in desired state); parsers on real `route`/`arp` sample output; name validation; old data file without the field still loads.
- [x] 7. **UI**: `api.ts` types and wrappers; `NetworksSection.svelte` in Settings (switch, current network row, list with editable name, action select, Forget, kill-switch note).
- [x] 8. **Docs**: changelog entry, guide mention; `aidlc-docs/construction/u2-auto-connect/code/summary.md`.
- [ ] 9. **Verify**: `cargo test`, `cargo clippy --all-targets -- -D warnings`, `svelte-check`; manual check list (real network switch, dialog).
