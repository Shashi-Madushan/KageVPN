# Build and test summary

## Round 4, pull request 1: TUIC, Hysteria v1, SSH, WireGuard (branch feature/protocols-and-system-proxy, units U1 to U4, at 080c53a)
| Check | Result |
|---|---|
| `cargo test --lib -- --test-threads=4` | 409 passed |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `npm run check` (svelte-check) | 0 errors, 0 warnings |
| `npx vite build` | succeeds |
| `npm run test:scripts` | 39 passed |
| `npm run build:app` | builds `Kage.app` (11 MB) |
| Real `sing-box check` (1.14.2) | passes every protocol's sample in TUN and Proxy configs, an SSH server with a pasted key, and a WARP-style WireGuard server |
| Real sing-box run on spare ports | all 10 nodes listed by the Clash API; TUIC, Hysteria, SSH and WireGuard selectable; delay tests answer (integration A and B) |
| Editor and Add server dialog | checked in the screenshot demo (headless Chrome): protocol dropdown, SSH key box and host key warning, port defaults, `.conf` detection |

Not verified:
- No connection to a real TUIC, Hysteria, SSH or WireGuard server (none available); integration steps 1 to 6 are for you.
- The live app was not run, so your real VPN on 2080/19090 was not touched.
- The engine-feature stop was tested as functions, not with an engine built without WireGuard.

## Round 2: per-app controls (branch feature/app-picker-and-usage-limits, units U1 to U4)
| Check | Result |
|---|---|
| `cargo test --lib -- --test-threads=4` | 344 passed |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `npm run check` (svelte-check) | 0 errors, 0 warnings |
| `npm run build` (vite) | succeeds |
| `npm run test:scripts` | 26 passed |
| `npm run build:app` | builds `Kage VPN.app` (10.4 MiB) |
| Real `sing-box check` | the config tests that call it pass (sing-box is installed at /opt/homebrew/bin) |

Not verified:
- Everything under "Round 2 integration (manual)": nothing was run in the live app, so the look of the Apps tab and the cap editor, the real block and reconnect, rollover at midnight, and notifications are unchecked.
- `cargo test --lib` at the default thread count: it again gave no result within 240 seconds. It also stalled before this work; the cause is still unknown, and `--test-threads=4` is the supported way to run it.
- Performance: not applicable. The cap check adds work to the existing 3-second poll only for capped apps, and one database read and write per capped app with traffic.

## Round 1 (kept for history)
| Check | Result |
|---|---|
| `cargo test --lib -- --test-threads=4` | 300 passed |
| `svelte-check`, `vite build`, `npm run test:scripts` | clean, succeeds, 9 passed |
