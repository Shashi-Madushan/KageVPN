# W1 Platform layer and engine: Code Generation Plan
Round 5, Part B, phase 1. Covers FR-B0.1, FR-B0.2, FR-B0.5, FR-B1.1, FR-B1.7. Design: `inception/application-design/windows-platform-design.md`. This plan is the single source of truth for W1.

## Goal
The crate compiles for Windows (`cargo check --target x86_64-pc-windows-gnu --all-targets`). Every Windows-only area is a file with the same API, and the engine manager downloads and unpacks the Windows engine. macOS behaviour and tests do not change.

## Steps
- [x] 1. **Engine asset and binary name** (`app_config.rs`): `asset_name` picks the OS (`darwin` with `.tar.gz`, `windows` with `.zip`), and `binary_name` is `sing-box.exe` on Windows. Unit tests cover the pure helper `asset_for(os, version, arch)`.
- [x] 2. **Zip extraction** (`core/installer.rs`):
  - The archive file name follows the asset's extension.
  - `extract_binary` reads `.zip` with the `zip` crate (deflate), with the same size caps as tar.
  - `set_executable` is a no-op on Windows.
  - Tests: a zip with `sing-box-1.0.0-windows-amd64/sing-box.exe` is extracted; a zip without the binary is refused; an oversized entry is refused. The tests run on macOS too.
- [x] 3. **Dependencies** (`Cargo.toml`):
  - Add `zip` (deflate only).
  - Add a `[target.'cfg(windows)'.dependencies]` block with `windows-sys` (Win32 features as needed) and `winreg`.
- [x] 4. **Module swaps** (`lib.rs`): `#[cfg_attr(windows, path = ...)]` for `runner`, `sysproxy`, `apps` and `appnap`. The Windows files compile with the same public API:
  - `runner_windows/mod.rs` and `runner_windows/daemon.rs`: start, stop and the signals. Proxy mode is filled in by W2; until then it returns "not available on Windows yet".
  - `sysproxy_windows.rs`: the state type and functions. It does nothing until W2.
  - `apps_windows.rs`: an empty list until W4.
  - `appnap_windows.rs`: no-op stubs.
  - The macOS files are unchanged, apart from `#[cfg]` on tests that only make sense there.
- [x] 5. **In-file splits.** These get `#[cfg(target_os = "macos")]` and `#[cfg(windows)]` pairs; the Windows bodies are filled in by W4 and return empty for now:
  - `network_watch::current_fingerprint`
  - `networks::current`
  - `sampler` internals
  - `reliability::restore_firewall` (Windows says "not available")
  - `core::commands` Homebrew upgrade (macOS only)
  - `lib.rs` autostart (`MacosLauncher` is ignored on Windows)
  - any other spot the Windows check reports
- [x] 6. **Platform description** (`platform.rs`, new): a `Platform` struct and the `platform()` command, registered in `lib.rs`. A matching `Platform` type and `api.platform()` go in `api.ts`, plus `src/lib/platform.svelte.ts` (`platform`, `isMac`, `isWindows`, `modKey`, a user-agent fallback), loaded once at start. Tests cover the macOS values. The mock config answers `platform` with macOS.
- [x] 7. **Windows check tooling.** Add a `check:windows` npm script that runs `cargo check --target x86_64-pc-windows-gnu --all-targets` and `cargo clippy` for that target, and note mingw-w64 in `docs/WINDOWS.md` (started here, completed in W5).
- [x] 8. **Verify:**
  - macOS: `cargo test`, `cargo clippy --all-targets -- -D warnings`, `svelte-check`, `vite build` and script tests all green.
  - Windows check: passes with no errors.
  - Add `aidlc-docs/construction/w1-platform-layer/code/summary.md`.
