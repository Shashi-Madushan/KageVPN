# U5 System proxy: domain entities

Answers used: `construction/plans/u5-system-proxy-functional-design-plan.md` (all A, recommended).

## SystemProxySettings (saved, part of `ReliabilitySettings`)
| Field | Type | Default | Meaning |
|---|---|---|---|
| `systemProxy` | bool | false | Set the macOS system proxy while connected in Proxy Only mode. |
| `bypass` | list of text | empty | Extra hosts that skip the proxy (domains such as `*.corp.example`, addresses, ranges such as `10.20.0.0/16`), added to the built-in list. |

Kept next to `proxyAuth` / `proxyPassword` because they describe the same local proxy.

## Built-in bypass list
`localhost`, `127.0.0.1`, `::1`, `*.local`, `169.254.0.0/16`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`. The full list Kage writes is the built-in list followed by the user's entries, without repeats, in that order.

## ServiceProxy (one network service, as macOS stores it)
| Field | Meaning |
|---|---|
| `service` | The network service name (`Wi-Fi`, `USB 10/100 LAN`, ...). |
| `web`, `secure_web`, `socks` | Each: `enabled`, `host`, `port`, `authenticated`. |
| `auto_url` | The PAC URL, and whether it is on. |
| `auto_discovery` | WPAD on or off. |
| `bypass` | The service's bypass list (shared by all its proxies). |

Read from `networksetup -getwebproxy`, `-getsecurewebproxy`, `-getsocksfirewallproxy`, `-getautoproxyurl`, `-getproxyautodiscovery`, `-getproxybypassdomains`.

## Snapshot (saved file, `system-proxy-saved.json`, in the app's data folder, private)
- `port`: the local port Kage set (to recognise its own settings, R4).
- `services`: the `ServiceProxy` of every service Kage changed, **as it was before** Kage changed it.
- Written atomically **before** the first change; extended (not replaced) when a new service is added (R6); deleted only after a successful restore.

## Restore script (saved file, `system-proxy-restore.sh`, private)
The `networksetup` commands that put each saved service back, in a form the supervisor can run without any logic of its own. Rewritten whenever the snapshot or the port changes. Each command is guarded so it only acts while the service still points at Kage (R4), using a tiny shell check per service.

## SystemProxyState (in memory, shown to the interface)
| State | Meaning |
|---|---|
| `Off` | Kage has not changed anything (no snapshot). |
| `Set { port, services }` | Kage's proxy is set on these services. |
| `Failed { action, reason }` | A set or restore did not complete; Repair is offered. `action` is `set` or `restore`. |

Exposed in `Status` as `systemProxy` so Home can show it.
