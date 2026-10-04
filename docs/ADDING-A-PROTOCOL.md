# Adding a protocol

Each protocol is one module in `src-tauri/src/profile/` that describes itself with a `ProtocolSpec`. The import box, subscriptions, share links, validation, duplicate detection and sing-box config generation all go through the registry, so they need no changes.

## Backend

1. **Create `profile/<name>.rs`** with a `pub static SPEC: ProtocolSpec` and four functions:
   - `parse(&str) -> Result<Profile>`: read one link. Reject anything you cannot run with a clear message rather than ignoring it (an ignored option can silently weaken the connection).
   - `to_link(&Profile) -> String`: write it back. A test should check `parse(to_link(p))` matches `p` with `same_endpoint`.
   - `validate(&Profile) -> Result<()>`: protocol-specific checks. Name, address and port are already checked.
   - `outbound(&Profile) -> Value`: the sing-box outbound without its `tag`.
2. **Use `profile/common.rs`** for query parameters, names, TLS and transport blocks. Add to it only what two protocols share.
3. **Add a `Protocol` variant** and list `&<name>::SPEC` in `REGISTRY` in `profile/mod.rs`.
4. **Add profile fields** to `Profile` only if the protocol needs new ones. Give them defaults so saved profiles still load, and include any secret in `same_endpoint` and in the redaction list in `feedback.rs`.
5. **Add `tests/profile_<name>.rs`** and one sample link to `every_protocol()` in `tests/config.rs`. That test runs the real `sing-box check` when sing-box is installed, which catches wrong outbound field names.

## Interface

Add an entry to `PROTOCOLS` in `src/lib/protocols.ts` and the protocol to the `Protocol` type in `src/lib/api.ts`. Then show its fields in `ProfileDialog.svelte`.

## Currently supported

| Protocol | Schemes | Notes |
|---|---|---|
| VLESS | `vless://` | TLS or Reality; TCP, WebSocket, gRPC, HTTPUpgrade |
| Trojan | `trojan://` | TLS; same transports as VLESS |
| Shadowsocks | `ss://` | SIP002 and legacy links, including 2022 ciphers. Plugins are rejected |
| Hysteria2 | `hysteria2://`, `hy2://` | Salamander obfuscation. Port hopping and `pinSHA256` are rejected |
