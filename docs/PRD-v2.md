# PRD v2: Kage VPN — core management, data usage and a richer VLESS client

| | |
|---|---|
| Status | Implemented (compiled, not yet tested live), see Section 0 |
| Date | 2026-10-01 |
| Builds on | [PRD.md](PRD.md) (v1 MVP, milestones M0–M10 done) |
| Scope rule | **VLESS only.** Other protocols stay out of v2. |
| Platform | macOS, Apple Silicon first |

## 0. Implementation status (2026-10-01)

Built in this pass. The Rust side passes `cargo clippy`, the Svelte side passes `svelte-check`, and a debug app builds. **Only core management (V2) and the V1 foundations were run live.** Everything marked "not tested" below was written and compiled but not exercised, by request.

| Milestone | State |
|---|---|
| V0 carry-over | Icons and plan committed. Release `.app` builds (4.6 MB). `.dmg` via the stock bundler fails in this environment, build it with `hdiutil` |
| V1 foundations | Done: schema versioning with backup, notifications, sidebar layout with a fixed sidebar and scrolling content, SQLite, `LSUIElement` so the Dock icon never appears. Verified live |
| V2 core management | Done and **verified live**: releases API, SHA-256 verified install, switch, roll back, remove, update notification |
| V3 usage collector | Built, not tested: per-minute store, hourly roll-up, per-app and per-domain estimates, quota alerts at 80 and 100 percent, optional disconnect |
| V4 usage UI | Built, not tested: range presets and custom dates, group by hour/day/week/month, profile, mode, app and domain filters, previous-period comparison, interactive chart with drill-down, top tables, CSV export, cap and cycle settings |
| V5 VLESS in full | Built, not tested: edit form, Reality, WebSocket, gRPC, HTTPUpgrade, groups, favorites, search, duplicate, bulk import, share link and QR code |
| V6 subscriptions | Built, not tested: plain and base64 lists, scheduled refresh, quota and expiry from `subscription-userinfo`, clipboard watcher (opt-in), `kagevpn://` deep links |
| V7 smart connect | Built, not tested: test all (real delay when connected, TCP otherwise), latency history sparklines, live profile switching without reconnect, auto-fastest mode with failover |
| V8 routing | Built, not tested: global, rule and direct modes, rule editor with reorder, country lists, DNS provider choice, rule tester |
| V9 connections and diagnostics | Built, not tested: live connection table with close, leak check, speed test, support report |
| V10 reliability | Built, not tested: auto-reconnect with backoff, watchdog, kill switch (experimental, uses the macOS firewall), global hotkey |
| V11 helper, signing | **Not done.** Needs an Apple Developer account and cannot be verified here |

### Deliberately deferred
- **Keychain storage** of UUIDs: unsigned development builds trigger a prompt on every access. Do it together with signing.
- **Privileged helper, signing, notarization, app auto-update** (V11).
- **QR import from an image**: needs image decoding crates that add size.
- **Trusted Wi-Fi rules**: macOS no longer exposes the Wi-Fi name without a Location permission, so there is no reliable way to read it.
- **Onboarding wizard** and **FakeIP**.

### Known risks in untested code
- The **kill switch** edits firewall rules as root. It is off by default and has a "Restore internet access" button, but it has never been run.
- Rule lists make sing-box download from GitHub at connect time; if GitHub is unreachable a connect with lists enabled may fail.
- Per-app usage depends on sing-box resolving the process name and is labelled an estimate.

## 1. What v2 is about

v1 proved the loop: import a link, connect in TUN or proxy mode, watch speed in the menu bar. v2 turns it into a tool you can live in:

1. **Manage sing-box itself**: see which core runs, update it safely, pin it, roll back.
2. **Know your data usage**: persistent, per-profile history with date-range filtering, breakdowns and quotas.
3. **A richer VLESS client**: full VLESS parameters (Reality, WebSocket, gRPC), groups, subscriptions, auto-pick fastest, split tunneling, a connection inspector, diagnostics, kill switch.
4. **Ship it properly**: privileged helper (no more password prompt for TUN), signed and notarized release.

## 2. Carry-over from v1

| Item | State |
|---|---|
| Release `.app` / `.dmg` build | Attempted, the process was killed (exit 137) before bundling; retry with more free memory or `CARGO_BUILD_JOBS=2` |
| New app icon (`design/app-icon.png`, `src-tauri/icons/*`) | Done, not yet committed |
| Settings toggles and TUN after the redesign | Toggles verified; **connect on launch** and **TUN** need port 2080 free (a terminal `sudo sing-box` was holding it) |
| Silent failure of background connects | v2 adds notifications (Section 5.9) |

