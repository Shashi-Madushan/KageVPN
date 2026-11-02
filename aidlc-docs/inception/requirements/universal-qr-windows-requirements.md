# Requirements: universal macOS build, QR import from images, Windows support

## Intent analysis
- **Request:** build the macOS release as one universal app, import servers from a QR code in an image, and add a Windows build whose window chrome, tray and engine management fit Windows.
- **Type:** release tooling (universal build), new feature (QR), new platform (Windows). Brownfield.
- **Clarity:** clear after the questions. **Scope:** release script; import dialog; for Windows almost every platform module, the window config, the installer and CI. **Complexity:** simple (universal), moderate (QR), complex (Windows). **Depth:** standard for the first two, comprehensive for Windows.
- Answers: `universal-qr-windows-questions.md`. The user accepted the recommended answer on Questions 1 to 9. The extension questions follow earlier rounds: Security No, Property-Based Testing No, Resiliency No. Checked for contradictions: none. Q9 changed after review: no GitHub Actions for now; the user builds and tests on their own Windows device.

## Delivery shape (Q1 A)
- **Part A** (one branch, one PR into `dev`): universal macOS build and QR import.
- **Part B** (branch `feature/windows`, PRs per phase into `dev`): Windows phase 1, phase 2, phase 3. macOS behaviour must not change in any phase.

---

## Part A

### FR-A1. Universal macOS release (Q3 A)
- **FR-A1.1** `scripts/release.mjs` builds with `--target universal-apple-darwin` and checks both Rust targets are installed (`aarch64-apple-darwin`, `x86_64-apple-darwin`), saying which `rustup target add` to run if not.
- **FR-A1.2** The `.app`, the updater archive and the optional `.dmg` come from `target/universal-apple-darwin/release/bundle/`. The `.dmg` is named for `universal`, not `process.arch`.
- **FR-A1.3** The update feed lists the universal archive for both `darwin-aarch64` and `darwin-x86_64`.
- **FR-A1.4** `npm run build:app` (local development build) stays host-only so day-to-day builds stay fast.
- **FR-A1.5** Script tests in `scripts/tests` cover the new naming and feed entries.

### FR-A2. QR import from images (Q2 A)
- **FR-A2.1** The import dialog accepts a QR code from: a chosen image file (PNG, JPEG, GIF, WebP, HEIC where the webview can decode it), an image dropped on the dialog, and an image on the clipboard (paste, plus a "From clipboard" button).
- **FR-A2.2** Decoding happens in the front end (canvas plus a small QR decoder library); no image crates are added to Rust.
- **FR-A2.3** The decoded text goes through the existing import path, so any supported link or subscription URL works and the usual per-server results and rejections are shown.
- **FR-A2.4** An image with no readable QR says so plainly. An image with several QR codes imports every one that decodes, if the decoder supports it; otherwise the first.
- **FR-A2.5** Large images are scaled down before decoding so a big screenshot does not stall the UI.
- **FR-A2.6** CSP stays strict: no new network origins; `blob:` is added to `img-src` only if needed.

---

## Part B: Windows

### Common rules
- **FR-B0.1** Platform code sits behind `cfg(target_os = ...)` modules with one shared interface per area (runner, system proxy, network watch, apps, sampler). macOS code moves, it does not change.
- **FR-B0.2** The front end asks the backend for the platform once and shows or hides features from that, instead of checking the user agent everywhere.
- **FR-B0.3** Features with no Windows version yet are hidden on Windows, not shown broken.
- **FR-B0.4** No CI in this round. The user builds and tests on their own Windows device. Each phase ships a Windows build-and-test guide (prerequisites: Rust MSVC toolchain, Node, WebView2; commands for `cargo test`, `svelte-check`, the installer build; and the manual checks for that phase).
- **FR-B0.5** On this Mac, Windows code is checked as far as possible without a Windows machine: `cargo check --target x86_64-pc-windows-msvc` (types and `cfg` wiring; linking is not attempted) and the platform-neutral unit tests. Anything that needs Windows to run is listed as unverified until the user reports results.

