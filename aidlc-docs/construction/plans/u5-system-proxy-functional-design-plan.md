# U5 System proxy: Functional Design Plan
Round 4, pull request 2 (branch `feature/system-proxy`, cut from pull request 1's branch). Covers FR2.1 to FR2.9 and NFR2, NFR3. Business logic only; code comes after approval and a code generation plan.

## What the code and the Mac showed (inputs to the design)
- **No password is needed.** macOS guards network settings with the right `system.services.systemconfiguration.network`, which allows root, an admin who types a password, or an *entitled* program. Apple's `/usr/sbin/networksetup` carries the entitlement for exactly that right (`com.apple.private.AuthorizationServices`). So `networksetup` can set and restore proxies as a normal user without any prompt, and so can the user-level supervisor. This settles the open point from the requirements and NFR3 needs no root service and no prompt. (Read with `security authorizationdb read` and `codesign -d --entitlements`, nothing changed; to be confirmed for real during code generation.)
- **Services on this Mac:** Mobile ECM Network Adapter, USB 10/100 LAN, Thunderbolt Bridge, Wi-Fi, iPhone USB (none disabled). Wi-Fi and USB LAN currently have no proxies and no PAC.
- **Per service, macOS keeps:** Web (HTTP) proxy, Secure Web (HTTPS) proxy, SOCKS proxy (each on/off, host, port, and an authenticated flag), an automatic proxy (PAC) URL with its own on/off, auto-discovery (WPAD), and one bypass list shared by all of them.
- **Proxy Only runs** start a user-level supervisor (`runner/supervisor.sh`) that runs sing-box until a stop file appears or **the app's process goes away**; then it stops sing-box. So after a crash of the app, sing-box is gone too and a system proxy left set would point at nothing: the case to prevent.
- **Ports move:** the local proxy uses 2080 or, when taken, a free port (`net::choose_ports`); the current one is `config::mixed_port()` and `Status.proxy_port`.
- **Connect flow:** sing-box starts, `wait_until_ready` waits for its ports, the status turns Connected with health Verifying, and the health monitor marks it Healthy or not. The watchdog and auto-reconnect restart runs (`disconnect_for_restart`, then connect).
- **Network changes** are already watched (`network_watch.rs`, interface fingerprints), so a newly enabled service can be noticed.
- **Proxy login:** `ReliabilitySettings.proxy_auth` and `proxy_password` (user `kage`).

## Proposed design in short
A new `sysproxy` component owns the macOS proxy settings. On connect in Proxy Only mode with the toggle on, it first **saves** every enabled service's current proxy settings to a file, then **sets** Web, Secure Web and SOCKS to `127.0.0.1:<port>` with the bypass list, and writes a **restore script** of `networksetup` commands next to the saved file. On disconnect, quit, mode change or toggle off it **restores** and deletes both files. The supervisor runs the restore script if it ends without a stop request (app or sing-box died). At launch, a saved file that is still there is restored. Home shows when the system proxy is set, and offers Repair if a set or restore failed.

## Steps
- [x] 1. Answer the questions below.
- [x] 2. Check the answers for contradictions or vague wording; ask follow-ups if needed.
- [x] 3. Write `aidlc-docs/construction/u5-system-proxy/functional-design/domain-entities.md` (service proxy state, snapshot, restore script, settings).
- [x] 4. Write `business-rules.md` (when to set and restore, what is restored, bypass list, login, ports, new services, failures).
- [x] 5. Write `business-logic-model.md` (connect, restart, disconnect, quit, crash, launch, network change, toggle and mode changes).
- [x] 6. Write `frontend-components.md` (Settings toggle and bypass list, Home indicator and Repair, API).
- [x] 7. Present for approval.

## Questions

## Question 1
When should the system proxy be set during a connect?

A) As soon as sing-box is listening on its port (when the status turns Connected), so apps start using it straight away; a failed health check does not undo it, since the watchdog handles a bad tunnel *(recommended)*

B) Only after the health check passes (Healthy), so a broken tunnel never captures traffic; apps go direct until then

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 2
What happens during a restart (watchdog, auto-reconnect, network change, routing change)?

A) Keep the system proxy set through the restart and only update the port if the new run uses another one; restore only when the user disconnects or quits, or when reconnecting gives up *(recommended)*

B) Restore on every stop and set again on every start (apps may briefly go direct, and settings change several times)

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 3
How should the supervisor restore the settings when the app dies?

A) The app writes a small restore script of `networksetup` commands when it sets the proxy; the supervisor runs it when it ends without a stop request (the app or sing-box died), then deletes it. The shell side holds no logic of its own *(recommended)*

B) The supervisor does nothing; restoring waits for the next launch

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 4
If you change a service's proxy settings yourself while connected (for example turn the proxy off in System Settings), what should the restore do?

A) Restore only what still points at Kage (`127.0.0.1:<Kage's port>`); leave anything you changed since alone *(recommended)*

B) Always put back the saved settings exactly, overwriting your change

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 5
A service that becomes enabled while connected (for example plugging in Ethernet):

A) Is noticed through the existing network watcher; its settings are saved and the proxy is set on it too *(recommended)*

B) Is left alone until the next connect

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 6
Where do the settings live in the interface?

A) Settings, in the section with the local proxy login: "Set system proxy (Proxy Only mode)" toggle, then the extra bypass entries (one per line, domains or address ranges); Home shows "System proxy on" while it is set *(recommended)*

B) On Home, next to the mode switch

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 7
Turning the toggle on, or switching to Proxy Only, while already connected:

A) Applies at once (set, or restore when turning off or leaving Proxy Only), with no reconnect *(recommended)*

B) Applies at the next connect, with a note saying so

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 8
How should the real proxy settings on your Mac be touched during code generation (NFR2)?

A) Only on one service that is not in use (for example "iPhone USB"), with its settings saved before and compared after, using spare ports, never 2080 or 19090; I ask before the first change *(recommended)*

B) On Wi-Fi as well, with the same save and compare (closer to real use, but Wi-Fi is what your Mac is using now)

C) Not at all; automated tests only

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)