### Security finding to fix in v2
In TUN mode the app starts sing-box **as root**, but the binary (`/opt/homebrew/bin/sing-box`, user-writable) and the config (`~/Library/Application Support/...`, user-writable) are both writable by any process running as the user. Malware running as the user could swap either and get root the next time you connect. v2 fixes this with the helper in Section 6.4: root-owned copies of the core and config, checked by hash before launch.

## 3. Goals and non-goals

**Goals**
- Core: detect, install, update, pin, roll back sing-box without Homebrew; verify every download.
- Usage: accurate, persistent, filterable by date range, profile, mode, app and domain; quotas with alerts.
- Profiles: complete VLESS coverage (TCP, WS, gRPC, HTTPUpgrade; TLS and Reality; Vision flow), groups, subscriptions, QR.
- Smart connect: pick the fastest profile, fail over automatically.
- Routing: global, rule and direct modes; per-app and per-domain split tunneling.
- Reliability: kill switch, auto-reconnect after sleep or network change, trusted Wi-Fi rules.
- Same footprint philosophy: installer under 20 MB, idle RAM under 80 MB.

**Non-goals**
- VMess, Trojan, Shadowsocks, Hysteria, TUIC (still out).
- Windows and Linux.
- Hosting or selling VPN service; no accounts, no telemetry, everything local.

## 4. Product areas

### 4.1 Core management (P0)

**Today:** the app finds the first `sing-box` on a fixed path list. No version control.

**v2 behavior**
- **Core page** (Settings → Core) shows: active core version, source (System/Homebrew or Managed), path, architecture, build tags, and a health row: requires `with_clash_api` and `with_utls`, minimum version 1.12.
- **Sources:** *System* (use what is installed) or *Managed* (app-downloaded, kept in `~/Library/Application Support/SingBoxGUI/core/<version>/`). Default to Managed once one is installed.
- **Update check:** query `https://api.github.com/repos/SagerNet/sing-box/releases`. Channel selector: **Stable** (default) or **Pre-release** (the repo publishes `1.15.0-alpha.*` builds). Check on launch and once a day; manual **Check now**. Show version, date, size and release notes.
- **Download and verify:** pick the asset `sing-box-<version>-darwin-arm64.tar.gz` (or `amd64`; skip the `legacy-macos-10.13` build). The releases API exposes a **SHA-256 digest per asset**; verify it before extracting. If the digest is missing, refuse unless the user explicitly confirms.
- **Install safely:** extract to a temp dir, run `sing-box version` (must report required tags), run `sing-box check` against the current generated config, then move into `core/<version>/`. Nothing becomes active until every step passes.
- **Switch and roll back:** keep the last 2 versions. **Use this version** and **Roll back** are one click. If connected, ask to reconnect (the new core only applies on restart).
- **Pin:** "Don't auto-switch" per version; updates are never applied silently, only offered. An optional "auto-install stable, apply on next disconnect" setting.
- **Homebrew users:** show `brew upgrade sing-box` as a copyable command and a **Run** button for the same.
- **Failure UX:** clear messages for no network, rate limit (GitHub 403), digest mismatch, extraction error, config rejected by the new core (with the reason and a keep-current option).
- **Disk:** about 27 MB per version; show total and offer **Remove**.

### 4.2 Data usage tracking (P0)

**Goal:** a trustworthy record of how much data went through the VPN, with the date filtering people expect from a phone data-usage screen.

**What is recorded (all local, SQLite)**
- Upload and download bytes, per **minute**, per **profile**, per **mode**. Authoritative source: the sing-box Clash API `/traffic` stream already used for the tray (one sample per second, summed into minute buckets and flushed every 10 s; a crash loses at most 10 s).
- Optional breakdowns, labelled **estimated** because they come from connection snapshots (`/connections`, polled every 2–5 s, delta per connection):
  - **By app**: needs `route.find_process`, which gives the process name on macOS.
  - **By domain**: the destination host from sniffing.
  - Both are **off by default for domains, on for apps**, each with its own switch and retention. Domains can be sensitive, so never recorded unless enabled.
