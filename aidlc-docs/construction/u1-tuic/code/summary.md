# U1 TUIC: code summary

## Created
- `src-tauri/src/profile/tuic.rs`: the TUIC v5 protocol (`Tuic`, `TuicSettings`).
- `src-tauri/src/tests/profile_tuic.rs`: parsing, defaults, alternative parameter names, round-trip, every rejection with its message, validate, normalize, outbound shape, summary.

## Modified
- `src-tauri/src/profile/mod.rs`: `tuic` in `REGISTRY`; new required `ProtocolDef::over_udp`.
- `src-tauri/src/profile/{vless,vmess,trojan,shadowsocks,hysteria2}.rs`: `over_udp` (Hysteria2 true); Hysteria2 uses the shared port-list check.
- `src-tauri/src/profile/common.rs`: `has_port_list` moved here from `hysteria2.rs`.
- `src-tauri/src/profile/links.rs`: `tuic` removed from the unsupported schemes.
- `src-tauri/src/profile/formats.rs`: sing-box JSON `tuic` outbounds and Clash `type: tuic` entries become `tuic://` links (Clash v4 `token` and `disable-sni` skipped with reasons); `Server::link_encoded` for a `user:password` userinfo.
- `src-tauri/src/health.rs`: the TCP pre-check asks the registry whether a protocol is UDP.
- Tests: `profile_formats.rs`, `profile_links.rs`, `profile_hysteria2.rs`, `health.rs`.
- `scripts/screenshots/protocols.json`: regenerated.
- Front end (text only): `api.ts` protocol type, import placeholder, empty server list text, onboarding text.
- Docs: `docs/ADDING-A-PROTOCOL.md` (table row, `over_udp`), `README.md`, docs site (user guide import list and pre-check note, new TUIC protocol page and its navigation entry, changelog entry under 21 October).

## Verified
- `cargo test --lib`: 363 passed, including the conformance suite and the real `sing-box check` (sing-box 1.14.2) with the TUIC sample in TUN and Proxy configs.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `npm run check`: 0 errors, 0 warnings. `npm run test:scripts`: 37 passed. `vite build`: ok.

## Not verified
- No connection to a real TUIC server (none available). The live app was not run, so your real VPN on 2080/19090 was not touched.

## Manual check
1. Import a TUIC v5 link from your provider (or a test server), connect in Proxy Only mode on a spare port, and open a site.
2. Paste `tuic://sometoken@host:443`: it is listed as skipped with "TUIC v4 is not supported".
3. Open the editor for the TUIC server: UUID, password, server name, congestion control, UDP relay, 0-RTT and ALPN fields show.
