# A1 Universal build: summary

## Changed
- `scripts/release-lib.mjs`: `UNIVERSAL_TARGET`, `UNIVERSAL_PLATFORMS`, `missingTargets()`; `buildManifest` accepts `platforms` (several keys share one file).
- `scripts/release.mjs`: checks both Rust targets (stops with the `rustup target add` command; `--dry-run` only reports), builds `--target universal-apple-darwin`, reads the bundle from `target/universal-apple-darwin/release/bundle/`, stages `kage-vpn_<v>_universal.app.tar.gz` and `.dmg`, writes `latest.json` with both `darwin-aarch64` and `darwin-x86_64`.
- `scripts/tests/release-lib.test.mjs`: three tests (missing targets, universal names, two-key manifest that keeps other platforms).
- Docs: `docs/RELEASING.md`, README build note, changelog, `docs/AUDIT-2026-10.md` O6 marked done.
- This Mac: `rustup target add x86_64-apple-darwin x86_64-pc-windows-msvc` (the Windows one is for Part B).

## Verified
- `npm run test:scripts`: 49 passed.
- `node scripts/release.mjs --dry-run`: prints the universal plan, no missing targets.
- Unsigned universal build: `lipo -archs` on `Kage.app/Contents/MacOS/kage-vpn` prints `x86_64 arm64`; the app is 21.9 MB (two architectures).
- The built app was not launched (live VPN on this Mac).

## For you to run
- A signed release: `npm run release -- --version <v> --github Shashi-Madushan/KageVPN --notes "..."`, then open `release/<v>/latest.json` and check both keys.
- Launch the universal app once on an Intel Mac if you have one.