- Roll-ups: minute buckets are kept 30 days, then rolled into **hourly** (kept 12 months) and **daily** (kept until deleted). Per-app and per-domain rows kept 30 to 90 days (setting).

**Usage screen**
- **Range picker:** Today, Yesterday, Last 7 days, Last 30 days, This month, Last month, This billing cycle, **Custom range** (start and end date). Previous period comparison toggle (for example, this week vs last week, with +/- %).
- **Group by:** Hour, Day, Week, Month (auto-chosen from the range, overridable). Buckets use the local time zone and handle DST; storage is UTC epoch seconds.
- **Filters:** profile, mode (TUN/Proxy), app, domain (search). Filters combine and persist for the session.
- **Summary cards:** Total, Download, Upload, Daily average, Peak day, Connected time, vs previous period.
- **Chart:** stacked bars (download and upload) per bucket with hover tooltip; click a bar to drill into that day or hour. Toggle between bytes and cumulative.
- **Tables:** Top apps, Top domains, By profile; sortable, each row with share-of-total bar. Click a row to apply it as a filter.
- **Export:** CSV and JSON of the current range and filters.
- **Quota:** monthly data cap with a **billing cycle start day** (1–31) and optional per-profile caps. Progress bar, alerts at 80% and 100% (notification), optional **disconnect at 100%**. Over-quota state shown on Home and in the popover.
- **Popover:** a compact "Today" and "This month" line under the speed graph.
- **Controls:** pause recording, **Reset usage** (with confirmation), choose retention, **Clear app/domain history**.

**Accuracy rules**
- Totals count the tunnel's own traffic including DNS, so they match the server's view closely, not the sum of app bytes. The UI labels breakdowns as estimates.
- Core restarts reset sing-box counters; the collector stores per-sample deltas so restarts never double count or go negative.
- Sleep and wake: gaps simply have no samples; no interpolation.

**Schema (SQLite, `usage.db`)**
```sql
usage_minute(ts INTEGER, profile_id TEXT, mode TEXT, up INTEGER, down INTEGER,
             PRIMARY KEY (ts, profile_id, mode));
usage_hour  (ts INTEGER, profile_id TEXT, mode TEXT, up INTEGER, down INTEGER,
             PRIMARY KEY (ts, profile_id, mode));
usage_dim   (ts_hour INTEGER, profile_id TEXT, kind TEXT, key TEXT, up INTEGER, down INTEGER,
             PRIMARY KEY (ts_hour, profile_id, kind, key));
sessions    (id INTEGER PRIMARY KEY, profile_id TEXT, mode TEXT, started INTEGER, ended INTEGER);
```
Commands: `usage_summary`, `usage_series`, `usage_top`, `usage_export`, `usage_set_quota`, `usage_reset`.

### 4.3 Profiles, VLESS in full (P0)
- **Edit form** for every VLESS parameter: server, port, UUID, flow (`xtls-rprx-vision`), TLS or **Reality** (public key, short id, SNI, fingerprint), transport (**TCP, WebSocket, gRPC, HTTPUpgrade**) with path, host, service name, ALPN, certificate pin handling (existing Trust flow), packet encoding.
- **Import:** paste, drag-drop text or file, **clipboard watcher** ("A vless:// link was copied. Import?"), **bulk import** (many links at once), QR code scan from an image, `kagevpn://import?url=...` deep link.
- **Export:** copy share link, show **QR code**.
- **Organize:** groups/folders, tags, favorites, drag to reorder, duplicate, rename, sort by name or latency.
- **Subscriptions (VLESS lists):** add a URL that returns links (plain or base64); refresh on a schedule or manually; show server-reported quota and expiry from the `subscription-userinfo` header; keep removed nodes for one cycle before deleting.
- **Credentials in Keychain:** UUIDs and pins move from `data.json` into the macOS Keychain.

### 4.4 Smart connect (P1)
- **Latency test all** (real delay through each profile, not just TCP), results kept as a small history per profile with a sparkline.
- **Auto profile "Fastest":** a virtual profile backed by sing-box `urltest`; re-tests every few minutes and switches without dropping when a clearly better node appears.
- **Failover:** a `fallback` group; if the active node fails the health check, move to the next and notify.
- **Retry policy** on connect failure with backoff.

