# U5 System proxy: Code Generation Plan
Round 4, pull request 2 (branch `feature/system-proxy`). Implements the approved functional design in `construction/u5-system-proxy/functional-design/` (rules R1 to R11, flows F1 to F8). This plan is the single source of truth for U5.

## Hook points found in the code
| Flow | Where |
|---|---|
| Set when listening (F1) | `session::connect`, right after the status turns Connected (`proxy_port` is set there) |
| Keep through restarts (F2) | `session::stop_with(.., Ending::Restart)` (used by `restart_tunnel` and `disconnect_for_restart`): no restore; the next connect sets the new port |
| Restore on stop (F3) | `session::stop_with` for `Ending::UserStop`, and for `Ending::Failed` when not reconnecting (`holds_block` already tells these apart) |
| Restore when reconnecting gives up (F3) | `reliability::spawn_reconnect`, outcomes `GaveUp` and `Failed` |
| Quit (F3) | `tray::quit_app` already calls `session::disconnect` before `app.exit(0)` |
| Supervisor (F4) | `runner/supervisor.sh`: after its loop, when the stop file is missing |
| Launch (F5) | `lib.rs` setup, before `auto_connect` |
| New services (F6) | `network_watch::handle`, at the start of a change event |
| Toggle, login, mode (F7) | `commands::set_reliability` and `commands::set_mode` |
| Status to Home | `session::Status` gains `system_proxy` |

## Steps
- [x] 1. **Settings** (`settings.rs`, `src/lib/api.ts`): `ReliabilitySettings.system_proxy: bool` (default false) and `bypass: Vec<String>` (default empty), with serde defaults so saved settings load unchanged. Tests: old settings load with the defaults; round trip.
- [x] 2. **Pure core** (`src-tauri/src/sysproxy.rs`, new; tests in `tests/sysproxy.rs`):
  - parse `networksetup -listallnetworkservices` (enabled services, `*` = disabled, the header line skipped) and the six `-get...` outputs into `ServiceProxy`;
  - `bypass_list(user) -> Result<Vec<String>>` (built-in first, validation and the R7 message, at most 200);
  - `set_commands(service, port, bypass)` and `restore_commands(saved)` as argument lists for `networksetup` (Web, Secure Web, SOCKS, PAC URL and its state, auto-discovery, bypass, with `Empty` for an empty bypass list as `networksetup` expects);
  - `still_kage(current, port) -> bool` (R4);
  - `restore_script(snapshot) -> String`: per service a guard on `networksetup -getwebproxy`, `-getsecurewebproxy` and `-getsocksfirewallproxy` showing `127.0.0.1` and the port, then the restore commands; every argument shell-quoted;
  - `Snapshot { port, services }` (serde) and `SystemProxyState`.
  Tests with recorded `networksetup` outputs (proxies on and off, PAC set, bypass empty and filled, a disabled service), command lists, guard and quoting in the script (a service name with spaces and an apostrophe).
- [x] 3. **Effects** (`sysproxy.rs`): a `Runner` trait for running `networksetup` (real: `/usr/sbin/networksetup`; tests: a fake that records commands and serves outputs); `apply(port, settings)` (F1, R3 with partial-failure rollback), `update_port(port)` (F2), `add_new_services()` (F6), `restore()` (F3, R4), `restore_at_launch()` (F5), `repair()` (F8); snapshot and script written with `fsutil::write_private_atomic` to `paths.system_proxy_snapshot()` and `paths.system_proxy_script()`; state kept in `AppState` and emitted with the status. Tests with the fake runner: apply then restore puts every value back; a service changed by the user is left alone; failure half-way rolls back; a new service is added to the snapshot; a port change rewrites the script; restore with no snapshot is a no-op; a restore failure keeps the files.
- [x] 4. **Session hooks** (`session.rs`, `reliability.rs`, `network_watch.rs`, `lib.rs`, `paths.rs`): wire F1 to F7 at the hook points above; `Status.system_proxy`; R1 checks (mode, toggle, login). Tests for the pure decision `should_restore(ending, reconnecting)` and `applies(mode, settings)`.
- [x] 5. **Supervisor** (`runner/supervisor.sh`, `runner/mod.rs`): `start_plain` passes the restore script's path as a new last argument; after the loop, if the stop file is missing and the script exists, run it with `/bin/sh` and delete it and the snapshot. Script test in `tests/runner.rs` (runs the supervisor with a fake `sing-box` that exits at once and a restore script that writes a marker file: the marker appears without a stop file, and not with one).
- [x] 6. **Commands** (`commands.rs`): `set_reliability` applies or restores at once when the toggle, login or bypass list change while connected in Proxy Only (F7); `set_mode` restores when leaving Proxy Only while connected; new `repair_system_proxy`. Register it in `lib.rs`.
- [x] 7. **Interface** (`SettingsView.svelte`, `HomePage.svelte`, `src/lib/api.ts`, `src/lib/app.svelte.ts`): the toggle with its hint, the login note and the TUN note; the bypass box with the built-in list shown, Save and error; the Home chip and the warning with Repair. Script tests for any pure helper (bypass text to list).
- [x] 8. **Runtime check on this Mac** (R11, NFR2), only after asking you: on the unused service "iPhone USB", save its six settings to a file; run the real `apply` for port 21080 through a small test binary or `cargo test -- --ignored` test; read the settings back (they must show 127.0.0.1:21080 and PAC off); run `restore`; compare with the saved file (must be identical); then run the supervisor path once (apply, kill the fake parent, check the restore ran). Wi-Fi and the other services are not touched; ports 2080 and 19090 are not used.
- [x] 9. **Docs**: user guide (Proxy Only mode: the system proxy option, what it changes, the bypass list, the crash restore, the login limitation), Settings reference, README feature line, changelog (21 October, not released), correct the earlier docs line that says Proxy Only does not change the system proxy; screenshots of the Settings section and the Home chip through the screenshot demo; `aidlc-docs/construction/u5-system-proxy/code/summary.md`.
- [x] 10. **Verify**: `cargo test --lib -- --test-threads=4`, `cargo clippy --all-targets -- -D warnings`, `npm run check`, `npm run test:scripts`, `npx vite build`, `npm run build:app`.

## Results and decisions made while coding
- **No password, confirmed:** the live test changed and restored a real service as a normal user with no prompt.
- **Exact restore of proxies that were off:** the live test showed a restore left Kage's address in the switched-off fields. `networksetup` accepts an empty address, so a restore now sets each proxy's old address (often empty) and then turns it off. A second live run ended with every field identical to before.
- **Proxies with a login are left alone:** their password is in the Keychain and `networksetup` cannot put it back, so such a service is not changed (logged).
- **One component for set, port change and new services:** `apply` saves services it has not seen, then sets every enabled one on the run's port, so F1, F2 and F6 share one tested path.
- **State shown after the supervisor restored:** a `Set` whose saved file is gone shows as off.
- **Screenshots:** the new Settings section and the Home chip were checked in the demo (a `systemProxy` demo option was added) but the published screenshot set was not regenerated in this unit.

## Interfaces added
`sysproxy` module; `ReliabilitySettings.system_proxy`, `bypass`; `Status.system_proxy`; Tauri command `repair_system_proxy`; supervisor gains an optional sixth argument (restore script path); two private files in the data folder.

## Out of scope
TUN mode (it captures traffic already), proxy login passed to the system proxy, per-service choice, a PAC file served by Kage.
