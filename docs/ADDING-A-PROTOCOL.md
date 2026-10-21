# Adding a protocol

Each protocol is one module in `src-tauri/src/profile/` that implements `ProtocolDef`. The import box, subscriptions, share links, the server editor, validation, duplicate detection across subscription refreshes, feedback redaction and sing-box config generation all go through the registry. Every method of the trait is required, so a missing piece does not compile.

## Steps

1. **Create `profile/<name>.rs`** with a unit struct and `impl ProtocolDef`:
   - `id`, `label`, `schemes`: its stable id, display name and URL schemes.
   - `over_udp`: whether it reaches the server over UDP (QUIC). The pre-connect check skips its TCP test for these.
   - `kind`: `ServerKind::Outbound`, or `ServerKind::Endpoint` for a protocol sing-box runs as an endpoint (WireGuard); the config builder puts it in the matching list and the selector refers to it by tag either way.
   - `feature`: the sing-box build feature it needs beyond the engine's required ones (`with_quic`, `with_wireguard`), or `None`. Servers whose feature the engine lacks are left out when connecting, and choosing one stops with a message naming the Engine page.
   - `parse` / `to_link`: read and write one link. Reject anything you cannot run with a clear message rather than ignoring it; an ignored option can silently weaken the connection.
   - `validate`: protocol-specific checks. Name, address and port are already checked.
   - `outbound`: the sing-box outbound without its `tag`.
   - `identity`: the values that make it the same server after a subscription refresh.
   - `secrets`: credentials and keys that feedback reports must hide.
   - `summary`: the short description shown in lists.
   - `form`: the editor fields. Build them with `Field::text(...)` and friends (`Field::multiline` for text that must keep its line breaks, `.warning_when_empty(...)` for an optional value that is risky to skip), and reuse `common::transport_fields()` and the other shared groups. An optional text setting stored as a plain `String` needs `#[serde(deserialize_with = "common::null_as_empty")]`, because the editor sends `null` for an empty optional field.
   - `blank`: a new server with defaults. Optionally `normalize` to clear settings that no longer apply before saving, and `import_note` to tell the user about a default the import filled in (shown in the import result).
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
| Hysteria (v1) | `hysteria://` | UDP only, `xplus` obfuscation, password optional. Missing speeds become 10 Mbps up / 50 Mbps down with an import note. `faketcp`, `wechat-video`, port hopping and binary `auth` keys are rejected |
| SSH | `ssh://` | Password or a pasted private key (with passphrase), optional host keys; port 22 by default. The link carries the key, passphrase and host keys as `private_key`, `passphrase` and `host_key` (or `pk`, `pkp`, `hk`). Key files on disk and host key fingerprints are rejected |
| WireGuard | `wireguard://`, `wg://`, wg-quick `.conf` | A sing-box endpoint, not an outbound. One peer; pre-shared key, MTU (1408 by default), keepalive and WARP `reserved` bytes. `.conf` `DNS`, narrower `AllowedIPs` and `PostUp`-style lines are ignored with an import note. Several peers and AmneziaWG options are rejected |
| TUIC | `tuic://` | v5 only (UUID and password); BBR, CUBIC or NewReno; native or QUIC UDP relay; 0-RTT. ALPN defaults to `h3`. v4 links, `disable_sni` and port hopping are rejected |
