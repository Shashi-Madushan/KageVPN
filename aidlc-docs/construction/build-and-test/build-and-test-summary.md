# Build and test summary

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
