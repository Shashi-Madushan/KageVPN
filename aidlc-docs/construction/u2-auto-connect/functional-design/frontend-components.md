# Frontend components
- **Settings > Networks** section (`SettingsView.svelte`, new `NetworksSection.svelte`): master switch; list of known networks, each with an editable name, an action select (Connect / Disconnect / Ignore) and Forget; a line under the Disconnect option saying it also releases the kill switch; "This network" row showing the current network and a one-click "Remember with action" when it is not yet known.
- State: `app.networks` (settings) loaded with the other settings; commands `network_settings`, `set_network_settings`, `current_network`.
- Notification actions (Connect / Ignore) call `network_decide(id, action)`.
- Validation: names trimmed, 1-40 characters; action is one of the three values.
