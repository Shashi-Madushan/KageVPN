# Adding a protocol

Each protocol is one module in `src-tauri/src/profile/` that implements `ProtocolDef`. The import box, subscriptions, share links, the server editor, validation, duplicate detection across subscription refreshes, feedback redaction and sing-box config generation all go through the registry. Every method of the trait is required, so a missing piece does not compile.

## Steps

1. **Create `profile/<name>.rs`** with a unit struct and `impl ProtocolDef`:
   - `id`, `label`, `schemes`: its stable id, display name and URL schemes.
   - `parse` / `to_link`: read and write one link. Reject anything you cannot run with a clear message rather than ignoring it; an ignored option can silently weaken the connection.
   - `validate`: protocol-specific checks. Name, address and port are already checked.
   - `outbound`: the sing-box outbound without its `tag`.
   - `identity`: the values that make it the same server after a subscription refresh.
   - `secrets`: credentials and keys that feedback reports must hide.
   - `summary`: the short description shown in lists.
   - `form`: the editor fields. Build them with `Field::text(...)` and friends, and reuse `common::transport_fields()` and the other shared groups.
   - `blank`: a new server with defaults. Optionally `normalize` to clear settings that no longer apply before saving.
2. **List it in `REGISTRY`** in `profile/mod.rs`.
3. **Define a settings struct** in the module (`#[serde(rename_all = "camelCase", default)]`) holding everything the protocol needs, reusing `common::Tls` and `common::Transport` where they fit. Build profiles with `Profile::build(ID, name, server, port, &settings)` and read them with `profile.settings_as::<YourSettings>()`. Nothing else stores or compares these fields: duplicate detection compares the whole settings object. Editor field keys are paths inside it, such as `tls.sni`.
4. **Add `tests/profile_<name>.rs`** for the protocol's own edge cases (odd link styles, links it must reject).
5. **Provide `sample()`** (an example link). The conformance suite in `tests/profile_conformance.rs` and the real `sing-box check` then cover round-trips, identity, secret redaction, editor keys and the outbound shape without further test code.
6. **Refresh the screenshot fixture.** `scripts/screenshots/protocols.json` is a generated copy of every protocol's editor fields, used by the README screenshot script. `cargo test` fails with a message when it is stale; `npm run screenshots` regenerates it before taking the pictures.

The interface needs no changes: the editor, the protocol picker, the accepted link schemes and the list descriptions are all read from the backend.

## Currently supported

| Protocol | Schemes | Notes |
|---|---|---|
| VLESS | `vless://` | TLS or Reality; TCP, WebSocket, gRPC, HTTPUpgrade |
| VMess | `vmess://` | v2rayN share format (base64 JSON). `alterId` above 0, HTTP/2, KCP and QUIC are rejected |
| Trojan | `trojan://` | TLS; same transports as VLESS |
| Shadowsocks | `ss://` | SIP002 and legacy links, including 2022 ciphers. Plugins are rejected |
| Hysteria2 | `hysteria2://`, `hy2://` | Salamander obfuscation. Port hopping and `pinSHA256` are rejected |
