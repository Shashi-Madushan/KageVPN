# U2 Hysteria v1: Code Generation Plan
Round 4, pull request 1. Covers FR1.2, FR1.5 and FR1.6 (Hysteria v1 part), NFR1 and NFR5. No functional design stage (skipped in the execution plan). This plan is the single source of truth for U2.

## Context
- **Path:** `docs/ADDING-A-PROTOCOL.md`, with `profile/hysteria2.rs` and U1's `profile/tuic.rs` as models (QUIC, `over_udp` true, TLS without fingerprint or Reality).
- **Depends on U1:** `common::has_port_list`, `ProtocolDef::over_udp`, `Server::link_encoded` in `formats.rs`.
- **New piece:** import notes (FR1.2 says the import result must say default speeds were used). Today `parse_many` returns profiles and skip reasons only, and the import toast is built by `importMessage` in `src/lib/format.ts`.

### What sing-box runs (outbound `hysteria`)
`up_mbps`, `down_mbps` (both required), `auth_str` (optional), `obfs` (optional, the xplus password), `tls` (required; ALPN defaults to `hysteria` in Hysteria v1 clients).

### Link format accepted
The official Hysteria v1 URI: `hysteria://host:port?protocol=udp&auth=..&peer=..&insecure=1&upmbps=10&downmbps=50&alpn=hysteria&obfs=xplus&obfsParam=..#name`. Also read: `sni` for `peer`, `auth` in the userinfo (`hysteria://auth@host:port`), `up`/`down` and `upmbps`/`downmbps` with values like `100` or `100 Mbps`, `obfs-password` for `obfsParam`, `allowInsecure` for `insecure`.

### Missing speeds (FR1.2, Question 5 answer A)
A link without `upmbps` or `downmbps` is imported with **10 Mbps up and 50 Mbps down** (only the missing one is filled). The import toast says so, the list summary shows the speeds ("Hysteria · 10/50 Mbps"), and the editor can change them. Subscriptions fill the same defaults; their refresh result does not repeat a note on every refresh (the summary shows the speeds instead).

### Rejected with a reason (FR1.5)
- `protocol` other than `udp` (`faketcp`, `wechat-video`): "Hysteria over faketcp is not supported, only udp".
- `obfs` other than `xplus`, or `xplus` without a password.
- Port hopping (`mport`, or a port list in the address).
- A speed that is not a positive number of Mbps (or Gbps, converted).

## Steps
- [x] 1. **Import notes** (`profile/mod.rs`, `profile/links.rs`): an optional `ProtocolDef::import_note(&self, link: &str, profile: &Profile) -> Option<String>` with a default of `None` (like `normalize`: most protocols have nothing to say, and forgetting it cannot weaken a connection). `parse_many` returns a third list, `notes`, one per imported server that has one, prefixed with the server name. Callers updated: `commands::import_text` (passes them on), `subscriptions.rs` (ignores them), tests.
- [x] 2. **Import summary** (`commands.rs`, `src/lib/api.ts`, `src/lib/format.ts`): `ImportSummary.notes: Vec<String>`; `importMessage` adds "1 using default speeds (…)" style text from the first note, counting them like skips. Script test in `scripts/tests/format.test.mjs` (or the existing format tests) for a summary with notes.
- [x] 3. **Module** (`profile/hysteria.rs`, new): `HysteriaSettings { auth: String, up_mbps: u32, down_mbps: u32, obfs: Option<String>, tls: Tls }`; `impl ProtocolDef for Hysteria` with:
  - `id` `"hysteria"`, `label` `"Hysteria"`, `schemes` `["hysteria"]`, `over_udp` true.
  - `parse`, `to_link` (official URI form, always writing both speeds), `import_note` (when the link lacked a speed).
  - `validate`: both speeds between 1 and 100 000 Mbps; obfs password not blank when set. Auth is optional, as in Hysteria v1.
  - `outbound`: `up_mbps`, `down_mbps`, `auth_str` when set, `obfs` when set, `tls_base` with ALPN.
  - `identity`: server, port, auth. `secrets`: auth and obfs password plus shared TLS secrets.
  - `summary`: "Hysteria · 10/50 Mbps". `form`: auth (secret, optional), upload and download speed (text with a number pattern), SNI, obfs password (optional), insecure, ALPN.
  - `blank` with 10/50 Mbps and ALPN `hysteria`; `normalize` fills the ALPN and drops a blank obfs.
  - Speeds are `u32` in the settings, so the editor's text field needs a number: check whether `Field` can hold a number key (the conformance test only checks the key exists); if the editor sends text, store and parse them as strings instead. Decided while coding, noted in the summary.
  - `sample()`.
