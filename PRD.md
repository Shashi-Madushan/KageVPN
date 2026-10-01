# PRD: SingBox GUI — a minimal sing-box VPN manager for macOS

| | |
|---|---|
| Status | v1 MVP built (M0–M10). Next: [PRD-v2.md](PRD-v2.md) |
| Date | 2026-09-30 |
| Platform | macOS (Apple Silicon first, Intel later) |
| Stack | **Tauri 2** (Rust backend) + Svelte/Vite + plain CSS |

## 0. Status (2026-09-30)

### Done (MVP scaffold)
- Tauri 2 + SvelteKit project, Rust core split into `profile`, `config`, `runner`, `session`, `logs`, `net`, `store`, `singbox`.
- VLESS link import with parsing and validation. Profiles persist in the app data dir.
- Generated sing-box config for **TUN** and **Proxy-only** modes, validated against the real `sing-box check` in tests.
- Process supervision via a shell supervisor with a stop file, so disconnect needs no second admin prompt, and sing-box stops if the app dies. TUN uses one `osascript` admin prompt per connect.
- Live log tail with level filter, search and clear. Error hints: changed server key (with **Trust & reconnect**) and unresolved hostname.
- TCP latency test per profile, real-delay probe and exit-IP lookup through the tunnel.
- Guard against a busy port 2080 (another sing-box running).
- 14 Rust tests, clippy clean, `svelte-check` clean, debug `.app` bundle builds.

- **Verified live in Proxy mode:** connect, real exit IP through the tunnel, Trust & reconnect, disconnect with no leftover processes and the port freed.
- **Background mode (M6):** Dock icon hidden (`Accessory` policy applied after `build()`), closing the window hides it while the VPN stays up, tray icon with a native menu (Open, Connect/Disconnect, Launch at login, Quit), Quit disconnects first, `--hidden` start for launch at login. Verified live.
- Status events now drive the exit-IP and latency check, so it works for tray-initiated connects too.

- **Verified live in TUN mode:** one admin prompt on connect, sing-box runs as root, a new `utun` interface appears, system traffic exits through the server with no proxy configured, and disconnect needs no prompt and removes the interface and all processes.

- **Traffic pipeline (M7):** sing-box Clash API on a local port with a per-session secret, a streaming `/traffic` reader, a 60-sample ring buffer and totals, pushed to every window as a `traffic` event. Verified with a real download.
- **Tray (M8):** three template icons (idle outline, connecting dimmed, connected filled). While connected, the icon is redrawn with **two stacked rows to its right, `↑` upload over `↓` download**, using a baked DejaVu Sans Mono glyph atlas so digits never jitter. A native `set_title` was tried first and rejected because macOS clips a two-line title at the menu-bar height. Left click opens the popover, right click opens the native menu. A setting hides the speed.
- **Popover (M9):** frameless, vibrant window anchored under the icon, closes on blur. Shows status, duration, connect toggle, exit IP and latency, live download and upload, a 60-second graph, profile and mode switchers, Open window and Quit.
- **UI redesign (M10):** inset title bar with native vibrancy, tokens for light and dark, hero card with a glowing power ring, speed card with sparkline, profile cards with double-click rename, toasts instead of inline errors, empty state with clipboard paste, Settings tab (launch at login, connect on launch, speed in menu bar, quit).

### Known gaps
- Sparkline is right-aligned on a fixed 60-second window, so it looks sparse for the first seconds.
- If a tray-initiated connect fails, the main window opens to show the error; there is no OS notification yet.
- Debug build only; no signed or notarized release yet.

## 1. Summary

A small native menu-bar app that wraps the `sing-box` CLI. The user pastes a `vless://` link, clicks **Connect**, and gets a full-system TUN VPN. No config editing, no terminal. Includes live logs and a latency ("ping") test.

## 2. Problem

Today's flow: `brew install sing-box`, hand-convert a VLESS URL into JSON (including hex→base64 cert pin conversion), then run `sudo sing-box run ...` in a terminal. It is error-prone (stale hostnames, pin mismatches, TUN needing root) and gives no feedback.

## 3. Goals / Non-goals

**Goals**
- Import a `vless://` URL (paste, clipboard, or from a file) and auto-generate a valid sing-box config.
- One-click Connect / Disconnect, with TUN (system-wide) mode and a proxy-only mode.
- Live log viewer and per-profile ping (latency) test.
- Tiny footprint: installer < 15 MB, idle RAM < 60 MB, no Electron.
- Clear, honest error messages (DNS failure, pin mismatch, permission denied).

**Non-goals (v1)**
- Protocols other than VLESS (VMess/Trojan/SS/Hysteria come in v2).
- Routing-rule editor, subscriptions, per-app proxy, Windows/Linux.
- Bundling/updating sing-box itself (v1 detects the Homebrew binary; bundling is v2).

