# W4 Remaining platform areas: Code Generation Plan
Round 5, Part B, phase 1. Covers FR-B1.8 to FR-B1.11. (The network watch, gateway and process-stats parts of FR-B1.9 and FR-B1.10 were done in W1.)

## Steps
- [x] 1. **Installed apps** (`apps_windows.rs`, swapped in for `apps.rs`):
  - Apps come from the uninstall registry entries (machine, 32-bit and user). System parts, updates and child entries are skipped.
  - The program comes from `DisplayIcon`, or else from the `.exe` named like the app in `InstallLocation`. An app is its program's folder.
  - Icons: `ExtractIconExW`, then `GetIconInfo`, then `GetDIBits`. Old icons get their alpha from the mask. The result is PNG-encoded and cached.
  - The pure parts are in `winapps.rs` and `pngenc.rs`, both tested on macOS.
- [x] 2. **App rules** (`config.rs`): a Windows folder becomes the case-insensitive pattern `(?i)^C:\\...\\`. The self-block guard reads `\` paths and `.exe` names. Tests are included.
- [x] 3. **Front end:** app labels split on `\` as well as `/`, and the Safari note is macOS only.
- [x] 4. **Single instance and deep links:** `tauri-plugin-single-instance` (Windows only, deep-link feature) passes a second launch's `kagevpn://` link to the running app or shows its window. A debug build registers the scheme itself.
- [x] 5. **Device info** (`device.rs`): `OS_NAME`, plus `os_version()` and `model()` from the registry on Windows. Subscription headers report Windows. Feedback and the diagnostics bundle name the OS (the macOS output is unchanged).
- [x] 6. **Autostart:** the plugin already uses the Run registry key on Windows; the `--hidden` flag is passed the same way. No change.
- [x] 7. **Verify:** macOS tests and clippy pass, and Windows clippy (all targets) passes.
