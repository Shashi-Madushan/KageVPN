# More protocols and system-proxy mode: requirement questions

Round 4 (branch `feature/protocols-and-system-proxy`). Please fill in each `[Answer]:` tag with a letter, or with X and your own words. Options marked *(recommended)* are my suggestion. Add notes under any answer if you like.

## Background (what the choices depend on)

**Protocols today.** Each protocol is one module in `src-tauri/src/profile/` implementing `ProtocolDef` (see `docs/ADDING-A-PROTOCOL.md`). `profile/links.rs` lists `tuic`, `hysteria`, `wireguard`, `wg` and `ssh` in `UNSUPPORTED`, so the import says "not supported yet". `profile/formats.rs` does the same for those types inside sing-box JSON and Clash YAML subscriptions.

**What sing-box can run:**
- **TUIC**: v5 only (QUIC, UUID + password, congestion control, UDP relay mode). v4 links cannot run.
- **Hysteria v1**: needs upload and download speeds (`up_mbps`, `down_mbps`); optional `auth_str` and `obfs`. Superseded by Hysteria2 upstream but still in sing-box.
- **WireGuard**: since sing-box 1.11 WireGuard is an **endpoint**, not an outbound, and the old outbound form was removed in 1.13. Today `ProtocolDef::outbound` and the config builder assume every server is an outbound, so WireGuard changes that shared code. There is no single standard link format: `wireguard://` links (v2rayN, Hiddify style) and wg-quick `.conf` files are both common. AmneziaWG options (`Jc`, `Jmin`, ...) cannot run in sing-box.
- **SSH**: user + password or private key, optional host key pinning (`host_key`).

**System proxy today.** Proxy Only mode starts sing-box with a local mixed (HTTP + SOCKS) listener on 127.0.0.1, port 2080 or a free one if that is taken, with an optional login (`kage` + password). Nothing touches macOS network settings; each app has to be pointed at the proxy. macOS stores proxy settings per network service (Wi-Fi, Ethernet, ...) and they can be changed with `networksetup` or `SCPreferences`. If Kage turns them on and then dies without undoing it, every app that follows the system proxy loses its connection.

**Your dev Mac runs your real Kage VPN on ports 2080 and 19090.** Testing a system proxy changes this Mac's real network settings (see Question 17).

---

## Part A: Protocols

## Question 1
Which protocols are in this round?

A) All four: TUIC, Hysteria v1, WireGuard, SSH *(recommended)*

B) TUIC, Hysteria v1 and SSH now; WireGuard later (it needs the endpoint change in shared code)

C) Only TUIC and WireGuard (most common in subscriptions)

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 2
Should the new protocols also be read from sing-box JSON and Clash YAML subscriptions, not only from share links?

A) Yes, links and both subscription formats *(recommended)*

B) Links only for now

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 3
How should WireGuard servers come in?

A) `wireguard://` links and pasted or dropped wg-quick `.conf` files *(recommended)*

B) `wireguard://` links only

C) `.conf` files only

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 4
Which WireGuard features must work?

A) One peer, pre-shared key, MTU, IPv4/IPv6 addresses and `reserved` bytes (needed for Cloudflare WARP); reject multi-peer configs and AmneziaWG options with a clear message *(recommended)*

B) The same plus multiple peers

C) The basics only (one peer, keys, address, MTU)

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 5
A Hysteria v1 link often has no upload/download speed, but sing-box requires both. What should happen?

A) Import it with defaults (e.g. 10 Mbps up, 50 Mbps down), say so in the import result, and let the editor change them *(recommended)*

B) Reject the link and say the speeds are missing

C) Import it and mark it as needing edits before it can connect

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 6
How should SSH servers sign in?

A) Password or a private key pasted into the editor (with optional passphrase) *(recommended)*

B) Password only

C) Password, a pasted key, or a path to a key file on disk

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 7
How should SSH check the server's identity?

A) Optional host key field; when empty, connect anyway and show a warning in the editor and server list *(recommended)*

B) Host key required; refuse to save without it

C) Trust on first use: record the key on the first connect and refuse later if it changes

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 8
Saved servers keep their passwords in the app's profile store today (the same file as VLESS UUIDs and Trojan passwords). Is that fine for WireGuard private keys and SSH private keys?

A) Yes, same store as today; feedback reports hide them through `secrets()` *(recommended)*

B) No, move these keys (and only these) into the macOS Keychain