## 4. Users

A single technical user (the author) who wants a one-click replacement for the terminal workflow. Design for one person; no accounts, no telemetry.

## 5. Functional requirements

### 5.1 Profiles & URL import (P0)
- Parse `vless://UUID@host:port?encryption&security&sni&fp&alpn&pcs&type&headerType#name`.
- Supported params: `security=tls|reality|none`, `sni`, `fp` (uTLS), `alpn`, `type=tcp|ws|grpc`, `flow`, `pcs` (pinned cert), `#name` as profile label.
- Import via: paste field, "Import from clipboard" button, drag-drop text/file. Show a parsed preview (server, SNI, transport) before saving.
- Store profiles as JSON in `~/Library/Application Support/SingBoxGUI/profiles.json`. Support rename, delete, duplicate, and "copy share link".
- Validate every generated config with `sing-box check` before connecting.
- **Cert pin handling:** `pcs` in share links is a *certificate* SHA-256 (hex). sing-box pins the *public key* SHA-256 (base64). Do not convert blindly. On a mismatch, show the fingerprint the server presented and offer **"Trust this key"** (explicit, user-confirmed), instead of silently disabling verification.

### 5.2 Connect / Disconnect (P0)
- Big single toggle in the main window and in the menu-bar popover. States: Disconnected, Connecting, Connected, Error.
- Spawn sing-box as a child process with the generated config; stream stdout/stderr to the log pipeline.
- Show: connected duration, current exit IP (fetched through the tunnel), up/down speed (via sing-box Clash API `/traffic`).
- Clean disconnect: terminate process, confirm routes/interface are removed. Recover if the app crashes (detect and kill orphaned sing-box on launch).

### 5.3 TUN mode & privileges (P0)
- Mode switch: **TUN (system-wide)** / **Proxy only** (`127.0.0.1:2080` mixed inbound, optional "set as macOS system proxy").
- TUN needs root. Approach (pick in M2 spike): a one-time-installed privileged helper (`SMAppService` daemon) that launches sing-box as root, with the app talking to it over XPC/a Unix socket. Fallback for v0: `osascript ... with administrator privileges` prompt per connect.
- Generated TUN config: `auto_route`, `auto_detect_interface`, sniff + `hijack-dns`, private IPs direct, DNS over the tunnel.
- Kill switch toggle (off by default): block traffic if the tunnel drops.

### 5.4 Logs (P0)
- Live log panel with level filter (info/warn/error), text search, pause, clear, copy, "Export .log".
- Ring buffer of the last ~5,000 lines in memory; rotate a file log (max 5 MB x 3).
- Errors are pattern-matched into human hints, e.g. `NXDOMAIN` → "Server hostname doesn't resolve; ask the server owner to update it"; `unrecognized remote public key` → "Server certificate changed" plus the Trust action; `operation not permitted` → "TUN needs administrator permission".

### 5.5 Ping / latency test (P0)
- Per-profile **Test** button (and "Test all"). Two measurements:
  1. **TCP connect** time to `host:port` (works while disconnected).
  2. **Real delay** — HTTP GET `https://www.gstatic.com/generate_204` through a temporary sing-box instance (or the Clash API `/proxies/{name}/delay` when connected).
- Display ms with color bands (green < 300, amber < 800, red/timeout). Result timestamp shown.

### 5.6 Menu-bar tray and background mode (P0, next)
- **Runs in the background:** the app is a menu-bar app. The Dock icon is hidden (`ActivationPolicy::Accessory`). Closing the main window hides it instead of quitting. The VPN keeps running because sing-box is already supervised independently of the window. **Quit** exists only in the tray menu and disconnects first.
- **Tray icon states:** a template (monochrome) icon when disconnected, a distinct connected variant (filled with a green dot), and an amber one while connecting. Icons follow light/dark menu bars.
- **Speed next to the icon:** while connected, the tray title shows live rates, for example `↓ 1.2 MB/s ↑ 80 KB/s`, updated every second via `TrayIcon::set_title` (macOS shows it beside the icon). Compact formatting (`B`, `K`, `M`) and fixed width digits to avoid the menu bar jittering. A setting toggles speed text on or off.
- **Dropdown popover:** left click on the icon opens a small frameless window (~320 x 420) anchored under the icon. It closes on blur or Escape. Contents:
  - Status header: state dot, profile name, connected duration.
  - Big connect/disconnect toggle.
  - Live stats: download and upload speed, session totals, latency, exit IP, mode.
  - A 60-second speed sparkline.
  - Profile quick switcher (disabled while connected) and a TUN/Proxy switch.
  - Footer: **Open window**, **Logs**, **Quit**.
