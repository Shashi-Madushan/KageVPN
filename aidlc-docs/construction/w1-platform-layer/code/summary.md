# W1 Platform layer and engine (with W2's runner and system proxy): summary

## Changed
- **Engine:**
  - `app_config::sing_box_asset`: `windows` uses `.zip`, other platforms `.tar.gz`.
  - `binary_name` is `sing-box.exe` on Windows, and there are no Homebrew system paths there.
  - `core/installer`: the archive name follows the URL; zip extraction uses the same size caps; there is no executable bit on Windows.
  - Tests: zip extract, missing binary, zip bomb, archive name, asset names.
- **Module swaps** in `lib.rs`:
  - `runner` maps to `runner_windows/` (job-object runner and service stub).
  - `sysproxy` maps to `sysproxy_windows.rs` (WinINet).
  - The shared system-proxy code moved to `sysproxy_common.rs` and is included by both.
- **New modules:**
  - `wininet.rs`: pure WinINet rules, built on Windows and in tests everywhere, with 5 tests.
  - `winnet.rs` (Windows): adapters, primary adapter, gateway MAC.
  - `platform.rs`: the `platform()` command, with tests.
- **In-file splits:**
  - `network_watch::current_fingerprint` (Windows uses `winnet`).
  - `networks::current` (gateway MAC from `ResolveIpNetEntry2`, adapter name) and `question` (a Windows message box).
  - `sampler` (`GetProcessMemoryInfo`, `GetProcessTimes`).
  - `error::SingBoxMissing` text on Windows.
  - Several unused items are allowed on Windows only.
- **Guard process:** `lib.rs::run` returns early when started as the system-proxy guard.
- **Front end:**
  - `api.platform()` and `src/lib/platform.svelte.ts` (`isMac`, `isWindows`, `modKey`, and `data-os` on `<html>`), loaded in the root layout.
  - The mock config and screenshot demo data answer `platform`.
- **Tests made portable:**
  - The diagnostics bundle is read with the `zip` crate instead of `/usr/bin/unzip`.
  - The owner-only test is Unix-only, the AppleScript test is macOS-only, and the engine layout test uses `binary_name`.
- **Tooling:**
  - Installed `brew install mingw-w64` and the Rust targets `x86_64-pc-windows-gnu` and `x86_64-pc-windows-msvc` on this Mac.
  - Added `npm run check:windows`.
- **Dependencies:**
  - `zip` 4 (deflate).
  - Windows only: `winreg` 0.55 and `windows-sys` 0.61.

## Verified on this Mac
- `cargo test --lib`: 462 passed (1 ignored, live).
- `cargo clippy --all-targets -D warnings`: clean on macOS and for `x86_64-pc-windows-gnu`.
- `svelte-check`: 0 errors.

## Not verified (needs Windows)
- Every Windows code path at run time: the job object, the stop file, the registry writes and refresh, the guard, the adapters and gateway MAC, and the message box.
- `cargo test` on Windows.
- These go in `docs/WINDOWS.md` (W5).
