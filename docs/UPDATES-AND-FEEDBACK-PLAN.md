# Plan: app updates, multi-core downloads, feedback

Status: planned, not started. Revised to cover several core types and a backend feedback endpoint.

## 1. Single config file: `src-tauri/src/app_config.rs`

Plain Rust constants. No `.env` files. Everything an operator would change lives here.

| Group | Contents |
|---|---|
| Identity | `APP_NAME`, `APP_SLUG`, `APP_VERSION` (from Cargo), `user_agent()` |
| Core registry | `CORES: &[CoreSpec]`, see section 2 |
| App updates | `APP_UPDATE_FEED: Option<&str>`, `APP_UPDATE_PUBKEY: Option<&str>` |
| Feedback | `FEEDBACK_ENDPOINT: Option<&str>`, `FEEDBACK_TOKEN: Option<&str>` |
| Network helpers | IP lookup, DNS-leak, ping and speed-test URLs, geoip/geosite rule-set URLs |

Rules:
- Unset (`None`) means the feature is off. No placeholder URLs are ever contacted.
- The frontend never duplicates these values. A new `app_info` command returns the name, version, core list and whether feedback and app updates are configured.
- `tauri.conf.json` (window title, bundle identifier) cannot import Rust and stays as is; a comment at the top of `app_config.rs` says so.

## 2. Multiple core types and their release endpoints

A core is described by data, declared once in `app_config.rs`:

```
CoreSpec {
  id: "sing-box",              // stable key used in settings and download ids
  name: "sing-box",
  releases_api: "https://api.github.com/repos/SagerNet/sing-box/releases?per_page=30",
  asset_name: fn(version, arch) -> String,   // per-core archive naming
  binary_name: "sing-box",
  version_args: ["version"],
  min_version: "1.12.0",
  required_features: ["with_clash_api", "with_utls"],
  system_names: ["sing-box"],  // looked up on PATH and in Homebrew prefixes
}
```

Adding a downloadable core is one `CoreSpec` entry. Running it needs more, so the design has two layers:

- **Download layer (config only):** release listing, checksum verification, extraction, version check, install into `core/<id>/<version>/`, update policy per core.
- **Runtime layer (`CoreAdapter` trait, code per core):** builds that core's config from the app's profiles and routing, gives launch arguments, and the local control API used for stats. Only the sing-box adapter exists today. A new core gets an adapter before it can be selected as the active engine.

Settings change from one managed version to one per core: `cores: { "<id>": { managedVersion, channel, updatePolicy, pendingVersion } }` plus `activeCore`. Existing single-core settings are migrated on load, so current users lose nothing.

The Engine page gets a core selector at the top; Versions, Channel and Update policy then apply to the selected core.

## 3. Download provider interface (feeds the title-bar menu)

```
trait DownloadProvider {
  id, kind (app | core), title
  check() -> available update?
  install(release, options)
  cancel()
}
```

Providers: one `CoreProvider` per entry in `CORES`, plus `AppUpdateProvider`. One registry owns the "one at a time" lock, the cancel flag and a single `download-progress` event. The title-bar menu groups items by kind ("App", then each core). The current engine installer moves behind this with no behavior change.

## 4. App updater

- Tauri's updater plugin, signature-verified against `APP_UPDATE_PUBKEY`.
- Checks `APP_UPDATE_FEED` on launch and daily. If it is `None`, nothing happens.
- Home banner and download-menu entry show the version and notes, with **Install and restart**.
- Never restarts while the VPN is connected without asking; offers "Install when I disconnect".
- Settings: App updates (Ask / Automatic / Off), current version, Check now.
- Needs its own signing key pair (not the unrelated key in `~/.tauri`).

## 5. Feedback via backend endpoint only

No mailto, no GitHub issue links. The app posts to the dashboard backend and nothing else.

**Form** (`FeedbackDialog`): type (Bug / Idea / Question), message, optional contact, an **Include diagnostics** checkbox (off by default) with a preview of the exact payload, and a single **Send** button. If `FEEDBACK_ENDPOINT` is `None` the entry points are hidden.

**Request contract** (`POST FEEDBACK_ENDPOINT`, `Content-Type: application/json`, optional `Authorization: Bearer FEEDBACK_TOKEN`):

```json
{
  "reportId": "uuid generated per submission",
  "kind": "bug | idea | question",
  "message": "...",
  "contact": "optional",
  "app": { "name": "...", "version": "..." },
  "diagnostics": {
    "macos": "...", "arch": "...",
    "core": { "id": "sing-box", "version": "..." },
    "mode": "tun | proxy", "state": "connected | ...",
    "logs": ["last 200 lines, redacted"]
  },
  "sentAt": 1700000000
}
```

`diagnostics` is `null` unless the user ticked the box. A `2xx` response means accepted; the optional body `{ "id": "..." }` is shown to the user as their reference number. Any other status or a network error keeps the draft open with the error and a Retry button, so nothing is lost.

**Redaction runs before the preview and before sending:** server hosts, UUIDs, Reality keys, pins and short IDs from the user's profiles are replaced, and a scan masks leftover UUIDs and IP addresses in the logs. A unit test with a fake profile asserts none of it leaks.

**Backend notes for the dashboard side:**
- A token embedded in a desktop app is not a secret. Treat it as an ingest key, and rate-limit and validate on the server.
- Reports carry no persistent install ID. If you want per-install grouping later, it would be an opt-in random ID generated locally.

**Rust:** `feedback.rs` with `feedback_preview` and `feedback_send`. Types `FeedbackKind` and `FeedbackInput` are mirrored in `api.ts`.

## 6. Build order

1. `app_config.rs`, moving the existing hardcoded values into it.
2. Core registry and settings migration, engine page core selector (sing-box only at first).
3. Download provider interface; engine moved onto it.
4. Feedback: Rust, types, dialog, redaction tests.
5. App updater, tested against a fake feed on localhost.
6. Browser-pane checks, one commit per step.

## 7. Open items

- Release host for app updates, and the updater signing key.
- Apple Developer account: updates work without it, first install shows a Gatekeeper warning.
- Which second core you plan to add first, so its `CoreSpec` and adapter can be scoped.
- Feedback backend URL and token, and whether the dashboard wants any fields beyond the contract above.