- **Right click menu:** a native menu with Connect/Disconnect, Open window, Quit, as the fallback path.
- **Launch at login** and **auto-connect last profile on launch** (off by default).

### 5.7 UI redesign (P0, next)
Goal: a calm, native-feeling, dense-but-clear interface. Replace the plain scaffold styling.
- **Window:** hidden title bar with inset traffic lights (`titleBarStyle: Overlay`), native macOS vibrancy (sidebar material) behind translucent surfaces, rounded 14 px cards, 8 px grid.
- **Home:**
  - Hero card with a state-coloured gradient ring around the power button (grey, amber pulse, green glow) and the profile name, duration and exit IP under it.
  - Stats row with four tiles: Download, Upload, Latency, Mode, each with a tiny sparkline for the speeds.
  - Profile cards with protocol badge, server, latency pill (colour banded) and overflow menu (rename, copy link, delete).
  - Empty state with an illustration and a single **Paste link** call to action that reads the clipboard.
- **Logs:** monospace view with level colour chips, sticky filter bar, "jump to latest" pill, copy and export.
- **Settings tab:** launch at login, auto-connect, show speed in menu bar, sing-box path, DNS server, log level.
- **System quality:** toasts instead of inline error text, skeleton/loader states, focus rings, full keyboard support, `prefers-reduced-motion`, and dark/light via CSS tokens.
- **Component plan:** `StatusHero`, `StatTile`, `Sparkline`, `ProfileCard`, `Toast`, `SegmentedControl`, shared `tokens.css`. The popover reuses the same components from a second route (`/tray`).

### 5.8 Settings (P1, folded into 5.7)
- Settings: launch at login, auto-connect last profile, mode default, sing-box binary path (auto-detect `/opt/homebrew/bin/sing-box`), log level, DNS server.
- Dark/light follows system.

### 5.9 sing-box management (P1)
- Detect missing binary → offer `brew install sing-box` with progress, or show the manual command.
- Show the installed version; warn if below the minimum supported (1.12+ for the new DNS format).

## 6. UX / UI

> The wireframe below is the v0.1 layout. Section 5.7 supersedes its visual design; the tray popover (5.6) is specified separately.

Design principles: one primary action, nothing hidden behind modals, status visible at a glance, calm and quiet.

**Window** — 420 x 640 fixed-ish, single column, 3 tabs:

```
┌──────────────────────────────────────┐
│  ● Connected · 00:12:41      ⚙      │  header: status + settings
│                                      │
│           ┌──────────────┐           │
│           │   ⏻  (big)   │           │  Connect / Disconnect
│           └──────────────┘           │
│      me-me · 203.0.113.10           │  profile + exit IP
│      ↓ 1.2 MB/s   ↑ 80 KB/s          │
│      Mode:  [ TUN | Proxy ]          │
│                                      │
│ ─ Profiles ────────────────  + Add ─ │
│ ● me-me      vless·tls   142 ms  ⋯   │  row: select, latency, menu
│ ○ work       vless·reality  —    ⋯   │
│                                      │
│ [ Profiles ]  [ Logs ]  [ Settings ] │  tab bar
└──────────────────────────────────────┘
```

- **Add profile sheet:** one textarea ("Paste vless:// link"), live-parsed preview card, **Save**. Errors inline ("Missing UUID").
- **Logs tab:** monospace, auto-scroll with "jump to latest", level chips, search box.
- **Visuals:** system font (SF), 8 px grid, single accent colour (green = connected, amber = connecting, red = error), 12 px radius cards, subtle vibrancy background. Respect `prefers-color-scheme` and reduced motion.
- **Accessibility:** full keyboard navigation, `⌘K` focus import, `⌘L` logs, `⌘⏎` connect/disconnect; VoiceOver labels on the toggle and state.

## 7. Technical design

```
 Svelte UI ──invoke/events──> Tauri (Rust core)
                                ├─ profile store (serde_json)
                                ├─ vless parser + config generator
                                ├─ process manager (spawn/kill sing-box)
                                ├─ privileged helper client (TUN)
                                ├─ log pipeline (ring buffer → events)
                                └─ stats/ping (Clash API, reqwest)
                                        │
                                 sing-box (child, root for TUN)
```

- **Why Tauri:** uses the system WebKit, so a ~8–12 MB app, low RAM, Rust for safe process and privilege handling, native tray support. Electron would be 100+ MB.
- **Config generation:** Rust builds the JSON from typed structs (templates for TUN and proxy modes). Enable the `experimental.clash_api` on `127.0.0.1:9090` (random secret per session) for traffic and delay stats.
- **Secrets:** profiles contain UUIDs; store in the app data dir with `0600` perms (Keychain in v2). Never log full links.
- **Distribution:** signed and notarized `.dmg`; universal binary (arm64 + x86_64) in v1.1.