- [x] 4. **Register** (`profile/mod.rs`) after TUIC; **links** (`profile/links.rs`): remove `"hysteria"` from `UNSUPPORTED`.
- [x] 5. **Subscriptions** (`profile/formats.rs`): a `hysteria_link` builder for sing-box JSON `"hysteria"` outbounds (`up_mbps`/`down_mbps` or `up`/`down` text, `auth_str`, `obfs`, TLS; `server_ports` skipped as port hopping; `auth` given only as base64 bytes skipped with a reason) and Clash `type: hysteria` (`up`, `down`, `auth-str`/`auth_str`, `obfs`, `protocol`, `alpn`, `sni`, `skip-cert-verify`, `ports`).
- [x] 6. **Tests**:
  - `tests/profile_hysteria.rs` (new): the official URI; userinfo auth; alternative names; speed text forms and Gbps; defaults with the note (and no note when both speeds are given); round-trip; every rejection with its message; outbound shape.
  - `tests/profile_links.rs`: the unsupported-scheme test no longer lists `hysteria`; `hysteria://` and `hysteria2://` in one paste go to the right protocols; `parse_many` notes.
  - `tests/profile_formats.rs`: one sing-box and one Clash Hysteria entry; a Clash `faketcp` entry skipped with its reason.
  - `tests/health.rs`: `hysteria` skips the TCP check.
  - Conformance suite and real `sing-box check` through the sample.
- [x] 7. **Screenshot fixture**: regenerate `scripts/screenshots/protocols.json`.
- [x] 8. **Docs**: `ADDING-A-PROTOCOL.md` (table row, `import_note`), README, docs site (import list, pre-check note, a Hysteria v1 protocol page and navigation entry, changelog line), front-end wording that lists protocols; `aidlc-docs/construction/u2-hysteria/code/summary.md`.
- [x] 9. **Verify**: `cargo test --lib -- --test-threads=4`, `cargo clippy --all-targets -- -D warnings`, `npm run check`, `npm run test:scripts`, `npx vite build`. No live-app run against your real VPN.

## Decisions made while coding
- **`parse_many` kept its two-value form.** A new `parse_all` returns `Parsed { profiles, errors, notes }`; `parse_many` wraps it. Only `import_text` needs the notes, so the 20-odd test callers did not change.
- **Speeds are numbers in saved profiles.** The editor sends text, so the speed fields read either a number or text such as "100 Mbps" (`#[serde(deserialize_with = "speed")]`); unreadable text becomes 0 and `validate` says the speed must be between 1 and 100000 Mbps.
- **Conformance exemption.** The test that a blank server needs credentials now skips `hysteria` by name, because a Hysteria v1 password is optional.

## Interfaces added
`profile::hysteria::{Hysteria, HysteriaSettings, ID}`; `ProtocolDef::import_note` (default `None`); `profile::parse_all` returning `links::Parsed` (`parse_many` unchanged); `ImportSummary.notes` (Tauri `import_text` result and `api.ts`).

## Out of scope
Port hopping, `faketcp` and `wechat-video`, `auth` as raw base64 bytes, receive-window tuning, notes on subscription refreshes.
