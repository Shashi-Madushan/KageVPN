# U2 Hysteria v1: code summary

## Created
- `src-tauri/src/profile/hysteria.rs`: Hysteria v1 (`Hysteria`, `HysteriaSettings`, `parse_speed`).
- `src-tauri/src/tests/profile_hysteria.rs`: official link form, other spellings, speed parsing, defaults and notes, no-password servers, round-trip, every rejection with its message, editor text speeds, outbound, summary.

## Modified
- `src-tauri/src/profile/mod.rs`: `hysteria` in `REGISTRY`; optional `ProtocolDef::import_note`; `parse_all` exported.
- `src-tauri/src/profile/links.rs`: `Parsed` and `parse_all` (with notes); `parse_many` wraps it; `hysteria` removed from the unsupported schemes.
- `src-tauri/src/profile/formats.rs`: sing-box `hysteria` outbounds (`up_mbps`/`up`, `down_mbps`/`down`, `auth_str`, `obfs`; binary `auth` and `server_ports` skipped) and Clash `type: hysteria` entries (`up`, `down`, `auth-str`, `obfs`, `protocol`, `ports`).
- `src-tauri/src/commands.rs`: `ImportSummary.notes`.
- `src/lib/api.ts`, `src/lib/format.ts`: `notes` in the import summary; the import toast adds "note (…)" or "N notes (…)".
- Tests: `profile_links.rs`, `profile_formats.rs`, `profile_conformance.rs` (Hysteria exempt from "blank needs credentials"), `health.rs`, `scripts/tests/format.test.mjs`.
- `scripts/screenshots/protocols.json`: regenerated.
- Wording: import placeholder, empty server list, onboarding, README, `ADDING-A-PROTOCOL.md` (table row, `import_note`), docs site (import list, pre-check note, new Hysteria v1 page and navigation entry, changelog line).

## Verified
- `cargo test --lib`: 375 passed, including the real `sing-box check` (1.14.2) with the Hysteria sample; a separate `sing-box check` on a Hysteria and a TUIC outbound printed no warnings.
- `cargo clippy --all-targets -- -D warnings`: clean. `npm run check`: 0 errors. `npm run test:scripts`: 38 passed. `vite build`: ok.

## Not verified
- No connection to a real Hysteria v1 server. The live app was not run; your real VPN was not touched.

## Manual check
1. Paste `hysteria://host:443?auth=pw#NoSpeeds`: the toast says "Imported 1 profile, note (NoSpeeds: no speeds in the link, using 10 Mbps up and 50 Mbps down; …)" and the list shows "Hysteria · 10/50 Mbps".
2. Open the editor, type `1 Gbps` as the download speed, save: the list shows "Hysteria · 10/1000 Mbps".
3. Paste a `protocol=faketcp` link: skipped with "Hysteria over faketcp is not supported, only udp".