### 7.1 Technical plan for tray, background and speed
- **Tray and popover:** Tauri `tray-icon` feature, `tauri-plugin-positioner` (`Position::TrayBottomCenter`) and a second window `tray` (`decorations: false`, `transparent`, `alwaysOnTop`, `visible: false`, `skipTaskbar`). Left click toggles it, blur hides it. Rust module `tray.rs` owns the icon, menu, title updates and popover placement. Custom popover chosen over a native menu because a native menu cannot show a sparkline, live stats or a toggle layout.
- **Background:** `app.set_activation_policy(Accessory)`, handle `CloseRequested` with `prevent_close` + `hide`, and quit only through the tray. Add `tauri-plugin-autostart` for launch at login.
- **Speed source:** enable sing-box `experimental.clash_api` on `127.0.0.1:9090` with a random per-session secret. A Rust task streams `GET /traffic` (one JSON line per second with `up` and `down`), keeps a 60-sample ring buffer, emits a `traffic` event to both windows and updates `tray.set_title`. The task starts on connect and stops on disconnect. Session totals come from summing samples. Verify the Homebrew sing-box build includes the Clash API.
- **Shared state:** both windows subscribe to the same `status`, `traffic` and `logs` events. The main store in `app.svelte.ts` is reused by the popover route, so there is one source of truth.
- **Icons:** ship three template-ready PNG/PDF icon sets (idle, connecting, connected) sized for 1x/2x menu bars.
- **Risks:** `set_title` is macOS-only (fine, the app is macOS-only). The popover must not steal focus from other apps or close when a native dialog (the admin prompt) appears. Anchor position on multi-display setups and notched Macs needs a manual check.

## 8. Milestones

| # | Deliverable | Status |
|---|---|---|
| M0 | Tauri scaffold, theming | Done |
| M1 | VLESS parser, config generator, `sing-box check` tests | Done |
| M2 | Process manager, proxy-only connect/disconnect, log stream | Done |
| M3 | TUN with `osascript` privilege flow, orphan protection | Done |
| M4 | Latency test, exit IP, error hints, Trust-key flow | Done |
| M5 | Live end-to-end verification in Proxy and TUN mode | Done |
| M6 | Background mode: accessory policy, hide on close, quit from tray, autostart, minimal tray menu | Done |
| M7 | Traffic pipeline: Clash API, `/traffic` stream, ring buffer, events | Done |
| M8 | Tray icon states, speed text in the menu bar, right-click menu | Done |
| M9 | Tray popover window with stats, sparkline, quick controls | Done |
| M10 | UI redesign: tokens, vibrancy, hero, stat tiles, profile cards, toasts, settings tab | Done |
| M11 | Packaging: signed and notarized `.dmg`, privileged helper spike | Later |


## 9. Acceptance criteria (v1)

- Paste the sample link → profile appears in < 1 s with a correct preview.
- Connect in TUN mode shows the server's IP at `ifconfig.me`; Disconnect restores the original IP and leaves no `utun` routes behind.
- Killing the app (or force-quit) never leaves sing-box running invisibly.
- A bad hostname, stale pin, and denied admin prompt each produce a specific, actionable message.
- Ping test returns a value (or timeout) for each profile in under 10 s.
- With the window closed, the tray icon stays, the VPN stays connected, and the Dock icon is hidden.
- Tray shows live up/down speed within 1.5 s of traffic changing, with no layout jitter.
- Clicking the tray icon opens the popover under the icon in under 200 ms; it closes on blur.
- Release build < 15 MB; idle RAM < 60 MB; cold start < 1 s.

## 10. Risks & open questions

- **Privileged helper complexity** (signing, `SMAppService`, notarization) is the biggest risk. Mitigation: ship the `osascript` prompt first, then upgrade.
- **Share-link `pcs` semantics** differ between Xray and sing-box (see 5.1). Needs the Trust-key UX, or a cert-hash check through a custom verifier.
- **sing-box config churn** between versions (DNS/route formats changed in 1.12–1.14). Pin a minimum version and run `check` on every generation.
- Open: bundle sing-box inside the app (bigger, self-contained) vs. depend on Homebrew (smaller, simpler)? Recommendation: depend on Homebrew for v1.
- Open: keep the app macOS-only, or keep the Rust core portable for later Windows/Linux? Tauri makes the latter cheap.

## 11. Future (v2+)

Subscription URLs, VMess/Trojan/Hysteria2/Reality UI, QR import/export, per-app/domain rules, Keychain storage, auto-update of sing-box, Windows/Linux builds.
