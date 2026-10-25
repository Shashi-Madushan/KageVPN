# U4 WireGuard: Functional Design Plan
Covers FR1.3, FR1.5, FR1.6 (WireGuard part) and NFR4. Business logic only; code comes after approval and a code generation plan.

## What the code showed (inputs to the design)
- **One place builds server entries.** `config::build` turns every node into an outbound through `Profile::outbound` (the only caller) and lists the node tags in the `proxy` selector or urltest. Nothing else in the app reads the `outbounds` array.
- **WireGuard is an endpoint.** Since sing-box 1.11 it goes in a separate top-level `endpoints` array; the old `wireguard` outbound was removed in 1.13. Endpoint tags can be used wherever an outbound tag can (selector, urltest, route rules).
- **Engine version is already safe.** The app refuses sing-box older than `1.12.0` (`app_config::SING_BOX.min_version`), so NFR4 ("say so when the engine is older than 1.11") cannot happen and needs no new check.
- **Engine build features are not.** WireGuard needs a sing-box built `with_wireguard`; TUIC and Hysteria need `with_quic`. The app only requires `with_clash_api` and `with_utls`, so an engine without these would fail at `sing-box check` with sing-box's own wording. Official release builds and Homebrew include both (your 1.14.2 has them).
- **The conformance suite** expects every server's config entry to have `server` and `server_port` at the top. A WireGuard endpoint has them inside `peers[0]` (`address`, `port`).
- **Subscriptions:** `formats.rs` reads only the `outbounds` of a sing-box file. WireGuard servers there sit in `endpoints` (new form) or `outbounds` with `type: wireguard` (old form).
- **Import of files:** dropping a file in the Add server dialog reads its text into the box (`ImportDialog.loadFile`); the file name is not passed on.
- **Pre-connect check:** WireGuard is UDP, so `over_udp` is true and the TCP test is skipped.

## Proposed design in short
WireGuard servers are profiles like any other (`protocol: "wireguard"`). A protocol says whether its server is an outbound or an endpoint; `config::build` puts endpoint servers in `endpoints` with the same node tag, and the selector, urltest, latency tests and routing use the tag unchanged. Servers come from `wireguard://` links, wg-quick `.conf` text (pasted or dropped), and sing-box or Clash subscriptions. One peer only.

## Steps
- [x] 1. Answer the questions below.
- [x] 2. Check the answers for contradictions or vague wording; ask follow-ups if needed.
- [x] 3. Write `aidlc-docs/construction/u4-wireguard/functional-design/domain-entities.md` (WireGuard settings, the endpoint kind, the parsed `.conf`).
- [x] 4. Write `business-rules.md` (what is accepted, defaults, what is rejected or noted, identity, engine features).
- [x] 5. Write `business-logic-model.md` (link, `.conf` and subscription to profile; profile to endpoint; config assembly; latency and pre-connect behaviour).
- [x] 6. Write `frontend-components.md` (editor fields; `.conf` drop and its name; import notes).
- [x] 7. Present for approval.

## Questions

## Question 1
How should a WireGuard server go into the sing-box config?

A) A protocol declares itself an outbound or an endpoint; `config::build` puts endpoint servers in `endpoints` with the usual node tag, and the selector, urltest and routing refer to it unchanged *(recommended)*

B) Special-case WireGuard by name in `config::build`

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 2
WireGuard needs a sing-box built `with_wireguard`, TUIC and Hysteria need `with_quic`. What should happen with an engine that lacks one?

A) Each protocol names the build feature it needs. Servers whose feature is missing are left out of the config; if the chosen server is one of them, connecting stops with "This sing-box was built without WireGuard support; install the official build on the Engine page" *(recommended)*

B) Add both features to the engine's required list, so such an engine is refused for every protocol

C) Nothing: sing-box's own check error is shown

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 3
What should the server be called when a wg-quick `.conf` is imported?

A) The file name without `.conf` when a file is dropped (for example `warp-tokyo`); the `Endpoint` host when the text is pasted *(recommended)*

B) Always the `Endpoint` host

C) Always "WireGuard", renamed by the user afterwards

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 4
A `.conf` often has lines Kage does not use: `DNS`, `AllowedIPs` narrower than everything, `Table`, `PostUp` and friends. What should happen?

A) Import the server; ignore those lines and add an import note naming them ("DNS and AllowedIPs from the file are not used: Kage's routing and DNS settings decide") *(recommended)*

B) Import silently, ignoring them

C) Reject a file whose `AllowedIPs` is not `0.0.0.0/0, ::/0`

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 5
Which defaults when a link or file leaves a value out?

A) MTU 1408 (sing-box's default); no keepalive; no `reserved` bytes; addresses must be given (a server without an `Address` is rejected) *(recommended)*

B) MTU 1280 (safe on every network, slightly slower); the rest as A

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 6
What makes two WireGuard entries the same server across a subscription refresh?

A) Server, port and the peer's public key *(recommended)*

B) The same plus the local address (a provider that hands out a new address each refresh would then look like a new server)

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 7
sing-box files from before 1.11 list WireGuard as an outbound (`type: wireguard` in `outbounds`). Should those be read?

A) Yes: read both the old outbound form and the new `endpoints` form, and always write the new one *(recommended)*

B) Only the new `endpoints` form; the old form is skipped with a reason

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)

## Question 8
How should the editor show a WireGuard server?

A) Connection: private key (secret), peer public key, local addresses (list). Security: pre-shared key (optional secret). Advanced: MTU, keepalive seconds, reserved bytes (optional, with a hint that Cloudflare WARP needs them) *(recommended)*

B) Everything in Connection

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended, chosen by the user)
