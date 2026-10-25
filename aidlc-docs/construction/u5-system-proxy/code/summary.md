# U5 System proxy: code summary

## Created
- `src-tauri/src/sysproxy.rs`: reading `networksetup` output, the bypass list, set and restore commands, the "still Kage's" check, the restore script, the snapshot, `apply` / `restore` behind a `Runner`, and the app glue (`sync`, `spawn_sync`, `restore_now`, `restore_at_launch`, `current`).
- `src-tauri/src/tests/sysproxy.rs`: 13 tests against a pretend macOS (set then restore, a user change kept, rollback after a failure half-way, new service and new port, a proxy with a login left alone, failed restore kept for Repair, the script's quoting and guards, when it applies, old settings) and one hand-run live test limited to "iPhone USB".

## Modified
- `settings.rs`: `ReliabilitySettings.system_proxy`, `bypass` (defaults keep old settings loading).
- `paths.rs`: `system_proxy_snapshot()`, `system_proxy_script()`.
- `state.rs`: `AppState.system_proxy`.
- `session.rs`: `Status.system_proxy`; set after Connected; restore after a user stop and after a first connect that fails; kept through restarts.
- `reliability.rs`: restore when reconnecting gives up or fails.
- `network_watch.rs`: new services get the proxy while it is set.
- `lib.rs`: restore leftovers at launch; module and command registered.
- `commands.rs`: `set_reliability` checks the bypass list and applies at once; `repair_system_proxy`.
- `runner/supervisor.sh`, `runner/mod.rs`: the supervisor gets the script and saved-file paths and runs the script when a run ends without a stop request; `tests/runner.rs` covers both cases.
- Front end: `api.ts` (settings, `SystemProxyState`, `repairSystemProxy`), `app.svelte.ts` (defaults), `format.ts` (`bypassLines`), `SettingsView.svelte` (toggle, notes, bypass box), `HomePage.svelte` (chip, Repair banner); `scripts/tests/format.test.mjs`; screenshot demo option `systemProxy`.
- Docs: user guide (Proxy Only section rewritten, new "Set system proxy" part), README, changelog.

## Verified
- `cargo test --lib -- --test-threads=4`: 424 passed (1 ignored: the live test). `cargo clippy --all-targets -- -D warnings`: clean. `npm run check`: 0 errors. `npm run test:scripts`: 40 passed. `vite build` and `npm run build:app` (`Kage.app`, 11 MB): ok.
- **Live, on this Mac, with your permission, "iPhone USB" only** (the test cannot see other services): set to 127.0.0.1:21080 with PAC and discovery off and the bypass list, no password prompt; the restore script (the supervisor's path) put it back; the app's restore found nothing left to do; every field equals the state read before the first change. Wi-Fi compared before and after: identical. No sing-box ran; ports 2080 and 19090 were not used.
- Settings section and Home chip checked in the screenshot demo.

## Not verified
- A full connect in Proxy Only mode with the option on in the live app (the app was not run, so your real VPN was not touched): browsing through the system proxy, the restore on disconnect, quit and a force-quit, and Repair.
- A service whose proxy has a login, and a Mac with a PAC file in use (covered by the pretend macOS only).

## Manual check
1. Settings: turn on Set system proxy; connect in Proxy Only mode. Home shows "System proxy on"; System Settings > Network > Wi-Fi > Proxies shows 127.0.0.1 and the port.
2. Disconnect: the proxies are as before.
3. Connect again, then force-quit Kage in Activity Monitor: within a second the proxies are as before.
4. Turn on the proxy password while connected: the option greys out and the proxies are put back.
