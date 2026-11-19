# Application Design: Windows platform layer (Round 5, Part B)

Requirements: `inception/requirements/universal-qr-windows-requirements.md` (FR-B0 to FR-B3). Execution plan: `inception/plans/universal-qr-windows-execution-plan.md`.

## Goals
1. macOS code is not rewritten. Files with macOS-only behaviour keep their content, and a Windows file with the **same public API** is chosen at compile time.
2. Callers do not change. `session.rs`, `commands.rs`, `reliability.rs` and the rest keep calling `crate::runner::start`, `crate::sysproxy::restore_now` and so on.
3. Features with no Windows version yet still compile and say "not available on Windows" at the edge, and the front end hides them using one platform description.

## Pattern: one module name, one file per OS
Most modules are swapped whole in `lib.rs`:

```rust
#[cfg_attr(windows, path = "runner_windows/mod.rs")]
mod runner;
#[cfg_attr(windows, path = "sysproxy_windows.rs")]
mod sysproxy;
#[cfg_attr(windows, path = "apps_windows.rs")]
mod apps;
#[cfg_attr(windows, path = "appnap_windows.rs")]
mod appnap;
```

Modules where only a function or two differ keep one file, with `#[cfg(target_os = "macos")]` and `#[cfg(windows)]` versions of those functions: `network_watch::current_fingerprint`, `networks::current`, `sampler` internals, `core` archive extraction, `lib.rs` setup. Pure helpers stay shared and keep their tests.

Tests for macOS-only modules (`tests/runner.rs`, `tests/sysproxy.rs`, `tests/apps.rs`) are compiled with their module, so on Windows the Windows file's own tests run instead.

## Areas and their Windows versions

| Area (module) | Public API kept | Windows version | Phase |
|---|---|---|---|
| Running sing-box (`runner`) | `start`, `request_stop`, `request_down`, `signal_ready`, `signal_block`, `is_alive`, `forget_repair_attempts`, `run_admin_script`, `daemon::{installed, binary_current, script_current, install, uninstall, install_commands}` | `runner_windows/mod.rs`. **Proxy mode:** spawns `sing-box.exe run` with `CREATE_NO_WINDOW` inside a Job Object with `KILL_ON_JOB_CLOSE`, so it dies with the app, and a watcher thread stops it when the stop file appears. **TUN mode:** `AppError::Launch("TUN mode is not available on Windows yet")` in phase 1; the service client in phase 2. `run_admin_script` returns an error. `daemon` holds the phase 2 service installer; in phase 1 `installed()` is `false` and `install` returns the same error. `is_alive` uses `OpenProcess` + `GetExitCodeProcess` | 1, 2 |
| System proxy (`sysproxy`) | `SystemProxyState`, `applies`, `bypass_list`, `has_snapshot`, `sync`, `spawn_sync`, `restore_now`, `restore_at_launch`, `current` | `sysproxy_windows.rs`: reads and writes `HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings` (`ProxyEnable`, `ProxyServer`, `ProxyOverride`, `AutoConfigURL`, `AutoDetect` is left alone), then calls `InternetSetOptionW(INTERNET_OPTION_SETTINGS_CHANGED / REFRESH)`. The snapshot JSON is in the same file as on macOS. The crash restore runs from the runner's watcher thread (see W2) and at launch. `bypass_list` and `applies` are shared, moved to `sysproxy_common.rs` | 1 |
| Network change (`network_watch`) | `spawn`, `is_virtual`, `fingerprint`, `detect` | `current_fingerprint` on Windows: `GetAdaptersAddresses` (up adapters, unicast addresses, link-local skipped). The same 2 s polling loop; no notification API is needed | 1 |
| Network identity (`networks::current`) | `CurrentNetwork`, `current` | `GetBestRoute2` toward 0.0.0.0 for the gateway, then `GetIpNetEntry2` for its MAC. The name is the adapter's friendly name, or `default_name(id)` | 1 |
| Installed apps (`apps`) | `InstalledApp`, `list`, `icon_data_url`, `bundle_of` | `apps_windows.rs`: Start Menu `.lnk` files (user and all-users) resolved to `.exe` through `IShellLinkW`, plus `App Paths` registry keys. `path` is the `.exe` path, `id` is the lower-case exe name. The icon comes from `SHGetFileInfoW` / `ExtractIconExW`, drawn to a 32 px PNG. `bundle_of` returns the `.exe` path when the process path ends in `.exe` | 1 |
| Process stats (`sampler`) | `spawn`, `raise_fd_limit` | `GetProcessMemoryInfo` and `GetProcessTimes`; the fd limit is a no-op | 1 |
| App Nap (`appnap`) | `hold`, `release`, `is_held` | no-op stubs that remember the flag | 1 |
| Kill switch restore (`reliability::restore_firewall`) | the command stays | Windows: returns "not available on Windows yet" (phase 3 makes it ask the service) | 1, 3 |
| Engine archive (`core::installer`, `core::releases`) | unchanged | asset `sing-box-<v>-windows-amd64.zip` / `-arm64.zip`; the zip is extracted with the `zip` crate (already a dependency); the binary name is `sing-box.exe` (`EXE_SUFFIX`) | 1 |
| Paths (`paths`) | unchanged | Tauri's data dir (`%APPDATA%\com.apzur.kage`). `ROOT_RUN_DIR` stays macOS-only; `SERVICE_DIR = %ProgramData%\Kage` is added in phase 2 | 1, 2 |
| App setup (`lib.rs`, `tray.rs`) | unchanged | the activation policy is already macOS-only. Single instance through `tauri-plugin-single-instance`, which focuses the window and forwards `kagevpn://` arguments to `deeplink::handle`. On Windows, a deep link arrives as a command-line argument | 1 |

