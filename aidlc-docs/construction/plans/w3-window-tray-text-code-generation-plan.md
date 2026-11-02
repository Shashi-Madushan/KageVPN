# W3 Window, tray and text: Code Generation Plan
Round 5, Part B, phase 1. Covers FR-B1.4 to FR-B1.6 and FR-B1.13. Design: `inception/application-design/windows-platform-design.md` (Window chrome).

## Steps
- [x] 1. **Window config.** Add `src-tauri/tauri.windows.conf.json` (merged on Windows only):
  - Main window: no decorations, transparent, shadow, Mica.
  - Tray window: Acrylic.
  - Bundle target: NSIS, per-user.
- [x] 2. **Window permissions** (`capabilities/default.json`): minimize, toggle-maximize, is-maximized and close.
- [x] 3. **`WindowControls.svelte`:** Windows 11-style minimize, maximize/restore and close buttons (46 x 32, Segoe Fluent Icons, red close on hover). The maximized state follows `onResized`.
- [x] 4. **Main window** (`+page.svelte`), on Windows only:
  - The app name at the left of a 32 px drag strip, and the controls on the right.
  - The sidebar and header move down clear of the strip.
  - Page shortcuts use Ctrl (Cmd or Ctrl on macOS, as before).
- [x] 5. **Tray** (`tray.rs`):
  - On Windows the template icons are tinted to the taskbar's text colour (`SystemUsesLightTheme`).
  - The popover is placed above an icon in the lower half of the screen; `popover_origin` is pure and tested.
  - The speed readout is macOS only (`Platform.speed_in_tray`).
- [x] 6. **Wording** (platform store: `device`, `osName`, `trayName`, `shortcut()`): about 20 strings in Settings, Home, Onboarding, Usage settings, Subscription form, Feedback and Sidebar now follow the OS. macOS text is unchanged.
- [x] 7. **Features not available on Windows are hidden:**
  - TUN in every mode picker; a picker with one option is hidden.
  - The kill switch toggle and its Home chip.
  - The root service and Repair VPN service.
  - Speed in the menu bar.
  - The Diagnostics "switch to TUN" suggestion.
  - Backend: `Store::mode()` returns Proxy Only where there is no TUN, and `set_mode(Tun)` is refused.
- [x] 8. **Mock:** `?os=windows` makes the browser preview act as Windows.
- [x] 9. **Verify** on macOS and with the Windows check, and look at the Windows chrome in the mock.
