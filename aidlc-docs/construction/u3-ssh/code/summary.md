# U3 SSH: code summary

## Created
- `src-tauri/src/profile/ssh.rs`: SSH (`Ssh`, `SshSettings`, `tidy_key`).
- `src-tauri/src/tests/profile_ssh.rs`: password link with port 22, key/passphrase/host keys in the query, round-trip with a multi-line key, every rejection with its message, the "host not checked" summary, outbound contents, normalize (CRLF keys, null editor fields, passphrase without a key), secrets.
- The SSH key for the real `sing-box check` is made by `ssh-keygen` when the test runs (the test is skipped without it); earlier commits kept one in `tests/fixtures/test_ssh_key`, removed after GitGuardian flagged it.

## Modified
- `src-tauri/src/profile/form.rs`: `Kind::Multiline`, `Field::multiline`, `Field::warning_when_empty` (`warnWhenEmpty`).
- `src-tauri/src/profile/common.rs`: `null_as_empty`.
- `src-tauri/src/profile/hysteria.rs`: `auth` reads `null` as empty (bug from U2).
- `src-tauri/src/profile/mod.rs`, `links.rs`: `ssh` registered; removed from the unsupported schemes.
- `src-tauri/src/profile/formats.rs`: sing-box `ssh` outbounds (key as text or list of lines; `private_key_path` skipped) and Clash `type: ssh` (a `private-key` that is a path skipped).
- `src/lib/components/ProfileDialog.svelte`: textarea for multiline fields; warning while empty; protocol picker is a dropdown; untouched port follows the protocol's default.
- `src/lib/api.ts`: `'multiline'` field kind, `warnWhenEmpty`, `'ssh'` protocol id.
- Tests: `config.rs` (SSH with a pasted key passes `sing-box check`), `profile_form.rs`, `profile_formats.rs`, `profile_links.rs`, `health.rs`, `profile_hysteria.rs`.
- `scripts/screenshots/protocols.json` regenerated; `scripts/screenshots/demo-data.mjs` knows the new protocols.
- Wording and docs: import placeholder, empty server list, onboarding, README, `ADDING-A-PROTOCOL.md` (table row, multiline, empty warning, `null_as_empty`), docs site (import list, new SSH page and navigation entry, changelog line).

## Verified
- `cargo test --lib`: 387 passed. The real `sing-box check` (1.14.2) accepts the SSH sample (a real host key) and an SSH server with a pasted key; by hand it rejects a bad host key and a bad private key, so these are checked for real.
- `cargo clippy --all-targets -- -D warnings`: clean. `npm run check`: 0 errors. `npm run test:scripts`: 38 passed. `vite build`: ok.
- Editor in the screenshot demo (vite dev on port 1481 with the demo mock, headless Chrome): SSH shows the textarea, the host key warning while empty (gone once filled) and port 22; switching to TUIC and Hysteria keeps a typed port (2222); the protocol dropdown fits.

## Not verified
- No connection to a real SSH server; the live app was not run, and your real VPN was not touched.
- The demo page logs `unregisterListener` from the screenshot mock (it does not implement event unlistening); this is from the mock, not this change.

## Manual check
1. Servers, Add server, Enter details manually, Protocol: SSH. Port shows 22; leave the host key empty and see the warning.
2. Paste a private key (with line breaks) and save; reopen: the key keeps its lines. The list shows "SSH · key · host not checked" until a host key is added.
3. Connect through an SSH server you run, on a spare port in Proxy Only mode.