C) No, move all server secrets into the Keychain (larger change)

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 9
TUIC v4 links and other options sing-box cannot run (for example Hysteria port hopping, AmneziaWG): what should happen?

A) Reject that server with a clear reason, as the existing protocols do *(recommended)*

B) Import it and drop the option silently

X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Part B: System-proxy mode

## Question 10
How is it turned on?

A) A "Set system proxy" toggle for Proxy Only mode, off by default; when on, Kage sets it on connect and clears it on disconnect *(recommended)*

B) On by default in Proxy Only mode

C) A separate third mode beside Proxy Only and TUN

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 11
Which macOS proxy types should be set?

A) Web (HTTP), Secure Web (HTTPS) and SOCKS, all pointing at the local listener *(recommended)*

B) Web and Secure Web only (many apps handle SOCKS poorly)

C) SOCKS only

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 12
Which network services?

A) Every enabled service (Wi-Fi, Ethernet, Thunderbolt Bridge, ...), so switching networks keeps the proxy *(recommended)*

B) Only the service in use when connecting

C) A list the user picks in Settings

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 13
When the local proxy login (password) is on, apps that follow the system proxy often ignore the stored login and fail. What should happen?

A) The system proxy toggle is unavailable while the login is on, with a note saying why *(recommended)*

B) Set the system proxy with the login anyway and warn that some apps will fail

C) Turning on the system proxy turns the login off

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 14
Which addresses skip the proxy (the bypass list)?

A) A built-in list (localhost, 127.0.0.1, *.local, 169.254/16, 10/8, 172.16/12, 192.168/16) plus extra entries the user can add in Settings *(recommended)*

B) The built-in list only

C) Keep whatever bypass list the user already has

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 15
If the Mac already has a proxy configured (for example a work PAC file) when connecting, what should happen?

A) Save it, replace it while connected, and put it back exactly on disconnect *(recommended)*

B) Refuse to turn on the system proxy and tell the user

C) Replace it and leave it cleared afterwards

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 16
How far should the "never leave a dead proxy" guarantee go?

A) Save the previous settings to disk before changing anything; restore on disconnect and quit; restore at the next launch if the save is still there; and have the existing proxy-mode supervisor restore them when the app or sing-box dies *(recommended)*

B) Restore on disconnect and quit, plus at the next launch (a crash leaves the proxy set until Kage is opened again)

C) Restore on disconnect and quit only

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 17
How should I test the system proxy on your Mac, which runs your real Kage VPN?

A) On this Mac, on a spare port, from a separate test build; I save your current proxy settings first and check they are back afterwards *(recommended)*

B) Only in a separate macOS user account or a VM

C) Automated tests only; you check the real toggle yourself

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 18
If changing the system proxy asks for an administrator password on your Mac, what is acceptable?

A) Use the optional root service when installed; otherwise one administrator prompt per connect *(recommended)*

B) Any number of prompts is fine

C) No prompts: if it needs one, show instructions for setting the proxy by hand instead

X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Part C: Delivery

## Question 19
How should this land?

A) One branch, two pull requests in order: protocols first, then system proxy *(recommended)*

B) One pull request for everything

C) One pull request per protocol plus one for system proxy

X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Part D: Extensions (earlier rounds chose No for all three)

## Question: Security Extensions
Should security extension rules be enforced for this project?

A) Yes — enforce all SECURITY rules as blocking constraints (recommended for production-grade applications)

B) No — skip all SECURITY rules (suitable for PoCs, prototypes, and experimental projects)

X) Other (please describe after [Answer]: tag below)

[Answer]: B (extension: same as earlier rounds)

## Question: Property-Based Testing Extension
Should property-based testing (PBT) rules be enforced for this project?

A) Yes — enforce all PBT rules as blocking constraints (recommended for projects with business logic, data transformations, serialization, or stateful components)

B) Partial — enforce PBT rules only for pure functions and serialization round-trips (suitable for projects with limited algorithmic complexity)

C) No — skip all PBT rules (suitable for simple CRUD applications, UI-only projects, or thin integration layers with no significant business logic)

X) Other (please describe after [Answer]: tag below)

[Answer]: C (extension: same as earlier rounds)

## Question: Resiliency Extensions
Should the resiliency baseline be applied to this project?

A) Yes — apply the resiliency baseline as directional best practices and design-time guidance

B) No — skip the resiliency baseline (suitable for PoCs, prototypes, and experimental projects where rapid iteration matters more than reliability)

X) Other (please describe after [Answer]: tag below)

[Answer]: B (extension: same as earlier rounds)
