# A1 Universal build: Code Generation Plan
Round 5, Part A. Covers FR-A1.1 to FR-A1.5 of `universal-qr-windows-requirements.md`. No functional design stage. This plan is the single source of truth for A1.

## Context
- `scripts/release.mjs` builds with `tauri build --bundles app` for the host CPU and reads the bundle from `src-tauri/target/release/bundle/`.
- With `--target universal-apple-darwin`, Tauri builds both architectures, merges them with `lipo`, and writes the bundle to `src-tauri/target/universal-apple-darwin/release/bundle/`. The `.dmg` is named `Kage_<version>_universal.dmg`.
- `release-lib.mjs`:
  - `platformKey()` returns one `darwin-*` key for the host.
  - `artifactName()` strips `darwin-` for the file name.
  - `buildManifest()` writes one platform entry and keeps the others from the same version.
- The updater on an Intel Mac asks for `darwin-x86_64`, and on Apple Silicon for `darwin-aarch64`. Both must point at the same universal archive and signature.
- Only `aarch64-apple-darwin` is installed here; `x86_64-apple-darwin` is needed.
- `build:app` in `package.json` stays host-only (FR-A1.4).
- **Branch:** `feature/universal-and-qr`, created from `feature/home-tray-redesign`.

## Steps
- [x] 1. **Branch.** Create `feature/universal-and-qr` from the current branch. The Round 5 planning docs (uncommitted) come along.
- [x] 2. **Library** (`scripts/release-lib.mjs`):
  - Add `UNIVERSAL_TARGET = 'universal-apple-darwin'` and `UNIVERSAL_PLATFORMS = ['darwin-aarch64', 'darwin-x86_64']`.
  - Add `missingTargets(installedList)`. It returns which of `aarch64-apple-darwin` and `x86_64-apple-darwin` are missing from `rustup target list --installed` output.
  - `artifactName(slug, version, 'universal', ext)` gives `kage-vpn_<v>_universal.<ext>`. The current function already does this when it is passed `'universal'`, so the code stays the same and only a test is added.
  - `buildManifest` accepts `platforms: string[]` as well as `platform`. Every listed key gets the same `{ signature, url }`; the existing single-key call keeps working.
- [x] 3. **Script** (`scripts/release.mjs`):
  - Check targets with `rustup target list --installed` before building. If any are missing, stop and print `rustup target add <missing>`.
  - Build with `--target universal-apple-darwin` for both the app and the `.dmg`.
  - Read the bundle from `target/universal-apple-darwin/release/bundle/`, with the `.dmg` at `dmg/Kage_<v>_universal.dmg`.
  - Stage `kage-vpn_<v>_universal.app.tar.gz` (and `.dmg`).
  - Write `latest.json` with both `darwin-aarch64` and `darwin-x86_64` pointing at it.
  - Print `Release Kage <v> for macOS (universal: Apple Silicon and Intel)`.
  - Remove the now-unused host `platformKey` call. The function itself stays exported, because Windows (W5) will reuse the pattern.
  - `--dry-run` also reports missing targets without failing, so the check can be tried safely.
- [x] 4. **Tests** (`scripts/tests/release-lib.test.mjs`):
  - `missingTargets`: both present, one missing, none present, and extra unrelated targets are ignored.
  - Universal artifact names.
  - A manifest with `platforms` has both keys with the same signature and URL.
  - Older entries for the same version are kept, and the single `platform` form is unchanged.
- [x] 5. **Install the target here**: `rustup target add x86_64-apple-darwin`. This is a toolchain change on this Mac only; nothing in the repo changes.
- [x] 6. **Docs**:
  - `docs/RELEASING.md`: what the release builds now, the one-time `rustup target add`, the new file names, and both feed keys.
  - README: the build requirements note mentions the extra target for releases only.
  - A changelog entry.
  - `aidlc-docs/construction/a1-universal-build/code/summary.md`.
  - Closes AUDIT O6's second bullet: mark it done in `docs/AUDIT-2026-10.md`.
- [x] 7. **Verify**:
  - `npm run test:scripts`.
  - `node scripts/release.mjs --dry-run`.
  - A real universal build without signing or publishing: `npm run tauri -- build --target universal-apple-darwin --bundles app --config '{"bundle":{"createUpdaterArtifacts":false}}'`, then check that `lipo -archs` on `Kage.app/Contents/MacOS/*` prints `x86_64 arm64`.
  - The built app is not launched, because the live Kage VPN is running on this Mac.
  - The signed release path (key, `.sig`, publish) is not run, since it needs your signing key and password. It is listed in the summary for you to run.

## Interfaces
- No app code, Tauri commands or settings change.
- **Release output names change:**
  - `kage-vpn_<v>_aarch64.app.tar.gz` becomes `kage-vpn_<v>_universal.app.tar.gz`.
  - `latest.json` gains a `darwin-x86_64` entry.
  - Existing installs keep updating, because they look up their own key, which is still present.

## Out of scope
Windows release path (W5), signing and notarization.
