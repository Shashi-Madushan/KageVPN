# U1 TUIC: Code Generation Plan
Round 4, pull request 1. Covers FR1.1, FR1.5 and FR1.6 (TUIC part) and NFR1, NFR5. No functional design stage (skipped in the execution plan). This plan is the single source of truth for U1.

## Context
- **Path:** the documented one in `docs/ADDING-A-PROTOCOL.md`. TUIC is QUIC-based like Hysteria2, so `profile/hysteria2.rs` is the model: TLS without browser fingerprint or Reality, no transport block.
- **Dependencies:** none on other units. U2 to U4 reuse what this unit adds to `formats.rs`.
- **Interfaces:** no new Tauri commands, settings or front-end code. The picker, editor, schemes and list summary come from the registry.
- **Subscriptions:** `formats.rs` turns each sing-box JSON or Clash YAML entry into a share link and parses it with the protocol, so subscription support is one link builder per format.

### What sing-box runs (outbound `tuic`, v5 only)
`uuid`, `password`, `congestion_control` (`cubic`, `new_reno`, `bbr`), `udp_relay_mode` (`native`, `quic`), `zero_rtt_handshake`, `tls` (required: `server_name`, `alpn`, `insecure`).

### Link format accepted
`tuic://<uuid>:<password>@host:port?sni=..&alpn=h3&congestion_control=bbr&udp_relay_mode=native&allow_insecure=1#name` (v2rayN, NekoBox and Hiddify style). Also read: `insecure=1`, `congestion_controller`, `reduce_rtt` / `zero_rtt_handshake`.

### Rejected with a reason (FR1.5)
- TUIC v4: no password in the userinfo, or a `token` parameter, or `version=4` ("TUIC v4 is not supported, ask for a v5 link").
- A user part that is not a UUID.
- An unknown congestion control or UDP relay mode.
- `disable_sni=1` (sing-box always sends the name; dropping it would change the handshake).
- Port lists (`443,5000-6000`), using the same check as Hysteria2.

## Steps
- [x] 1. **Shared port-list check** (`profile/common.rs`, `profile/hysteria2.rs`): move `has_port_list` from `hysteria2.rs` to `common.rs` unchanged so TUIC (and U2) use it; Hysteria2 calls the shared one. Its existing test moves with it to `tests/profile_links.rs` or stays in `profile_hysteria2.rs` calling `common::has_port_list`.
- [x] 2. **Module** (`profile/tuic.rs`, new): `TuicSettings { uuid, password, congestion_control, udp_relay_mode, zero_rtt: bool, tls: Tls }` (`#[serde(rename_all = "camelCase", default)]`); `impl ProtocolDef for Tuic` with:
  - `id` `"tuic"`, `label` `"TUIC"`, `schemes` `["tuic"]`.
  - `parse` and `to_link` as above; defaults `bbr` and `native` when the link leaves them out (what most servers use and what NekoBox assumes).
  - `validate`: UUID shape, non-empty password, known choices.
  - `outbound`: the fields above; `tls` from `common::tls_base` with no fingerprint.
  - `identity`: server, port, UUID.
  - `secrets`: UUID and password (plus the shared TLS secrets).
  - `summary`: "TUIC · bbr" (congestion control).
  - `form`: UUID (secret), password (secret), SNI, congestion control (select), UDP relay mode (segmented), zero-RTT (toggle), insecure, ALPN.
  - `blank` with `bbr`, `native`, ALPN `h3`; `normalize` lowercases the choices and fills defaults.
  - `sample()` with documentation-range values.
- [x] 3. **Register** (`profile/mod.rs`): `pub mod tuic;` and `&tuic::Tuic` in `REGISTRY` (after Hysteria2).
- [x] 4. **Links** (`profile/links.rs`): remove `"tuic"` from `UNSUPPORTED`.
- [x] 5. **Subscriptions** (`profile/formats.rs`): a `tuic_link` builder used by
  - sing-box JSON `"tuic"` outbounds (`uuid`, `password`, `congestion_control`, `udp_relay_mode`, `zero_rtt_handshake`, TLS block);
  - Clash YAML `type: tuic` (`uuid`, `password`, `congestion-controller`, `udp-relay-mode`, `reduce-rtt`, `alpn`, `sni`, `skip-cert-verify`, `disable-sni`); a Clash entry with `token` (v4) is skipped with the v4 reason.
- [x] 6. **Tests**:
  - `tests/profile_tuic.rs` (new): parses a full link; defaults when options are missing; the alternative parameter names; round-trip through `to_link`; every rejection in the list above has a case and checks the message; the outbound has no `utls` or `reality` and carries the choices.
  - `tests/profile_formats.rs`: one sing-box JSON and one Clash TUIC entry convert; a Clash v4 entry is skipped with its reason.
  - `tests/profile_links.rs`: a `tuic://` link in a mixed subscription is imported, not reported as unsupported.
  - The conformance suite and the real `sing-box check` pick up the sample automatically.
- [x] 7. **Screenshot fixture**: regenerate `scripts/screenshots/protocols.json` (the stale-fixture test says how).
- [x] 8. **Docs**: `docs/ADDING-A-PROTOCOL.md` supported table (TUIC row with its limits), the user guide's protocol list in `docs-site`, README protocol list, changelog entry under 21 October (not released); `aidlc-docs/construction/u1-tuic/code/summary.md` with what changed and a manual check (import a TUIC v5 link, connect in Proxy Only mode on a spare port if a test server is available).
- [x] 9. **Verify**: `cargo test --lib -- --test-threads=4`, `cargo clippy --all-targets -- -D warnings`, `npm run check` (svelte-check), `npm run test:scripts`, `npx vite build`. No live-app run against your real VPN; any runtime check uses a spare port.

- [x] 10. **Added during generation: UDP protocols skip the TCP pre-check** (`profile/mod.rs`, every protocol module, `health.rs`). `health::needs_tcp_check` named only `hysteria2`, so a TUIC server would have failed the pre-connect TCP test. `ProtocolDef` gains a required `over_udp()` (true for Hysteria2 and TUIC) and the check asks the registry, so U2 and U4 cannot forget it. Test in `tests/health.rs`.

## Interfaces added
`profile::tuic::{Tuic, TuicSettings, ID}`; `common::has_port_list` (moved, `pub` inside the private `common` module); `ProtocolDef::over_udp`. Saved profiles gain `protocol: "tuic"` with the settings above; no migration needed.

## Out of scope
TUIC v4, `udp_over_stream`, heartbeat interval, certificate pinning, port hopping.
