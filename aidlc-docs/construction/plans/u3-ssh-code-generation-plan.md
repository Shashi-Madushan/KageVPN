# U3 SSH: Code Generation Plan
Round 4, pull request 1. Covers FR1.4, FR1.5, FR1.6 (SSH part), FR1.7, FR1.8, NFR1 and NFR5. No functional design stage (skipped in the execution plan). This plan is the single source of truth for U3.

## Context
- **Path:** `docs/ADDING-A-PROTOCOL.md`. SSH runs over TCP (`over_udp` false), has no TLS and no transport block, and its default port is 22, not 443.
- **Depends on U1 and U2:** `over_udp`, `formats.rs` builders, `link_encoded`. No dependency on U4.
- **Two editor gaps** found while reading `ProfileDialog.svelte`:
  1. No multi-line field: every text field is a single-line `<input>`, so a pasted private key would lose its line breaks.
  2. A field `warning` is only shown for a toggle that is on. SSH needs a warning while the host key field is **empty** (Question 7, answer A).
- **The server list** shows `profile.summary` under each server (`ProfilesPage.svelte`), so the "host not checked" warning in the list is the summary text; no list code changes.

### What sing-box runs (outbound `ssh`)
`server`, `server_port`, `user`, `password`, `private_key` (the key text), `private_key_passphrase`, `host_key` (list of `algorithm base64` lines). `private_key_path` exists but is out of scope (Question 6, answer A: pasted keys only).

### Link format accepted
There is no single standard. Accepted: `ssh://user:password@host:port#name`, plus optional query parameters `private_key` (or `pk`, the key text, percent-encoded), `passphrase` (or `pkp`), `host_key` (or `hk`, one or more keys separated by commas). Written back in the same form.

### Rejected with a reason (FR1.5)
- No user name.
- Neither a password nor a private key.
- A private key that is not a PEM or OpenSSH key ("-----BEGIN … PRIVATE KEY-----").
- A host key that is not `algorithm base64` (for example a fingerprint such as `SHA256:…`): "host key must be the full public key line, not a fingerprint".
- From subscriptions: `private_key_path` (sing-box) or a Clash `private-key` that is a file path: "SSH key files on disk are not supported, paste the key into the server editor".

## Steps
- [x] 1. **Multi-line secret field** (`profile/form.rs`, `ProfileDialog.svelte`, `src/lib/api.ts`): `Kind::Multiline` and `Field::multiline(key, label, section)` (monospace). The editor renders a `<textarea>` (about 5 rows), keeps the value untrimmed like a secret, and treats an empty optional value as null. `FieldKind` in `api.ts` gains `'multiline'`.
- [x] 2. **Warning while empty** (`profile/form.rs`, `ProfileDialog.svelte`, `api.ts`): `Field::warning_when_empty(text)` sets the existing `warning` plus `warnWhenEmpty: true`; the editor shows it under a text, list or multiline field while that field is empty. The existing toggle behaviour is unchanged.
- [x] 3. **Module** (`profile/ssh.rs`, new): `SshSettings { user, password, private_key, passphrase, host_keys: Vec<String> }`; `impl ProtocolDef for Ssh` with:
  - `id` `"ssh"`, `label` `"SSH"`, `schemes` `["ssh"]`, `over_udp` false.
  - `parse`, `to_link` as above; default port 22.
  - `validate`: the rejections above (key shape, host key shape, user, password or key).
  - `outbound`: `user`, then `password` and/or `private_key` and `private_key_passphrase` when set, `host_key` when any.
  - `identity`: server, port, user. `secrets`: password, private key, passphrase (host keys are public).
  - `summary`: "SSH · key" or "SSH · password", plus " · host not checked" with no host key.
  - `form`: user (text), password (secret, optional), private key (multiline, optional), passphrase (secret, optional, shown in Advanced), host keys (list, optional, `warning_when_empty`: "Without a host key, anyone who can intercept your connection could pretend to be this server. Ask your provider for the server's public host key.").
  - `blank` with port 22; `normalize` trims the user and host keys, drops empty ones, and normalises key line endings to `\n` with a final newline.
  - `sample()`: password plus one documentation host key (no real private key in the sample).
- [x] 4. **Register** (`profile/mod.rs`) and **links** (`profile/links.rs`): remove `"ssh"` from `UNSUPPORTED`. The unsupported-scheme test switches to another scheme it still lists.
- [x] 5. **Subscriptions** (`profile/formats.rs`): an `ssh_link` builder for sing-box `"ssh"` outbounds (`user`, `password`, `private_key` as text or a list of lines, `private_key_passphrase`, `host_key`; `private_key_path` skipped) and Clash `type: ssh` (`username`, `password`, `private-key` as key text, `private-key-passphrase`, `host-key`; a path skipped). `Server::read` falls back to port 22 for SSH entries without one? No: entries without a port are skipped as today, for consistency.
- [x] 6. **Tests**:
  - `tests/profile_ssh.rs` (new): password link; key and passphrase and host keys in the query; round-trip with a multi-line key; every rejection with its message; host key warning in the summary; outbound with and without each part; normalize of CRLF keys; default port 22.
  - `tests/profile_form.rs`: the new kind and the empty warning serialize as the editor expects (`"multiline"`, `warnWhenEmpty`).
  - `tests/profile_formats.rs`: one sing-box and one Clash SSH entry; key paths skipped with the reason.
  - `tests/profile_links.rs`: `ssh://` imported; mixed paste includes it.
  - `tests/health.rs`: `ssh` keeps the TCP check.
  - Conformance suite and real `sing-box check` through the sample (with a test-only generated key added to a config test, so the `private_key` path is checked by sing-box too, if sing-box validates key text at check time).
- [x] 7. **Screenshot fixture**: regenerate `scripts/screenshots/protocols.json`.
- [x] 8. **Docs**: `ADDING-A-PROTOCOL.md` (table row, `Field::multiline`, `warning_when_empty`), README, docs site (import list, an SSH protocol page and navigation entry, changelog line), front-end wording that lists protocols; `aidlc-docs/construction/u3-ssh/code/summary.md`.
- [x] 9. **Verify**: `cargo test --lib -- --test-threads=4`, `cargo clippy --all-targets -- -D warnings`, `npm run check`, `npm run test:scripts`, `npx vite build`, and a look at the editor in the dev build for the textarea and the empty-field warning (front end only, no connection). No live-app run against your real VPN.

- [x] 10. **Added during generation: fixes found by the editor check.**
  - **U2 bug:** the editor sends `null` for an optional field left empty, and Hysteria's optional password was a plain `String`, so saving a password-less Hysteria server from the editor would have failed. New `common::null_as_empty`, used by Hysteria `auth` and the SSH optional texts; test `an_empty_password_from_the_editor_saves`.
  - **Protocol picker:** eight protocols overflowed the row of buttons and made the dialog scroll sideways; the picker is now a dropdown (`Select`).
  - **Port on protocol switch:** a new server switched to SSH kept 443. An untouched port now moves to the new protocol's default (22 for SSH); a typed port is kept.
  - **Screenshot demo mock** (`scripts/screenshots/demo-data.mjs`): `new_profile` knows TUIC, Hysteria and SSH.

## Interfaces added
`profile::ssh::{Ssh, SshSettings, ID}`; `form::Kind::Multiline`, `Field::multiline`, `Field::warning_when_empty` and the `warnWhenEmpty` field property (Tauri `protocols` result and `api.ts`); `common::null_as_empty`.

## Out of scope
Key files on disk, keyboard-interactive login, host key algorithms list, client version string, the Keychain (Question 8, answer A: same store as today).
