# Build and test summary

## Round 4, pull request 2: system proxy (branch feature/system-proxy, unit U5, at db08ecc)
| Check | Result |
|---|---|
| `cargo test --lib -- --test-threads=4` | 424 passed, 1 ignored (the live test) |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `npm run check` (svelte-check) | 0 errors, 0 warnings |
| `npx vite build` | succeeds |
| `npm run test:scripts` | 40 passed |
| `npm run build:app` | builds `Kage.app` (11 MB) |
| Live set and restore on "iPhone USB" (with permission) | no password prompt; every field back as before; Wi-Fi untouched (integration C) |
| Crash path with a real sing-box | supervisor stops sing-box, runs the restore script, removes the files (integration D) |
| Settings section and Home chip | checked in the screenshot demo |

Not verified:
- Integration steps 7 to 14 (a real Proxy Only connection in the running app, force-quit, power-cut case, PAC, Ethernet): the live app was not run, so your real VPN was not touched.
- The published screenshots were not regenerated for the new Settings section.

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
