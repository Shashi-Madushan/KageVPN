# U1 Recovery tools: Code Generation Plan
Single source of truth for this unit. Covers FR2, FR3, FR4. Code lives in the workspace (src-tauri/, src/), never in aidlc-docs/.

## Deviation to confirm
FR2 said "saved where the user picks". The app has no file-dialog plugin; adding one is a new dependency and capability. The plan saves to `~/Downloads/kage-diagnostics-<timestamp>.zip`, reveals it in Finder and copies the short summary to the clipboard. Say if you want a real save dialog instead.

## Steps
- [x] 1. **Engine rollback, backend** (`core/mod.rs`, `core/commands.rs`, `core/provider.rs`): add `good_version` and `previous_version` to `CoreSettings`; one `activate(version)` method that records the old active version as `previous_version`, used by `core_use`, `apply_pending` and install-activation; `mark_good` called once a connection is verified (`session.rs`); `core_rollback` command (target = good version if different from active, else previous; must still be on disk); `core_remove_old` command (delete every installed version except the active and the rollback target).
- [x] 2. **Rollback tests** (`tests/core.rs`): activation records previous; rollback picks good over previous, refuses when the target is missing; remove_old keeps active + target; mark_good only for the active managed version.
- [x] 3. **Repair service, backend** (`runner/mod.rs`, new `repair.rs` or in `commands.rs`): `repair_service` command: refuse while connected/connecting; clear stale start/stop/down/block/quit/pid files; reset the once-per-launch flags; reinstall via `daemon::install` (one prompt) when an engine resolves; verify `daemon::installed()`; return a short report. Pure helper for "which files are stale" so it is testable with a temp dir.
- [x] 4. **Repair tests**: stale-file clearing in a temp dir; refuses while connected.
- [x] 5. **Diagnostics bundle, backend** (`diagnostics_bundle.rs`, reuses `feedback::redact`): build a zip (add `zip` crate, deflate only) with redacted app log, sing-box log tail (last 2000 lines), config with secrets/servers removed (Clash secret, passwords, UUIDs, server hostnames/IPs via redact + JSON key scrub), versions, settings summary (no server data, no subscription URLs), plus a short summary string; command `diagnostics_export` writes it to Downloads, returns path + summary; reveal + clipboard done from the UI.
- [x] 6. **Bundle tests**: a profile's host/uuid/password and a subscription URL never appear in any file of the zip; zip is readable; log tail is capped.
- [x] 7. **Register commands** (`lib.rs`, `commands` lists) and add typed wrappers in `src/lib/api.ts`.
- [x] 8. **UI**: Core page: "Roll back to last working version" (shown when a target exists, with the version) and "Remove old versions"; Diagnostics page: "Export diagnostics" and "Repair VPN service" with confirmation dialog explaining the one password prompt.
- [ ] 9. **Docs**: docs-site guide + changelog entry; brief `aidlc-docs/construction/u1-recovery-tools/code/summary.md`.
- [ ] 10. **Verify**: `cargo test`, `cargo clippy --all-targets -- -D warnings`, `svelte-check`. Root-service reinstall cannot be exercised here; listed for manual check.

## Dependencies / contracts
No other unit depends on U1. New Tauri commands: `core_rollback`, `core_remove_old`, `repair_service`, `diagnostics_export`.
