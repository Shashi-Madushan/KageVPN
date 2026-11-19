# W4 Remaining platform areas: summary

All steps of `plans/w4-platform-areas-code-generation-plan.md` are done.

## Verified on this Mac
- `cargo test --lib`: 471 passed. New tests:
  - PNG encoder: CRC and a round trip through Tauri's image loader.
  - Windows uninstall-entry parsing and folder matching.
  - Windows app-rule patterns.
  - The self-block guard with Windows paths.
- Clippy is clean on macOS and Windows (gnu), with all targets.
- `svelte-check`, the script tests and `vite build` pass.

## Not verified (needs Windows)
- The app list and icons, including the quality of old icons that get their alpha from the mask.
- Routing an app by folder with sing-box's `process_path_regex` on Windows.
- A `kagevpn://` link from a browser reaching the running app.
- Autostart.
- The Windows version and model strings.
- Microsoft Store apps are not listed (known limit). A process-name rule covers them.
