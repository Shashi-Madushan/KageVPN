# Build and test summary
| Check | Result |
|---|---|
| `cargo test --lib -- --test-threads=4` | 300 passed |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `svelte-check` | 0 errors |
| `vite build` | succeeds |
| `npm run test:scripts` | 9 passed |
| Real `sing-box check` | passes for the IPv6-off config |

Not verified: everything under "Integration (manual)", the root-service reinstall, and the unexplained stall of `cargo test --lib` at default thread count.
