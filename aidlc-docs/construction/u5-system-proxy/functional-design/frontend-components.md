# U5 System proxy: frontend components

## Settings (`SettingsView.svelte`, in the section with the local proxy login; Question 6)
- **Toggle** "Set system proxy (Proxy Only mode)". Hint: "Apps that follow the macOS proxy settings use the VPN without setting each one up. Your previous settings come back when you disconnect."
  - Disabled while the proxy login is on, with the note: "Unavailable while the proxy login is on: apps that follow the system proxy usually ignore the login."
  - Saved with `saveToggle` (reverts on a failed save, as the other switches).
  - When the mode is TUN, a muted note: "Used in Proxy Only mode."
- **Extra bypass entries** (shown while the toggle is on): a textarea, one entry per line, with the built-in list shown read-only above it ("Always skipped: localhost, *.local, private ranges"). Saved with a Save button; a refused entry shows the R7 message under the box.
- Both go through `app.saveReliability({ systemProxy, bypass })`.

## Home (`HomePage.svelte`)
- While connected in Proxy Only mode, next to the `127.0.0.1:<port>` chip: a chip "System proxy on" while `status.systemProxy` is `set`.
- `failed` (set or restore): a warning row with the R10 text and a **Repair** button that calls `api.repairSystemProxy()`; shown even when disconnected for a failed restore.

## API
- `ReliabilitySettings` gains `systemProxy: boolean` and `bypass: string[]`.
- `Status` gains `systemProxy: { state: 'off' } | { state: 'set'; port: number; services: string[] } | { state: 'failed'; action: 'set' | 'restore'; reason: string }`.
- New command `repair_system_proxy` → returns the new state, or the "nothing to restore" message.

## Interaction flows
1. Settings: turn on the toggle while connected in Proxy Only → Home shows "System proxy on" within a second; Safari loads through the VPN.
2. Disconnect → the chip goes away; the macOS proxy settings are as before.
3. Turn on the proxy login → the toggle greys out with the note; the system proxy is restored.
4. Force-quit Kage while connected → the supervisor restores; Safari loads directly again.
5. A restore failed → Home shows the warning with Repair; Repair succeeds → the warning goes.