### Phase 1: everything that needs no admin rights (Q4 A)
- **FR-B1.1 Engine manager.** Picks `sing-box-<v>-windows-amd64.zip` (or `-arm64`), verifies SHA-256 as today, extracts `sing-box.exe` from the zip, and runs it. Install, switch, roll back and remove work as on macOS.
- **FR-B1.2 Proxy Only mode.** sing-box runs as a child of the app with no console window; it stops when the app exits or dies (a Windows job object replaces `supervisor.sh`). Free-port selection, health checks, traffic, connections and diagnostics work as on macOS.
- **FR-B1.3 System proxy.** The "Set system proxy" toggle sets the per-user WinINet proxy (`HKCU\...\Internet Settings`: `ProxyEnable`, `ProxyServer`, `ProxyOverride`, and any `AutoConfigURL`) and tells running apps through `InternetSetOption`. The FR2 rules from Round 4 apply: save before changing, restore exactly on disconnect, quit and next launch, and restore if the app dies (the job-object watcher or the next launch).
- **FR-B1.4 Main window (Q7 A).** No native title bar. The app draws a title bar area with Windows 11-style minimize, maximize/restore and close buttons on the right (correct hover colours, red close, snap layouts menu on hover of maximize where Windows provides it), a drag region, double-click to maximize, and Mica as the background effect. The macOS traffic-light inset is not reserved on Windows.
- **FR-B1.5 Tray.** Left click opens the popover above the taskbar next to the icon, whichever edge the taskbar is on; right click shows the menu. The popover uses Windows effects (Mica or Acrylic) with rounded corners, and closes when it loses focus.
- **FR-B1.6 Text and shortcuts.** `⌘` labels become `Ctrl` and shortcuts register with `Ctrl`. macOS-only wording (menu bar, Finder, "Safari...", System Settings paths) has a Windows version or is hidden.
- **FR-B1.7 Paths.** Data in `%APPDATA%\com.apzur.kage` (what Tauri gives), logs and caches in the matching local folders. File permission code that assumes Unix modes is skipped on Windows; files sit in the per-user profile folder.
- **FR-B1.8 Installed-apps picker.** Lists apps from the Start Menu shortcuts and the per-user and machine uninstall registry keys, with `.exe` icons. Per-app rules use the `.exe` name, which sing-box's `process_name` matches on Windows.
- **FR-B1.9 Network changes and "connect by network".** Network change and gateway detection use Windows APIs (`NotifyIpInterfaceChange` / `GetAdaptersAddresses`, gateway MAC from the neighbour table), so auto-reconnect and network rules work.
- **FR-B1.10 Process stats.** The sampler reads memory and CPU of sing-box through Windows APIs; the open-file check is skipped.
- **FR-B1.11 Startup, deep links, notifications, autostart.** `kagevpn://` registered by the installer and the deep-link plugin; launch at login through the autostart plugin; notifications through the notification plugin. Single instance: a second launch focuses the running app and passes it the deep link.
- **FR-B1.12 Installer and updates (Q8 A).** NSIS per-user `.exe`, x64 only. The release script gains a Windows path, run on the Windows device, producing the installer and the updater artifact; the update feed gets a `windows-x86_64` entry. Code signing for Windows is out of scope (SmartScreen will warn), as on macOS.
- **FR-B1.13 Hidden in phase 1:** TUN mode, kill switch, root-service repair, App Nap. TUN shows "coming in a later version" text instead of the mode choice, or is removed from the choice.

### Phase 2: TUN mode (Q5 A)
- **FR-B2.1** A small Windows service ("Kage Helper") installed once through a UAC prompt from Settings or on the first TUN connect. It runs as LocalSystem and starts and stops `sing-box.exe` for the app.
- **FR-B2.2** The app talks to the service over a named pipe whose ACL lets only the installing user's account connect. Requests are start, stop and status; the service does not read requests from user-writable files (avoids the macOS O1 design weakness).
- **FR-B2.3** The service runs the engine from a copy it makes in a folder only administrators can write (`%ProgramData%\Kage\engine`), checked against the SHA-256 the app recorded at install, as S1 does on macOS. The config is passed through the pipe and validated (no `log.output`, no arbitrary file paths) before sing-box runs.
- **FR-B2.4** sing-box TUN on Windows uses wintun, which sing-box carries built in; `auto_route` on, `strict_route` on (see FR-B3.1).
- **FR-B2.5** The app shows the service state in Diagnostics, can repair or remove it (UAC), and offers to update it when its version is older than the app's.
- **FR-B2.6** Disconnect, quit and app crash stop sing-box (the service notices the pipe closing).

### Phase 3: kill switch (Q6 A)
- **FR-B3.1** Until phase 3, TUN mode uses sing-box's `strict_route`, which blocks traffic outside the tunnel while sing-box runs. The kill-switch setting is hidden on Windows.
- **FR-B3.2** Phase 3 adds a persistent block through WFP filters owned by the service, with the same behaviour as the macOS design (`KILL-SWITCH-PLAN.md`): kept during reconnects and restarts, allows the server addresses and the local network, released only on an explicit disconnect or "Unblock internet". A separate design step is required first.

## Non-functional requirements
- **NFR1** macOS behaviour, tests and screenshots stay the same. All existing tests pass on macOS; the platform-neutral ones also pass when the user runs them on Windows.
- **NFR2** Windows-only code has unit tests for its pure parts (asset names, zip extract, WinINet value building, registry restore, tray position maths, pipe message parsing) that run on any OS where possible.
- **NFR3** No new console windows: every process the app starts on Windows uses `CREATE_NO_WINDOW`.
- **NFR4** Release size: the Windows installer stays under 15 MB; the universal macOS app is about twice today's 4.6 MB.
- **NFR5** The live VPN on the development Mac is not touched (see memory: spare ports only).

## Out of scope
- Windows ARM64 builds, MSI, Microsoft Store, Windows code signing.
- GitHub Actions or any other CI (may be added later).
- Linux.
- Screen scanning for QR codes.
- Network Extension mode (Round 3, parked).

## To verify in design
- QR decoder library choice (size, licence, multi-code support) and HEIC support in WebView2 and WKWebView.
- Whether Tauri's Mica on the main window works together with a transparent webview background, and the fallback on Windows 10 (Acrylic or solid).
- Snap layouts on a custom maximize button (needs `WM_NCHITTEST` returning `HTMAXBUTTON`; may need a small Win32 hook or a Tauri plugin).
- Whether `windows-sys` or the `windows` crate is used, and whether `sysinfo` is worth its size for FR-B1.10.
- How the service is packaged: a second binary in the NSIS installer vs the main app binary started with a `--service` argument.