### 4.5 Routing and split tunneling (P1)
- **Modes:** Global (everything through VPN), Rule (use rules), Direct (VPN on, nothing routed; for testing).
- **Rules by:** domain, domain suffix, keyword, CIDR, **app (process name)**, country (via sing-box rule-sets that the core downloads and caches).
- **Presets:** bypass LAN and private ranges (default), bypass a country, "only these apps use the VPN", "all except these apps".
- **Rule editor:** ordered list, drag to reorder, enable/disable, test a domain ("which rule matches example.com?").
- **DNS:** DoH/DoT/system choice, optional FakeIP, and a built-in **DNS leak test**.

### 4.6 Connections and diagnostics (P1)
- **Live connections view:** host, app, chain, rate, age; close one or all; filter and sort. Data from the Clash API `/connections`.
- **Diagnostics page:** exit IP and country, **IP leak** and **DNS leak** checks, an in-tunnel **speed test** (download and upload), latency history, **Copy diagnostics** (versions, config check result, recent errors, redacted).

### 4.7 Reliability (P1)
- **Kill switch:** block non-tunnel traffic if the tunnel drops (strict route), off by default.
- **Auto-reconnect** after sleep/wake and network change, with a visible state.
- **Trusted networks:** by Wi-Fi name, auto-disconnect on trusted networks and auto-connect elsewhere.
- **Connect on launch / login** (already built) stays.

### 4.8 UI structure for v2
The 420 px column gets crowded, so the main window becomes resizable with a **sidebar** (about 760 x 560 default):

| Sidebar | Content |
|---|---|
| Home | Hero connect card, speed, quick profile switch |
| Profiles | List, groups, edit form, subscriptions, import/export |
| Usage | Everything in 4.2 |
| Routing | Mode, rules, DNS |
| Connections | Live list (4.6) |
| Logs | As v1 plus export |
| Settings | General, Core (4.1), Notifications, Network, Data |

The tray popover stays compact and gains a Today/This-month usage line and the quota bar.

### 4.9 Notifications and shortcuts (P1)
- macOS notifications for: connect failed (including background and auto-connect), tunnel dropped, quota 80% and 100%, core update available, subscription expired.
- Global hotkey to toggle the connection; URL scheme `kagevpn://connect`, `disconnect`, `import`.
- Onboarding on first run: check core, offer install, import a link, test.

## 5. Technical design

### 5.1 Module layout (Rust, `src-tauri/src`)
```
core/        locator.rs  releases.rs  installer.rs  verify.rs  version.rs
usage/       collector.rs  store.rs (SQLite)  query.rs  quota.rs  rollup.rs
profile/     vless.rs  model.rs  subscription.rs  keychain.rs  qr.rs
routing/     rules.rs  rulesets.rs  dns.rs
config/      builder.rs (modes, groups, routing, reality/ws/grpc)
connections/ clash.rs (traffic, connections, proxies, delay)
helper/      client.rs  protocol.rs  (privileged daemon in its own crate)
notify.rs  schema.rs (versioned migrations)
```
Existing `session`, `runner`, `tray`, `traffic`, `logs` stay and are refactored to use these.

### 5.2 Data and migration
- Add `schemaVersion` to `data.json`; migrate v1 to v2 on first launch (profiles keep their ids; settings gain defaults). Back up the old file as `data.v1.json`.
- Usage data goes to SQLite (`rusqlite` with the `bundled` feature) in the same data dir. WAL mode, one writer task, batched flushes.
- Profiles and settings stay JSON for readability; secrets move to the Keychain.

### 5.3 Core installation details
- HTTP via the existing `reqwest`. Streamed download with progress events to the UI; SHA-256 via the `sha2` crate; extraction via `flate2` + `tar`.
- Architecture from `std::env::consts::ARCH`; asset name matched exactly, never the legacy build.
- Active core path stored in settings; `locate()` resolves Managed first, then System.

### 5.4 Usage collector
- One task per connection session. Reads `/traffic` (existing) and polls `/connections`. Aggregates in memory, flushes to SQLite every 10 s and on disconnect or quit.
- Per-app attribution requires `route.find_process: true` in the generated config.
- Quota evaluation runs on every flush; crossing a threshold fires a notification once per cycle.

### 5.5 Privileged helper (TUN without prompts)
- A small signed launch daemon registered with `SMAppService`. The app talks to it over XPC (or a root-owned Unix socket with code-signature checks).
- The helper installs the chosen core and config **into a root-owned directory**, verifies the stored SHA-256 on every start, then runs sing-box. This closes the finding in Section 2.
- One-time approval in System Settings; fall back to the current `osascript` flow if the user declines.

