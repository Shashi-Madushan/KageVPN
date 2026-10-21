# Requirements: more protocols and system-proxy mode

## Intent analysis
- **Request:** add TUIC, WireGuard, Hysteria v1 and SSH (today reported as "not supported yet"), and a system-proxy option for Proxy Only mode with a restore step so a crash never leaves a dead proxy configured. Network Extension mode is parked for later.
- **Type:** New features (brownfield). **Clarity:** clear after the questions. **Scope:** multiple components (profile modules, link and subscription import, config builder, runner supervisor, settings, Settings UI). **Complexity:** moderate to complex (WireGuard changes shared config code; the system proxy changes macOS network settings). **Depth:** standard.
- Answers: `protocols-and-system-proxy-questions.md`. The user asked for the recommended answers on every question (A for Questions 1 to 19). The extension questions had no recommendation, so they follow the earlier rounds: Security No, Property-Based Testing No, Resiliency No. Answers checked for contradictions: none found. One open point is listed under "To verify in design".

## What exists and is reused
The protocol registry (`profile/mod.rs`, `ProtocolDef`, `REGISTRY`), shared TLS and transport settings and form groups (`profile/common.rs`, `profile/form.rs`), link parsing (`profile/links.rs`), sing-box JSON and Clash YAML subscriptions (`profile/formats.rs`), the conformance suite and real `sing-box check` (`tests/profile_conformance.rs`), the screenshot fixture (`scripts/screenshots/protocols.json`), the config builder (`config.rs`), the proxy-mode supervisor (`runner/supervisor.sh`, which already stops sing-box when the app exits), the optional root service, and the local proxy login setting.

## Functional requirements

### FR1. Protocol modules (all four in this round)
Each protocol is one `ProtocolDef` module with parse, link writing, validation, outbound, identity, secrets, summary, editor form, blank and sample, listed in `REGISTRY`, following `docs/ADDING-A-PROTOCOL.md`. Its scheme leaves `UNSUPPORTED` in `links.rs`.

- **FR1.1 TUIC (`tuic://`)**: v5 only. UUID, password, TLS (SNI, ALPN, insecure flag), congestion control (cubic, new_reno, bbr), UDP relay mode (native, quic), zero-RTT handshake. A v4 link (token, no UUID/password) is rejected with a reason.
- **FR1.2 Hysteria v1 (`hysteria://`)**: auth string, upload and download speed in Mbps, obfs password, TLS (SNI, ALPN, insecure flag). A link without speeds is imported with defaults (10 Mbps up, 50 Mbps down); the import result says the defaults were used and the editor can change them. Port hopping (`mport` or a port range) is rejected with a reason.
- **FR1.3 WireGuard (`wireguard://`, `wg://`)**:
  - Runs as a sing-box **endpoint**, not an outbound. The config builder puts WireGuard servers in `endpoints` with the node's tag, so selector, urltest, routing and every feature that refers to a node tag keep working unchanged.
  - Fields: private key, peer public key, pre-shared key, local IPv4 and IPv6 addresses, MTU, `reserved` bytes (needed for Cloudflare WARP).
  - Also imported from a wg-quick `.conf` file, pasted into the import box or dropped on it.
  - Rejected with a reason: more than one peer, AmneziaWG options (`Jc`, `Jmin`, `Jmax`, `S1`, `S2`, `H1` to `H4`).
- **FR1.4 SSH (`ssh://`)**: user, password or a private key pasted into the editor with an optional passphrase, optional host key. With no host key the server connects and the editor and the server list show a warning that the server's identity is not checked.
- **FR1.5 Rejections:** anything sing-box cannot run is rejected per server with a clear reason, as the existing protocols do; nothing is dropped silently.
- **FR1.6 Subscriptions:** sing-box JSON (`tuic`, `hysteria`, `wireguard` endpoints and outbounds, `ssh`) and Clash YAML (`tuic`, `hysteria`, `wireguard`, `ssh`) entries are read with the same rules as links, instead of "servers are not supported yet".
- **FR1.7 Secrets:** passwords, auth strings, the WireGuard private and pre-shared keys and the SSH private key and passphrase stay in the existing profile store and are listed in `secrets()` so feedback reports hide them.
- **FR1.8 Interface:** no front-end code changes beyond what the registry already provides (picker, editor fields, schemes, list summaries). Multi-line fields (SSH private key) use or add a textarea field kind if the form does not have one.

