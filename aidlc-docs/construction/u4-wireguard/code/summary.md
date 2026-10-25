# U4 WireGuard: code summary

## Created
- `src-tauri/src/profile/wireguard.rs`: WireGuard (`WireGuard`, `WireGuardSettings`, `reserved_of`, `is_wg_quick`, `parse_wg_quick`).
- `src-tauri/src/tests/profile_wireguard.rs`: link spellings (unescaped keys, `+` as space), private key as a parameter, reserved in five spellings, defaults, round trip, every refusal with its message, the endpoint entry, identity without the local address, editor text and nulls, the WARP summary; wg-quick files (name comment, file name, host, ignored-lines notes, refusals, IPv6 endpoint, CRLF, repeated Address lines, detection).

## Modified
- `src-tauri/src/profile/mod.rs`: `ServerKind`, `ProtocolDef::kind` and `feature` (required), `Profile::kind`, `missing_feature`, `missing_feature_message`, `usable_nodes`; WireGuard registered; `parse_all_named` exported.
- Every protocol module: `kind` (Outbound) and `feature` (`with_quic` for Hysteria2, Hysteria, TUIC).
- `src-tauri/src/config.rs`: endpoint servers go in `endpoints` (written only when there is one); the selector lists every node tag.
- `src-tauri/src/session.rs`: inspects the engine's build features once per connect; a chosen server the engine cannot run stops the connect with a message naming the Engine page; others are left out with a log line.
- `src-tauri/src/profile/links.rs`: `parse_all_named` (wg-quick text before JSON, YAML and links; file-name hint); `wireguard`, `wg` removed from the unsupported schemes.
- `src-tauri/src/profile/formats.rs`: sing-box `endpoints` read; WireGuard in the new and old sing-box forms and both Clash forms; AmneziaWG and multi-peer refused; unknown sing-box types named.
- `src-tauri/src/commands.rs`: `import_text` takes an optional `file_name`.
- Front end: `api.ts` (`importText(text, fileName?)`, `wireguard` id), `app.svelte.ts`, `format.ts` (`isWireGuardFile`), `ImportDialog.svelte` (detects `.conf` and sing-box `endpoints`, passes a dropped file's name until the text is edited, WireGuard wording), protocol wording in the empty list and onboarding.
- Tests: `config.rs` (endpoints placement, real `sing-box check` of a WARP-style server in TUN and Proxy, engine-feature filtering and messages), `profile_conformance.rs` (endpoint address, kind/feature consistency), `profile_formats.rs`, `profile_links.rs`, `health.rs`, `scripts/tests/format.test.mjs`.
- `scripts/screenshots/protocols.json` regenerated; `demo-data.mjs` knows WireGuard.
- Docs: `ADDING-A-PROTOCOL.md` (WireGuard row, `kind`, `feature`), README, docs site (import list, pre-check note, WireGuard page and navigation entry, changelog lines for WireGuard, the engine-feature message and the protocol dropdown).

## Verified
- `cargo test --lib`: 409 passed, including the real `sing-box check` (1.14.2) of the WireGuard sample and a WARP-style server in both modes.
- `cargo clippy --all-targets -- -D warnings`: clean. `npm run check`: 0 errors. `npm run test:scripts`: 39 passed. `vite build`: ok.
- sing-box run on spare ports (21080, 29090): the endpoint is listed in the Clash API and its delay test answers. Stopped afterwards.
- Screenshot demo (headless Chrome, port 1482): pasted `.conf` text enables Add server; the WireGuard editor shows its fields, port 51820 and MTU 1408.

## Not verified
- No connection to a real WireGuard or WARP server. The live app was not run; your real VPN on 2080/19090 was not touched.
- The engine-feature stop was tested as pure functions, not with a sing-box built without WireGuard.

## Manual check
1. Drop a WireGuard `.conf` (for example a WARP export) on Add server: one server named after the file; a note lists `DNS` if the file has it.
2. Connect to it in Proxy Only mode on a spare port and open a site; run Test on it in the server list.
3. Add a server by hand with Protocol: WireGuard; leave keepalive empty and save.
