# U2 Auto-connect by network: code summary
- `settings.rs`: `NetworkAction`, `KnownNetwork`, `NetworkSettings`; `store.rs` persists them (additive, defaults off).
- `networks.rs`: pure `decide`, `parse_gateway` (netstat), `parse_mac` (arp), names; `current()` reads the network; `apply()` acts; the new-network question is an `osascript` dialog (Ignore/Connect, 60 s timeout = Ignore). Commands `set_network_settings`, `current_network`.
- `network_watch.rs`: evaluates 5 s after launch, and after the settle delay on each network change or wake, unless `manual_actions` changed meanwhile.
- UI: `NetworksSection.svelte` in Settings; `api.ts`, `app.svelte.ts` state.
- Tests: tests/networks.rs (8). Manual check: switch Wi-Fi networks on a real Mac; the dialog; Wi-Fi name usually falls back to "Network xxxx" on recent macOS.