## Platform description for the front end
A new `platform.rs` with a `platform()` Tauri command:

```rust
pub struct Platform {
    pub os: &'static str,            // "macos" | "windows"
    pub tun: bool,                    // phase 1 Windows: false
    pub kill_switch: bool,            // Windows before phase 3: false
    pub root_service: bool,           // macOS: launchd service; Windows phase 2: the helper service
    pub system_proxy: bool,           // both: true
    pub app_nap: bool,                // macOS only
}
```

The front end reads it once into `src/lib/platform.svelte.ts`, which exports `platform` (the description), `isMac`, `isWindows`, `modKey` (`⌘` or `Ctrl`) and `shortcutLabel(n)`. Components use these instead of user-agent checks. Where a description is needed before the backend answers (first paint), `navigator.userAgent` containing `Windows` is the fallback. The mock config and the screenshot scripts set `os: "macos"`.

## Window chrome (W3)
- `tauri.windows.conf.json`, which Tauri merges on Windows only:
  - Main window: `decorations: false`, `transparent: false`, `titleBarStyle` removed, `windowEffects: { effects: ["mica"] }`.
  - Tray window: `windowEffects: { effects: ["acrylic"] }`, `radius` dropped (Windows 11 rounds the corners itself).
- A new `TitleBar.svelte`, rendered only on Windows. It is a 32 px strip with `data-tauri-drag-region` (double-click maximizes; Tauri handles it), and three 46 x 32 buttons on the right (minimize, maximize/restore, close) with Segoe Fluent Icons glyphs and the Windows hover colours (close turns red `#c42b1c`). It calls `getCurrentWindow().minimize() / toggleMaximize() / close()`. The macOS layout keeps its traffic-light inset; on Windows that padding is 0 and the sidebar starts under the title bar.
- Snap layouts on hover of maximize are not available through Tauri. It is noted as a known limit; no Win32 subclassing in this round.
- Tray popover: `tray.rs` already clamps to the monitor. On Windows the icon rectangle is at the taskbar, so the popover is placed above it (or below, for a top taskbar) using the tray `Rect` and the monitor work area.

## Service boundary (phase 2, detailed in W6 functional design)
- **Binary:** the same `kage-vpn.exe`, started with `--service` by the Service Control Manager (`windows-service` crate). There is no second binary, so the engine and app versions cannot drift.
- **Install:** `daemon::install` runs `sc create KageHelper binPath= "<ProgramData>\Kage\kage-vpn.exe --service" start= demand obj= LocalSystem` through one elevated PowerShell (`Start-Process -Verb RunAs`). The service executable and the engine are copied to `%ProgramData%\Kage`, which only Administrators and SYSTEM can write.
- **Channel:** a named pipe `\\.\pipe\com.apzur.kage.helper`. Its DACL allows SYSTEM, Administrators and the installing user's SID only. Messages are JSON lines: `Start { config, kill_switch }`, `Stop`, `Down`, `Ready`, `Block`, `Status`. The config is sent in the message, validated by the service (no `log.output` or other file paths, no `experimental.cache_file.path` outside its folder), and written to its own folder.
- **Lifetime:** the service stops sing-box when the pipe client disconnects (app quit or crash).

## Dependencies added
- `[target.'cfg(windows)'.dependencies]`:
  - `windows-sys` with the features each unit lists.
  - `windows-service` (phase 2).
  - `winreg` for the registry.
- `tauri-plugin-single-instance` (all platforms; it also helps on macOS when the app is opened twice from Finder).
- `zip`: already present (diagnostics bundle). Extraction is added.

## Verification plan
- Every unit:
  - macOS: `cargo test`, `cargo clippy --all-targets -- -D warnings`, `svelte-check`, `vite build` and script tests.
  - Windows: `cargo check --target x86_64-pc-windows-gnu --all-targets` (mingw-w64, C deps compile) and `cargo clippy` on the same target.
- **On your Windows device:** `docs/WINDOWS.md` lists the build commands, `cargo test`, and the manual checks per phase.
