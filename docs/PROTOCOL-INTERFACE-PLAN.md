# Plan: a protocol interface that needs no edits elsewhere

## Progress

Decisions:
- New design only. No compatibility with older app versions and no data migration. Saved servers in the old format are not converted; the store keeps the old file as `data.v3.json` and servers are re-imported.
- VLESS is the reference protocol. Each step is built and tested against VLESS first (it is the most complex: TLS, Reality, flow, four transports), then the other three follow. Step 5 (a brand-new protocol) stays as the final proof.

| Step | Status |
|---|---|
| 1. Trait and string id | Done |
| 2. Form descriptors and the generic editor | Done |
| 3. Settings block | To do |
| 4. Conformance test | To do |
| 5. Prove it with a new protocol | To do |

Today a new protocol is one module plus about six edits in other files (see "Where it falls short"). The goal is one module plus one registry line.

Step order changed: the editor (old step 3) now comes before the settings block (old step 2). The editor reads and writes fields by key path (`sni`, `transport.path`), so moving the stored data into a `settings` block later only changes the keys in each protocol's field list, not the editor. Doing the data change first would have left the app broken between commits.

Step 2 notes: each protocol also provides `blank()` (defaults for a new server) and an optional `normalize()` (clears settings that stopped applying, fills required defaults). `protocols.json` under `scripts/screenshots/` is a copy of the real field lists for the screenshot mock, and a test fails if it goes stale.

## Where it falls short today

| Edit outside the module | Why it is needed |
|---|---|
| `Protocol` enum variant | Protocols are identified by an enum |
| `Profile` fields | One flat struct holds every protocol's fields |
| `same_endpoint` | Lists the comparable fields by hand |
| Subscription refresh key in `store.rs` | Lists the identifying fields by hand |
| Secret list in `feedback.rs` | A missed secret leaks into feedback reports |
| `api.ts`, `protocols.ts`, `ProfileDialog.svelte` | The form and type are written per protocol |

Two of these fail silently (a missed duplicate field, a missed secret). The design below makes them compile errors or removes them.

## Design

### 1. A trait instead of a struct of function pointers

```rust
pub trait ProtocolDef: Sync {
    fn id(&self) -> &'static str;                 // "vless", stored in saved profiles
    fn label(&self) -> &'static str;              // "VLESS"
    fn schemes(&self) -> &'static [&'static str];
    fn parse(&self, link: &str) -> Result<Profile>;
    fn to_link(&self, p: &Profile) -> String;
    fn validate(&self, p: &Profile) -> Result<()>;
    fn outbound(&self, p: &Profile) -> Value;      // sing-box outbound, no tag
    fn identity(&self, p: &Profile) -> String;     // survives a subscription refresh
    fn secrets(&self, p: &Profile) -> Vec<String>; // redacted from feedback
    fn summary(&self, p: &Profile) -> String;      // "Reality · grpc" in lists
    fn form(&self) -> Vec<Field>;                  // what the editor shows
}
pub static REGISTRY: &[&dyn ProtocolDef] = &[&Vless, &Trojan, &Shadowsocks, &Hysteria2];
```

Forgetting `secrets` or `identity` no longer compiles. The protocol id becomes a string, so the enum and its edits disappear.

### 2. Common fields plus a per-protocol settings block

```rust
pub struct Profile {
    id, protocol: String, name, server, port, group, favorite, subscription_id,
    settings: serde_json::Map<String, Value>,   // uuid, password, method, obfs, tls, transport ...
}
```

Each module defines its own typed settings struct and reads it with serde (`VlessSettings`, `TrojanSettings`). `Profile` never changes again. `same_endpoint` compares the common fields and the settings map, so it needs no per-protocol code. Shared pieces (TLS, transport) stay in `common.rs` as reusable structs that protocol settings embed.

### 3. The backend describes the editor

```rust
pub struct Field {
    key: &'static str,        // path inside settings, e.g. "tls.sni"
    label: &'static str,
    kind: Text | Secret | Number | Select(&[..]) | Toggle | Segmented(&[..]),
    section: Connection | Security | Transport | Advanced,
    optional: bool,
    placeholder: Option<&'static str>,
    warning: Option<&'static str>,     // shown when the field is on
    show_when: Option<(&'static str, &'static str)>,  // e.g. ("tls.mode","reality")
}
```

`common::tls_fields()` and `common::transport_fields()` return the shared groups, so VLESS and Trojan reuse them. A new `protocols` command returns each protocol's id, label, schemes and fields. `ProfileDialog.svelte` renders any protocol from that list. `protocols.ts`, the `Protocol` type and the `{#if}` blocks are deleted. The import box, clipboard watcher and list labels also read schemes and summaries from the backend.

Validation stays in Rust. The form only disables Save while a required field is empty, and shows the backend's message on save.

### 4. One conformance test for every protocol

A single test iterates `REGISTRY` and, for each protocol's sample link, checks:
- parse, then `to_link`, then parse gives the same endpoint
- `validate` passes
- `outbound` passes the real `sing-box check`
- `secrets` is non-empty for any protocol with a credential, and `redact` removes them
- `identity` is stable across a re-parse of the same link
- every `form()` field key exists in the settings the parser produces

A protocol author adds a sample link; the test then covers everything above.

## Steps

1. **Trait and string id.** Convert the four modules, keep the flat `Profile`, add `identity`, `secrets`, `summary`. Behaviour does not change. Existing tests must pass unchanged.
2. **Settings block and migration.** Schema version 4 moves `uuid`, `password`, `method`, `obfs`, `allow_insecure`, `flow`, TLS and transport fields into `settings`. The store already writes `data.vN.json` before migrating. Test with a saved v3 file that has every protocol and a subscription.
3. **Form descriptors and the generic editor.** Add `Field`, the `protocols` command, the generic renderer. Delete `protocols.ts` and the per-protocol blocks. Compare screenshots of the editor for all four protocols before and after.
4. **Conformance test.** Replace the scattered per-protocol round-trip checks with the registry-wide test. Keep each protocol's tests for its own edge cases.
5. **Prove it.** Add VMess (a different link format, base64 JSON) using only a new module, a registry line and a sample link. If any other file needs editing, fix the interface and repeat. Record the diff size in `ADDING-A-PROTOCOL.md`.

Each step is its own commit and leaves the app working.

## Risks

- **Data migration (step 2).** A bug would break saved servers. Mitigations: automatic backup, a migration that can run twice safely, and a fixture test with real v3 data.
- **Generic form loses polish (step 3).** Special cases such as "Reality switches the default flow" need a small hook. Allow a protocol to supply `on_change` defaults in Rust rather than adding UI special cases.
- **Looser frontend types.** `settings` is untyped on the TypeScript side. Access goes through field keys from the descriptor, so the types only need to describe the common fields.

## Alternatives considered

- **Keep the flat struct and enum.** Works for four protocols. Each new protocol edits `Profile`, the form and the secret list, and a missed secret is a privacy bug.
- **Compile-time macro or derive for forms.** Cleaner to write, but harder to read and debug than a plain list of fields. Revisit if the field lists get long.

## Open questions

- Is VMess the right proof for step 5, or would you rather prove it with WireGuard or TUIC (more work, different settings)?
- Do you want older app versions to read data saved after the migration? Plan assumes no: the backup is the way back.