### 5.6 Frontend
- New routes under a sidebar layout. State stays in Svelte 5 runes stores split per area (`usage`, `core`, `profiles`, `routing`).
- Charts: a small hand-rolled SVG stacked-bar component (no heavy chart library) to protect the footprint.
- Date handling with `Intl` and a tiny local-time bucket helper; tested around DST.

## 6. Milestones

| # | Deliverable | Est. |
|---|---|---|
| V0 | Finish v1 carry-over: release `.dmg`, TUN and connect-on-launch live checks, commit icons | 0.5 d |
| V1 | Foundations: schema versioning and migration, SQLite, notifications, sidebar layout shell | 1.5 d |
| V2 | **Core management**: releases API, download and verify, install, switch, roll back, Core page | 2.5 d |
| V3 | **Usage tracking**: collector, SQLite store, roll-ups, quota, notifications | 2.5 d |
| V4 | **Usage UI**: range picker, filters, chart, tables, export, popover line | 3 d |
| V5 | VLESS in full: edit form, Reality/WS/gRPC, groups, bulk import, QR, Keychain | 3 d |
| V6 | Subscriptions and clipboard watcher, deep links | 1.5 d |
| V7 | Smart connect: test all, history, Fastest, failover | 2 d |
| V8 | Routing: modes, rule editor, rule-sets, split tunneling, DNS, leak test | 3 d |
| V9 | Connections view and diagnostics | 2 d |
| V10 | Reliability: kill switch, auto-reconnect, trusted Wi-Fi | 1.5 d |
| V11 | Privileged helper, signing, notarization, app auto-update | 3–4 d |

**Suggested order:** V0, V1, V2, V3, V4 first (core management and usage are the headline asks), then V5/V6, then V7 to V10, and V11 last since it needs an Apple Developer account.

## 7. Acceptance criteria

**Core**
- With no Homebrew install, the app installs a verified core in under 60 s on a normal connection and connects with it.
- A tampered download (wrong digest) is rejected and nothing changes.
- Switching to a newer core and rolling back both work with one click; a core that fails `sing-box check` is never activated.

**Usage**
- After a known test transfer (for example 100 MB), the Usage total for that range is within 3% of the transferred size.
- Closing and relaunching the app keeps all history; a crash loses at most 10 s.
- Changing the range, group-by, or any filter updates the chart and tables in under 300 ms for a year of data.
- Custom range, Last month and DST-boundary days show correct bucket boundaries in local time.
- A 5 GB monthly cap raises one notification at 80% and one at 100%, and disconnects at 100% if that option is on.
- Export CSV matches the on-screen numbers for the same range and filters.

**Profiles and routing**
- Import of Reality, WebSocket and gRPC VLESS links round-trips: import, edit, export gives an equivalent link.
- "Fastest" switches to a better node within one test interval without dropping existing connections for more than a second.
- Split tunneling by app sends only the listed apps through the tunnel (verified by exit IP per app).

**Reliability and security**
- Kill switch blocks traffic within 1 s of a tunnel drop.
- With the helper installed, TUN connects with no password prompt; the helper refuses a core or config whose hash does not match.

## 8. Risks and open questions

- **Per-app attribution depends on process lookup** (`find_process`) and can miss short-lived processes; keep it labelled "estimated".
- **Usage vs server-side accounting** will differ (headers, retransmits, DNS). Document it in the UI.
- **GitHub API rate limits** (60 requests per hour unauthenticated): cache results, use ETags, check at most once a day.
- **Pre-release cores** can break the config format. Pre-release stays opt-in, and the `sing-box check` gate stays mandatory.
- **Helper and notarization** require a paid Apple Developer account and add real engineering time; the `osascript` fallback must keep working.
- **sing-box config churn** across versions: pin a tested range per app version and show a compatibility warning outside it.
- Open: should usage also read the server-reported quota from subscriptions and show both side by side? Recommendation: yes, as a separate "Provider quota" row.
- Open: bundle a default core inside the app (adds about 27 MB, works offline) or download on first run? Recommendation: download on first run, keep the installer small.
- Open: keep per-domain recording at all, given the privacy cost? Recommendation: ship it off by default with a clear explanation.

## 9. Out of scope for v2

Other protocols, Windows/Linux builds, a CLI companion, iCloud sync of profiles, team or multi-user features.
