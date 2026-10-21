# U4 WireGuard: domain entities

Answers used: `construction/plans/u4-wireguard-functional-design-plan.md` (all A, recommended).

## WireGuard server (a profile with `protocol: "wireguard"`)
The shared profile fields keep their meaning: `name`, `server` (the peer's host), `port` (the peer's UDP port), `group`, `favorite`, `subscriptionId`.

`WireGuardSettings` (saved inside the profile, camelCase):

| Field | Type | Required | Meaning |
|---|---|---|---|
| `privateKey` | text (base64, 32 bytes) | yes | This device's private key. Secret. |
| `peerPublicKey` | text (base64, 32 bytes) | yes | The server's public key. Public. |
| `preSharedKey` | text (base64, 32 bytes) | no | Extra symmetric key. Secret. Empty when unused. |
| `addresses` | list of CIDR prefixes | yes, at least one | This device's addresses inside the tunnel, such as `172.16.0.2/32` and `fd01::2/128`. A bare address gets `/32` (IPv4) or `/128` (IPv6). |
| `mtu` | number | yes (default 1408) | Tunnel MTU, 1280 to 1500. |
| `keepalive` | number (seconds) | no (0 = off) | `PersistentKeepalive`, 0 to 65535. |
| `reserved` | three numbers 0 to 255, or none | no | The three reserved header bytes Cloudflare WARP uses to tell clients apart. |

Optional text fields read `null` from the editor as empty (`common::null_as_empty`), as for Hysteria and SSH.

## Server kind (new, on every protocol)
`ServerKind`: `Outbound` (every protocol so far) or `Endpoint` (WireGuard). Decides which top-level array of the sing-box config the server goes in. Declared by each protocol, like `over_udp`, so a new endpoint protocol later needs no change to the config builder.

## Required build feature (new, on every protocol)
Each protocol names the sing-box build feature it needs, or none:

| Protocol | Feature |
|---|---|
| VLESS, VMess, Trojan, Shadowsocks, SSH | none beyond the engine's required list |
| Hysteria2, Hysteria, TUIC | `with_quic` |
| WireGuard | `with_wireguard` |

The engine's reported tags already come from `core::inspect` (`CoreInfo.tags`).

## Parsed wg-quick file (import only, not saved)
`WgQuick { interface: { private_key, addresses[], dns[], mtu?, other_keys[] }, peers: [ { public_key, preshared_key?, endpoint?, allowed_ips[], keepalive?, other_keys[] } ], name_hint? }`
- `other_keys`: keys Kage does not use (`Table`, `PostUp`, `PreDown`, `SaveConfig`, `FwMark`, `ListenPort`, ...), kept only to name them in the import note.
- `name_hint`: the dropped file's name without `.conf`, when there was one.
- `Jc`, `Jmin`, `Jmax`, `S1`, `S2`, `H1` to `H4` (AmneziaWG) are recognised by name so they can be refused.

## Import note
The existing `Parsed.notes` entry ("Name: what"), reused for ignored `.conf` lines.