### FR2. System-proxy option (Proxy Only mode)
- **FR2.1** A "Set system proxy" toggle for Proxy Only mode, off by default. When on, Kage sets the macOS system proxy after the connection is verified and clears it on disconnect. The toggle does nothing in TUN mode.
- **FR2.2** Sets Web (HTTP), Secure Web (HTTPS) and SOCKS proxies, all to 127.0.0.1 and the port the current connection uses (which may not be 2080 when that port was taken).
- **FR2.3** Applies to every enabled network service, so moving between Wi-Fi and Ethernet keeps the proxy. A service enabled while connected is set too.
- **FR2.4** While the local proxy login is on, the toggle is unavailable with a note saying apps that follow the system proxy usually ignore the login.
- **FR2.5** Bypass list: a built-in list (localhost, 127.0.0.1, ::1, *.local, 169.254/16, 10/8, 172.16/12, 192.168/16) plus extra entries the user adds in Settings.
- **FR2.6** Existing proxy settings (including an automatic PAC URL and its own bypass list) are saved before anything changes, replaced while connected, and put back exactly on disconnect.
- **FR2.7 Restore guarantee:**
  - The saved settings are written to disk before the first change.
  - Restored on disconnect and on quit.
  - Restored by the proxy-mode supervisor when the app exits or dies, and when sing-box dies on its own.
  - Restored at the next launch if the saved file is still there (for example after a power loss or a killed supervisor).
  - The saved file is deleted only after a successful restore.
- **FR2.8 Changing mode or turning the toggle off while connected** restores the settings immediately; turning it on while connected sets them immediately.
- **FR2.9** The Home page shows that the system proxy is set while it is, and a failed set or restore is shown with a Repair action that restores the saved settings.

## Non-functional requirements
- **NFR1 Testing:** a `tests/profile_<name>.rs` file per protocol for its own edge cases (odd links, rejected links, `.conf` parsing); the conformance suite and the real `sing-box check` cover round-trips, identity, redaction, editor keys and outbound or endpoint shape; the screenshot fixture is regenerated. The system-proxy logic that builds commands and decides what to restore is pure Rust with unit tests; shell parts get script tests where the project already has them.
- **NFR2 Live machine safety:** the user's real Kage VPN runs on ports 2080 and 19090 on the dev Mac. System-proxy tests on this Mac use a spare port and a separate test build; the current proxy settings of every service are saved before the test and checked afterwards to be identical.
- **NFR3 Privileges:** use the optional root service when it is installed; otherwise at most one administrator prompt per connect. If no prompt is needed on an admin account, none is shown.
- **NFR4 Compatibility:** the WireGuard endpoint needs sing-box 1.11 or later; when the installed engine is older, connecting with a WireGuard server says so instead of failing in sing-box.
- **NFR5 Docs:** `docs/ADDING-A-PROTOCOL.md` supported table, the user guide (protocols, Proxy Only mode), README features and the changelog.
- **NFR6 Delivery:** one branch, two pull requests in order: protocols first, then system proxy.

## Out of scope
Network Extension mode (Round 3, parked), multi-peer WireGuard, AmneziaWG, Hysteria port hopping, TUIC v4, moving secrets to the Keychain, SSH key files on disk, a separate third connection mode.

## To verify in design
- Whether `networksetup -setwebproxy` and friends need an administrator password on an admin account on the current macOS. This decides how often NFR3's prompt appears and whether the supervisor (running as the user) can restore without the root service.
- Whether the supervisor's restore can reuse the same commands file the app writes, so the shell side holds no logic of its own.
