# Business rules
1. **Identity**: a network is its default gateway's hardware address, taken from the interface-scoped default route of a physical interface (so it works while the tunnel is up). No Location permission. No gateway or no hardware address (no network, or a VPN-only link) means "no network": do nothing.
2. **Naming (Q2=A)**: the Wi-Fi name when macOS exposes it, otherwise "Network <last 4 characters of the MAC>". Always editable. Reading the name never blocks or fails the feature.
3. **Unknown network (Q1=B)**: store it with action `ignore`, and show a notification with Connect and Ignore buttons; the choice is saved as that network's action and also applied now. No action until the user answers.
4. **Known network**: `connect` connects if disconnected; `disconnect` disconnects if connected or connecting; `ignore` does nothing.
5. **Manual choices (Q3=A)**: a manual connect/disconnect (button, menu, shortcut, deep link) stands auto-connect down until the next network change; then rules apply again.
6. **Kill switch (Q4=A)**: a `disconnect` action on a known network releases the kill switch with the disconnect. This is the user's explicit "trusted network" choice and is stated next to the setting.
7. **When evaluated (Q5=A)**: at app start (after the store loads), after wake, and on every network change, each after the existing settle delay, so DHCP and the interface have finished.
8. **Master switch**: nothing happens while `enabled` is false; networks are still remembered only while enabled.
9. **Safety**: auto-connect never runs while the reconnect loop is active, the engine is missing, or there are no servers (a notice is logged, not shown). A failed auto-connect is logged and notified once per network change. Forgetting a network removes its entry.
