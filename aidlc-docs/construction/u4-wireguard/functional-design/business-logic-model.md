# U4 WireGuard: business logic model

## F1. Text to servers (import box, subscription body)
```
text
 |
 +-- sing-box JSON?  -> outbounds[] (incl. old type:wireguard) + endpoints[] (type:wireguard)
 |                      each entry -> share link (formats.rs) -> protocol parse
 +-- Clash YAML?     -> proxies[] (type:wireguard) -> share link -> protocol parse
 +-- wg-quick text?  -> WgQuick (new, in the WireGuard module)
 |                      -> refuse (R3) or -> profile + optional note (R4), name per R5
 +-- otherwise       -> split links -> wireguard:// and wg:// -> protocol parse
```
- The wg-quick check runs after the JSON and YAML checks and before link splitting. A `.conf` holds `Key = value` lines, which neither JSON nor YAML readers accept as a server list, so the order cannot misread one as another.
- Notes join the existing `Parsed.notes`.

## F2. A dropped file
The Add server dialog already reads a dropped file's text into the box. It now also keeps the file's name; on Import, the name (without extension) goes with the text to `import_text` as an optional `file_name`. Only the wg-quick reader uses it (R5). Pasting text clears it.

## F3. Profile to config entry
```
for each node (up to MAX_NODES, chosen one first):
    def = protocol of node
    if def.feature not in engine tags (and tags known): leave out (F5)
    entry = def.entry(node) + tag
    if def.kind == Endpoint: endpoints += entry else outbounds += entry
proxy group: selector or urltest over every kept node tag
outbounds += proxy group, direct
config.endpoints = endpoints (only when not empty)
```
The WireGuard entry:
```
{ "type": "wireguard", "tag": <node tag>, "system": false, "mtu": <mtu>,
  "address": [<addresses>], "private_key": <privateKey>,
  "peers": [{ "address": <server>, "port": <port>, "public_key": <peerPublicKey>,
              "pre_shared_key"?: .., "allowed_ips": ["0.0.0.0/0", "::/0"],
              "persistent_keepalive_interval"?: .., "reserved"?: [a, b, c] }] }
```
`allowed_ips` is always everything: what goes through the tunnel is decided by Kage's routing, which picks the node; WireGuard's own filter would only drop traffic.

## F4. Shared server code that assumed outbounds
- `ProtocolDef::outbound` becomes the server's config entry for either kind (renamed in code to say so, or kept with its doc changed; decided in code generation).
- The conformance test "the outbound has an address and no tag" reads the address from `server`/`server_port`, or from `peers[0].address/port` for endpoints.
- `every_protocol_round_trips_through_a_singbox_config` puts endpoint entries under `endpoints`.

## F5. Connect with engine features
```
tags = engine tags (CoreInfo.tags), or unknown
missing(node) = def(node).feature is set and tags known and feature not in tags
if missing(chosen): stop with R9 message (before sing-box starts, before any privileged step)
nodes = nodes without missing ones; log one line naming how many were left out and why
```
Auto mode (urltest) with the chosen server missing: same stop, since the user chose it.

## F6. Latency and pre-connect
Unchanged code paths: pre-connect skips TCP for `over_udp`; latency uses the tag. Verified at runtime (R11).

## F7. Editing
Editor saves go through `normalize` then `validate`: keys trimmed, addresses given prefixes, empty optional values cleared, reserved written as three numbers, MTU filled with 1408 when empty.
