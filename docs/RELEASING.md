# Releasing

How a new version of the app gets built, signed and offered to people who already have it.

## The pieces

| Piece | Where | Notes |
|---|---|---|
| Private signing key | `~/.tauri/singbox-gui-updater.key` | Never committed (`*.key` is git-ignored). Password-protected. |
| Key password | macOS Keychain, item `singbox-gui-updater-key` | Or set `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` for the run. |
| Public key | `APP_UPDATE_PUBKEY` in `src-tauri/src/app_config.rs` | Compiled into every build. Installed apps only accept updates signed by the matching private key. |
| Update feed URL | `APP_UPDATE_FEED` in `app_config.rs` | `None` until you pick a host. While it is `None` the app never checks for updates. |
| Release script | `scripts/release.mjs` (`npm run release`) | Builds, signs and stages; publishes only with `--publish`. |

## Back up the key

If the private key or its password is lost, no installed copy can ever accept an update again,
and the only way out is asking everyone to reinstall by hand. Keep copies of both somewhere safe
and separate (a password manager is ideal):

- the file `~/.tauri/singbox-gui-updater.key`
- the password: `security find-generic-password -s singbox-gui-updater-key -w`

Anyone who gets both can publish updates that every installed copy will trust. Do not share
them, and do not paste them into chats, issues or CI logs.

## One-time setup

1. **Choose a host** and set `APP_UPDATE_FEED` in `app_config.rs` to the URL of `latest.json`.
   For GitHub Releases that is
   `https://github.com/<owner>/<repo>/releases/latest/download/latest.json`
   (the script prints it when you pass `--github owner/repo`).
2. Make sure `gh auth status` works if you want the script to publish to GitHub.

## Cutting a release

```bash
# 1. Build, sign and stage (nothing leaves your machine)
npm run release -- --version 0.2.0 --github owner/repo --notes "What changed"

# 2. Look at release/0.2.0/, then publish
npm run release -- --version 0.2.0 --github owner/repo --notes "What changed" --skip-build --publish
```

What it does:

1. Checks the working tree is clean, the signing key exists, and that **the public key in
   `app_config.rs` matches the signing key**. A mismatch stops the release, because shipping it
   would strand every installed copy.
2. Sets the version in `tauri.conf.json`, `package.json` and `Cargo.toml`. Commit that change.
3. Runs `tauri build --bundles app`, which produces the app and a signed update archive
   (`.app.tar.gz` plus its `.sig`). Add `--dmg` to also build the disk image for first installs;
   it needs a normal desktop session, and if it fails the release carries on without it.
4. Copies them to `release/<version>/` with plain file names, and writes `latest.json` with the
   signature and download URL for this Mac's CPU. Run it on an Intel Mac too (same version) and
   that CPU is added to the same `latest.json`.
5. With `--publish`: creates the GitHub release `v<version>` (or adds to it) with all of those files.

Other options: `--dmg`, `--base-url <url>` for a host other than GitHub, `--notes-file`, `--draft`,
`--allow-dirty`, and `--dry-run` to print the plan without building.

## Checking a release

- Open `latest.json`: `version` is the new version and `platforms` has your CPU.
- From an older installed build, Settings → App updates → **Check now** should offer it.
- `npm run test:scripts` covers the script's helpers, and `cargo test` covers the updater
  against a local fake feed, including rejection of a tampered download.

## Not covered yet

- **Apple signing and notarization.** The update itself is verified by the key above, but macOS
  also checks the app. Without an Apple Developer ID, first installs show a Gatekeeper warning
  (right-click, Open). Add the certificate and notarization settings to the Tauri bundle config
  when you have an account.
- **Downgrade protection.** The updater has a `requireSignedVersion` option that rejects an old
  artifact served under a newer version number. Turn it on in `tauri.conf.json`
  (`plugins.updater`) once you have confirmed that a release built by this script carries its
  version in the signature, and that earlier releases can still update.
- **Key rotation.** There is no automatic way to change keys. A new key means shipping one
  last release signed with the old key that contains the new public key.
