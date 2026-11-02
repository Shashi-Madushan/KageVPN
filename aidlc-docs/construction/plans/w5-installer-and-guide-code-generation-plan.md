# W5 Installer and guide: Code Generation Plan
Round 5, Part B, phase 1. Covers FR-B1.12, FR-B0.4, FR-B0.5.

## Steps
- [x] 1. **NSIS bundle:** `tauri.windows.conf.json` sets `targets: ["nsis"]` and `installMode: currentUser` (done in W3).
- [x] 2. **Release script, Windows path** (`scripts/release.mjs`, `release-lib.mjs`):
  - On `win32` it builds `--bundles nsis` and signs the installer (Tauri v2 updater artifacts), staging `kage-vpn_<v>_windows-x86_64.setup.exe` and its `.sig`.
  - The feed entry is `windows-x86_64`.
  - There is no Keychain on Windows, so the password must come from the environment.
  - npm is run through a shell on Windows, and the key override is passed as a file.
  - On both OSes, the published `latest.json` for the version is downloaded first, so the other OS's entries are kept.
  - Tests cover the installer name and a manifest that merges the macOS and Windows entries.
- [x] 3. **`docs/WINDOWS.md`:** setting up the PC, the automated checks, running and building, the phase 1 manual checklist, releasing, and checking Windows code from a Mac.
- [x] 4. **Docs:** a Windows section in `RELEASING.md`, the README badge and requirements line, and a changelog entry.
