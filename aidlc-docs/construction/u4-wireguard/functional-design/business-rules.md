# U4 WireGuard: business rules

## R1. Sources accepted
1. **Links:** `wireguard://` and `wg://`, in the v2rayN / Hiddify form:
   `wireguard://<private key>@host:port?publickey=..&address=..&mtu=..&reserved=..&presharedkey=..&keepalive=..#name`.
   Parameter names also read: `publicKey`, `public_key`, `peer_public_key`; `ip` and `local_address` for `address`; `preSharedKey`, `pre_shared_key`, `psk`; `persistentKeepalive`. The private key may also be a `privatekey` parameter instead of the user part. Base64 keys are taken percent-decoded, and a space where a link had a raw `+` is turned back into `+`.
2. **wg-quick text:** text whose first section header is `[Interface]` and that has a `[Peer]` section, pasted or dropped. Recognised before link splitting, like Clash and sing-box files.
3. **sing-box files:** `endpoints` entries with `type: wireguard` (1.11 and later) and `outbounds` entries with `type: wireguard` (before 1.11).
4. **Clash files:** `type: wireguard` proxies (`private-key`, `public-key`, `pre-shared-key`, `ip`, `ipv6`, `mtu`, `reserved`, `persistent-keepalive`, and the `peers:` list form).

## R2. Values and defaults
- `server` and `port` come from the peer's endpoint (`Endpoint = host:port` in a `.conf`, the link's host and port, `peers[0].address/port` in sing-box). A server without an endpoint is refused: "the file has no Endpoint for its peer".
- MTU: the given value, else **1408**. Valid 1280 to 1500.
- Keepalive: the given value, else off.
- Reserved: absent unless given. Accepted as three numbers (`1,2,3`, `[1, 2, 3]`) or a base64 string of exactly three bytes (`AQID`).
- Addresses: required. Several are allowed; each gets a prefix if it had none (`/32`, `/128`).
- Keys: each must be base64 of exactly 32 bytes; otherwise "private key is not a WireGuard key" (or public, pre-shared).

## R3. Refused, each with a reason (FR1.5)
| Case | Message |
|---|---|
| More than one `[Peer]`, or more than one entry in `peers` | "WireGuard configs with more than one peer are not supported" |
| AmneziaWG keys (`Jc`, `Jmin`, `Jmax`, `S1`, `S2`, `H1` to `H4`) | "AmneziaWG options (Jc, ...) are not supported; sing-box runs plain WireGuard" |
| No `Address` | "WireGuard needs this device's address (Address)" |
| No peer endpoint | "the file has no Endpoint for its peer" |
| A key that is not 32 bytes of base64 | "<which> key is not a WireGuard key" |
| Reserved bytes not three values 0 to 255 | "reserved must be three numbers from 0 to 255" |
| MTU outside 1280 to 1500 | "MTU must be between 1280 and 1500" |

## R4. Imported, with a note (Question 4)
Lines Kage does not use do not block the import. One note per server lists what was ignored, for example:
"warp-tokyo: DNS, AllowedIPs and PostUp from the file are not used; Kage's DNS and routing settings decide".
- `AllowedIPs` is named only when it is narrower than `0.0.0.0/0` plus `::/0` (or `0.0.0.0/0` alone).
- `DNS` is named whenever present.
- Every other unused key is named (`Table`, `PostUp`, `PostDown`, `PreUp`, `PreDown`, `SaveConfig`, `FwMark`, `ListenPort`).

## R5. Name (Question 3)
- Dropped `.conf` file: the file name without `.conf`.
- Pasted `.conf` text: a `# Name = ...` comment line if the file has one (common in exports), else the endpoint host.
- Link: the `#fragment`, else the host (as every protocol).
- Subscriptions: the entry's tag or name.

## R6. Identity (Question 6)
Server, port and the peer public key. A provider that hands out a new local address on refresh keeps the same server.

## R7. Secrets
The private key and the pre-shared key are hidden from feedback reports. The peer public key and addresses are not secrets (addresses are already masked by the report's IP masking).

## R8. Config placement (Question 1)
- A server whose protocol is an `Endpoint` goes in the config's top-level `endpoints` array, built by the protocol, with the node tag added by the caller (as for outbounds).
- The `proxy` selector or urltest lists every node tag, endpoint or outbound, in the same order as today.
- `endpoints` is only written when there is at least one endpoint server.
- The endpoint always runs in user space (`system: false`) so it needs no extra privileges in Proxy Only mode.

## R9. Engine features (Question 2)
- Before building the config, nodes whose protocol needs a build feature the engine lacks are left out.
- If the **chosen** server is one of them, connecting stops before sing-box starts with: "This sing-box was built without <WireGuard|QUIC> support, which <protocol> servers need. Install the official build on the Engine page." Other servers left out are listed in the connection log, not shown as an error.
- If every server is left out, the same error is shown for the chosen one.
- When the engine's tags are unknown (inspect failed), nothing is left out; sing-box's own check decides.

## R10. Engine version (NFR4)
No new check: the app already refuses sing-box older than 1.12.0, and endpoints exist since 1.11.

## R11. Health
- `over_udp` is true: the pre-connect TCP test is skipped; the name is still resolved.
- Latency tests go through the Clash API by tag. sing-box registers endpoints as outbounds, so the delay test works on the tag. This is checked at runtime during code generation (a spare-port sing-box run with a WireGuard endpoint and a `/proxies/<tag>/delay` call); if it does not, latency for WireGuard shows "not available" instead of a timeout.

## R12. Export
- `to_link` writes the `wireguard://` form with every set value.
- The "copy as sing-box" round trip (`every_protocol_round_trips_through_a_singbox_config`) writes an `endpoints` entry and reads it back to the same server.
