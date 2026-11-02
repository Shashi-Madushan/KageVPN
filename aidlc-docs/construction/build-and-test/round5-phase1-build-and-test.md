# Round 5 build and test: Part A and Windows phase 1

## Run on this Mac (2 November 2026)
| Check | Result |
|---|---|
| `cargo test --lib` (macOS) | 471 passed, 1 ignored (live) |
| `cargo clippy --all-targets -D warnings` (macOS) | clean |
| `cargo clippy --target x86_64-pc-windows-gnu --all-targets -D warnings` | clean |
| `svelte-check` | 0 errors |
| `npm run test:scripts` | 50 passed |
| `vite build` | ok |
| Universal app (`lipo -archs`) | `x86_64 arm64` |
| QR import in the mock UI | link and subscription paths work |
| Windows chrome in the mock (`?os=windows`) | title strip, buttons, hidden features and wording look right; macOS view unchanged |

## On the Windows device (user)
Follow `docs/WINDOWS.md` sections 2 to 4, and record the results here before the phase 1 PR is opened.

| Check | Result |
|---|---|
| `cargo test --lib` on Windows | not run yet |
| Manual checklist, window and tray | not run yet |
| Engine | not run yet |
| Connecting (Proxy Only) | not run yet |
| System proxy, including the crash restore | not run yet |
| Apps and rules | not run yet |
| Other (deep link, single instance, autostart, notifications, networks) | not run yet |

## Not run anywhere
- A signed release on either OS (needs the signing key and password).
- The universal app on an Intel Mac.
