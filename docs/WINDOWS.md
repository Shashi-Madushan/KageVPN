# Kage on Windows: build and test

Windows support is arriving in phases:

| Phase | What works | State |
|---|---|---|
| 1 | Proxy Only mode, the Windows system proxy, the engine manager, the Windows title bar and tray, installed apps, Ctrl shortcuts, the NSIS installer and updates | Built; compiled and unit-tested from macOS; **needs the checks below on Windows** |
| 2 | TUN mode through a helper service installed once (UAC) | Planned |
| 3 | A firewall kill switch (WFP) | Planned |

Until phase 2, TUN mode, the kill switch and the VPN service are hidden on Windows, and the app always connects in Proxy Only mode.

## 1. Set up the PC (once)

1. **Windows 10 (22H2) or 11, x64.** WebView2 is built into Windows 11; on Windows 10, install the *Evergreen Runtime* if it is missing.
2. **Visual Studio Build Tools** with the *Desktop development with C++* workload (MSVC and the Windows SDK).
3. **Rust:** <https://rustup.rs>. Keep the default `x86_64-pc-windows-msvc` toolchain.
4. **Node.js 20 or newer.**
5. **Git**, then clone the repository and check out `feature/windows`.

```powershell
npm install
```

## 2. Automated checks

Run these in the repository folder:

```powershell
cd src-tauri; cargo test --lib -- --test-threads=4; cd ..
cd src-tauri; cargo clippy --all-targets -- -D warnings; cd ..
npx svelte-check --threshold error
npm run test:scripts
```

**Expected:**
- Every test passes. A few macOS-only tests (the root helper, `networksetup`, `.app` bundles and file modes) do not compile on Windows. Windows runs its own instead: `wininet`, `winapps`, `pngenc`, `platform`, the tray placement tests and the zip engine tests.
- Send me the output of any failure.

## 3. Run the app

```powershell
npm run tauri dev
```

To build the installer, the same way a release is built but unsigned:

```powershell
npm run tauri build -- --bundles nsis --config '{\"bundle\":{\"createUpdaterArtifacts\":false}}'
```

The installer is `src-tauri\target\release\bundle\nsis\Kage_<version>_x64-setup.exe`. It installs for the current user, so it needs no administrator rights. SmartScreen will warn, because the app is not code-signed: choose *More info* and then *Run anyway*.

## 4. Manual checks (phase 1)

Tick each one, and note anything that looks or behaves wrong.

### Window and tray
- [ ] The main window has no Windows title bar. "Kage" is at the top left, and minimize, maximize/restore and close are at the top right. They are 46 px wide, and close turns red on hover.
- [ ] Dragging the top strip moves the window; double-clicking it maximizes and restores.
- [ ] The window resizes from its edges, has rounded corners and a shadow on Windows 11, and Mica shows through the sidebar.
- [ ] Close hides the window (the app keeps running in the tray).
- [ ] The tray icon is white on a dark taskbar and dark on a light one. Change *Settings > Personalization > Colors* and restart the app to check both.
- [ ] Left-clicking the tray icon opens the popover **above** the taskbar, next to the icon, fully on screen. Clicking elsewhere closes it. Right-click shows the menu.
- [ ] Ctrl+1 to Ctrl+9 switch pages. The sidebar tooltips say `Ctrl+1` and so on.
- [ ] The text says "PC", "Windows" and "system tray", never "Mac", "macOS" or "menu bar". Look at Settings, Home, the setup guide (*Settings > Run setup again*) and the subscription options.
- [ ] There is no TUN mode anywhere (Home, popover, setup guide, Diagnostics), and no kill switch, VPN service or "Speed in menu bar" setting.

### Engine
- [ ] On first run the Engine page offers to download sing-box. It downloads `sing-box-<version>-windows-amd64.zip`, checks it, and lists the version as installed.
- [ ] Switching, rolling back and removing versions work.
- [ ] `%APPDATA%\com.apzur.kage\cores\sing-box\<version>\sing-box.exe` exists.

### Connecting (Proxy Only)
- [ ] Add a server and connect. No console window appears.
- [ ] Home turns green. Diagnostics shows the tunnel IP. The Connections page lists connections.
- [ ] `curl.exe -x http://127.0.0.1:2080 https://api.ipify.org` prints the server's IP. If another program holds port 2080, use the port shown on Home.
- [ ] Disconnect: in Task Manager, `sing-box.exe` is gone within about a second.
- [ ] Kill the app while connected: end the Kage task in Task Manager. `sing-box.exe` must be gone too (the job object).

### System proxy
- [ ] Turn on *Settings > Set system proxy*, then connect.
- [ ] *Settings > Network & internet > Proxy* shows "Use a proxy server: On", with `127.0.0.1` and the port, and the bypass list includes `<local>`.
- [ ] Edge or Chrome, with no proxy setup of their own, browse through the VPN.
- [ ] Disconnect: the proxy settings are exactly as they were before. Try it twice: once with no proxy before, and once with your own proxy or a setup script set first.
- [ ] **Crash restore:** connect with the system proxy on, then end the Kage task in Task Manager. Within about 2 seconds the proxy settings are back (the guard process). You can also check `HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings` in `regedit`.
- [ ] Turn the proxy setting off yourself while connected, then disconnect: Kage leaves your change alone.

### Apps and rules
- [ ] *App controls* lists your installed programs with their icons.
- [ ] Set one app (Chrome, say) to *Direct* while everything else goes through the VPN. That app shows your real IP and the others show the server's. App rules need TUN mode to catch apps that ignore the system proxy, so test with an app that follows the proxy, or check the Connections page routing.
- [ ] Blocking Kage itself is refused.

### Other
- [ ] Open a `kagevpn://` link from a browser while the app runs: the running app handles it, and no second copy starts.
- [ ] Start the app a second time from the Start menu: the first window comes forward.
- [ ] *Settings > Launch at login*, then sign out and back in: the app starts in the tray.
- [ ] Notifications appear (connect, disconnect, and an engine update).
- [ ] *Connect by network:* on a new network the app asks once in a Windows message box.
- [ ] Feedback and *Export diagnostics* name the Windows version (for example `Windows 23H2 (build 22631)`).
- [ ] QR import: drop a QR screenshot on *Add server*. HEIC files may not open on Windows; PNG and JPEG should.

## 5. Releasing for Windows

On the Windows PC, with the updater signing key copied to `%USERPROFILE%\.tauri\apzur-vpn-updater.key` (and `.pub`):

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<password>"
npm run release -- --version <v> --github Shashi-Madushan/KageVPN --notes "..."
```

This builds the NSIS installer and its signature, and stages them as `release\<v>\kage-vpn_<v>_windows-x86_64.setup.exe`. It writes `latest.json` with a `windows-x86_64` entry and keeps the macOS entries already published for that version (it downloads the release's `latest.json` first). Add `--publish` to upload.

## Checking Windows code from a Mac

```bash
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
npm run check:windows
```

This runs clippy on the Windows code and its tests (all targets) with the GNU toolchain. It catches type and `cfg` mistakes, but runs nothing; the checks above still need Windows.
