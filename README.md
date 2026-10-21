<div align="center">

# Kage VPN

**A clean, native-feeling VPN manager for macOS, powered by [sing-box](https://sing-box.sagernet.org).**

Import your VLESS, VMess, Trojan, Shadowsocks or Hysteria2 servers, connect with one click, and see exactly where your traffic goes.

![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![Built with](https://img.shields.io/badge/built%20with-Tauri%202%20%2B%20Svelte-blue)
![License](https://img.shields.io/badge/license-MIT-green)
[![Latest release](https://img.shields.io/github/v/release/Shashi-Madushan/KageVPN)](https://github.com/Shashi-Madushan/KageVPN/releases/latest)
[![Docs](https://img.shields.io/badge/docs-online-blue)](https://shashi-madushan.github.io/KageVPN/)

<img src="docs/images/home.png" alt="Kage VPN home screen, connected" width="860">

</div>

---

## What it is

Kage VPN is a desktop app that sits in your menu bar and manages a [sing-box](https://sing-box.sagernet.org) tunnel for you. You bring a server (a `vless://`, `vmess://`, `trojan://`, `ss://` or `hysteria2://` link, or a subscription URL from your provider); the app turns it into a working connection, keeps it healthy, and shows you what it is doing.

It is for people who already have a VLESS, VMess, Trojan, Shadowsocks or Hysteria2 server or provider and want a polished Mac app instead of hand-editing config files. It does not sell or provide VPN servers.

> Development happens on `dev` and is merged into `main` when it is ready; see [docs/BRANCHING.md](docs/BRANCHING.md).

## Install

> **Requirements:** a Mac (Apple Silicon or Intel) and a server link or subscription. The app downloads the sing-box engine for you on first run.

1. **Download** the latest `.dmg` from the [Releases page](https://github.com/Shashi-Madushan/KageVPN/releases).
2. **Open** the disk image and drag **Kage VPN** into your **Applications** folder.
3. **Open the app.** The first time, macOS will probably stop it. That is expected, and the next section explains how to get past it.

### "Kage VPN cannot be opened" (unsigned app)

macOS Gatekeeper blocks apps that are not signed and notarized with a paid Apple Developer ID. Kage VPN is not notarized yet, so macOS shows a warning such as *"Apple could not verify Kage VPN is free of malware"* or *"Kage VPN is damaged and can't be opened"*. The file is not damaged: macOS adds a quarantine flag to everything downloaded from the internet, and that flag is what triggers the warning.

Only do this for a build you downloaded from this repository's Releases page.

**Option A: Open Anyway in System Settings** (works on every recent macOS, required on macOS 15 Sequoia and later)

1. Open the app once and click **Done** (or **Cancel**) on the warning.
2. Open **System Settings → Privacy & Security**.
3. Scroll to **Security**. You will see *"Kage VPN was blocked to protect your Mac."*
4. Click **Open Anyway**, enter your password, then click **Open**.

**Option B: Right-click → Open** (macOS 14 and earlier)

1. In **Applications**, hold **Control** and click **Kage VPN** (or right-click it).
2. Choose **Open**, then click **Open** again in the dialog.

**Option C: Remove the quarantine flag in Terminal**

This is the fix for the "is damaged and can't be opened" message.

```bash
xattr -dr com.apple.quarantine "/Applications/Kage VPN.app"
```

Then open the app normally. You only need to do this once per downloaded version. Updates that the app downloads and installs itself are checked against its own signing key and normally do not show the warning again.

### First run

1. A short **welcome tour** explains the basics. You can replay it from **Settings → Help**.
2. **Download the engine.** If sing-box is not installed, the app offers to download the official release, checks its published checksum, and keeps it in its own folder. If you already installed sing-box with Homebrew, the app uses that and skips the prompt.
3. **Add a server.** Paste a server link (or several, mixed protocols are fine), drop a text file, or add a subscription URL.
4. **Connect.** Click the Kage mascot. In **Full System (TUN)** mode macOS asks for your administrator password once so the app can create the tunnel.

### Uninstall

1. Quit the app from its menu-bar icon or **Settings → Quit**.
2. Drag **Kage VPN** from Applications to the Trash.
3. Optional, remove saved data (servers, settings, usage history, downloaded engines):
   ```bash
   rm -rf "$HOME/Library/Application Support/com.apzur.kage"
   ```
4. If you had the kill switch on and your internet stays blocked after an unexpected quit, open the app and use **Settings → Restore internet access**, or run:
   ```bash
   sudo pfctl -a "com.apple/250.KageVPN" -F all
   ```

## Features

### Connect

- **One-click connection** with a clear status: protected, connecting or not protected, plus session time, exit IP and latency.
- **Two modes.** *Full System (TUN)* sends all of your Mac's traffic through the VPN. *Proxy Only* runs a local proxy on `127.0.0.1:2080` (the next free port if that one is taken; the Home page shows the address) for just the apps you point at it.
- **Five protocols.** VLESS (TLS or Reality), VMess, Trojan, Shadowsocks (including the 2022 ciphers) and Hysteria2 (with Salamander obfuscation). VLESS, VMess and Trojan run over TCP, WebSocket, gRPC or HTTPUpgrade.
- **Menu-bar app** with live download and upload speed beside the icon, and a compact popover led by the Kage mascot, which is also the connect button and shows at a glance whether you are protected. The popover has a latency pill, a server picker that switches servers without disconnecting, the timer and exit IP, live traffic, what is left of your provider's plan, and a mode switch, and shows loading skeletons while data loads.
- **Hide addresses.** Server addresses and IP addresses are blurred until you click the eye icon (next to the IP address, in the app and in the menu-bar popover), so screen shares and screenshots do not give them away. The choice is remembered.
- **Optional global shortcut** (⌥⌘V, off by default) to connect or disconnect from anywhere.
- **Launch at login** and **connect on launch**.

### Servers

- **Import** from pasted links, text files, **Clash or sing-box configuration files**, or **subscription URLs** (share-link, Clash and sing-box lists), with scheduled refresh and the data and expiry information your provider reports. You are told once when 90% of the plan's data is used and when it runs out, and 7 days, 3 days and 1 day before the plan expires.
- **Subscription options** for providers that are picky: choose which client the app asks as (v2rayN, sing-box, Clash Meta or your own), and send a device ID for providers that limit how many devices may use a subscription. Errors such as a reached device limit or an expired link say what to do.
- **Search, group, favorite and sort** servers. Duplicates are detected and skipped.
- **Latency tests** for every server, with a small history graph for each, and one button to remove every server the last test could not reach.
- **Fastest server automatically**, with failover when one stops working.
- **Share** a server as a link or QR code, and edit every field by hand when you need to.

### Protection and reliability

- **Verified connections.** The app says "You're protected" only after a real request has gone through the tunnel, keeps checking in the background, and tells you why when traffic stops (server not answering, server rejected the connection, or no traffic). Failed connects explain what went wrong, with a fix button.
- **Auto-reconnect** when the tunnel drops, and a **restart a dead tunnel** setting (on by default for new installs).
- **Crash log.** If sing-box or the tunnel stops on its own you get a notification with the reason, and an entry in a crash log you can open inside the app.
- **Kill switch** (experimental, TUN mode, off by default). It blocks traffic if the tunnel stops, and keeps blocking through restarts and failed reconnects until the new tunnel is verified. Home says when it is blocking and has an **Unblock internet** button. It allows IPv6 neighbour discovery and your server's addresses (looked up again every few minutes), and lets DNS through for up to 20 seconds while a reconnect starts. It has been tested against a stub firewall only, not yet on the real one.
- **Connect or disconnect by network.** Remember the networks you use and choose for each one whether the VPN connects, disconnects or does nothing when you join it. Networks are recognised by their router, so no location permission is needed.
- **Local proxy password** (optional). Protect the proxy on `127.0.0.1` with a login so other users on your Mac cannot use your tunnel.
- **Routing rules** with Global, Rules and Direct modes. Match by domain, domain suffix, keyword, IP range, process name, or an **app picked from your installed apps** (split tunneling: the rule covers the app's helper processes too), send traffic through the VPN, direct, or block it, bypass your local network, and choose your DNS resolver (Cloudflare, Google, Quad9, the system resolver, or your own DoH, DoT, DoQ or plain server). An **Allow IPv6** switch (off for new installs) keeps IPv6 out of the tunnel for servers without it, and optional **FakeIP** routes Full System traffic by site name. A built-in tester shows which rule a domain would hit.
- **Ready-made lists** picked by name (countries, ad networks, Google, Netflix, Telegram and dozens more) instead of codes, kept up to date for you. Choose where they come from (SagerNet, MetaCubeX, v2rayN, Loyalsoldier, runetfreedom, or your own address), and download, update or delete them from the app. Downloads go through the VPN first when connected, so a network that blocks GitHub does not matter.
- **Live connections** with an **Apps** view (which app is sending and receiving data right now, with live speeds, and a **Route** button to always send it through the VPN, directly or not at all) and a table of every open connection, its app, destination, route and traffic, with the option to close any of them.

### Apps

- **App controls.** A page listing the apps installed on your Mac, each with its icon and *VPN*, *Direct* and *Block* buttons (split tunneling without typing executable names). Apps without a choice follow your routing default, and blocked apps are gathered at the top with an Unblock button. The app itself and its engine cannot be blocked.
- **Data caps for apps.** Give any app a daily, weekly or monthly limit. You are told at 80% and 100%, and can choose to be offered **Block now** at the limit; Home shows the offer as a card. A block lifts itself when the period starts over. Counting is approximate and covers traffic through the VPN.

### Insight

- **Diagnostics** with a leak check (tunnel IP, system route and DNS resolver side by side) and a live **speed test** gauge for ping, jitter, download and upload through the tunnel.
- **Data usage** that opens on this month in plain language, with a day-by-day chart and a "Where did it go?" list of apps, sites and servers. An optional monthly limit and the privacy options live in Settings. Export to CSV. Detailed app and site recording can be turned off.
- **Logs** with level filters, search and plain-language hints for common errors.

### Engine and updates

- **Engine manager.** Download, switch between and remove sing-box versions on the stable or pre-release channel. Downloads run in the background with progress, are checksum-verified, and never replace a working engine if they fail.
- **Update policy** for the engine and for the app: *Ask me*, *Install automatically* or *Do nothing*. Automatic installs never interrupt a live connection, and restarting is always your choice.
- **Download manager** in the title bar for progress, cancel and retry.

### Polish

- **Light, dark or system** theme, switchable from the sidebar.
- **Responsive layout** that collapses to an icon rail in narrow windows.
- **Backup and restore** of your servers, subscriptions, routing and settings in one file, to keep or to move to another Mac.
- **Feedback form** (when the build is configured with a feedback endpoint) that shows exactly what would be sent and redacts server addresses and keys before anything leaves your Mac.

### Privacy and safety

- **No telemetry and no analytics.** The app only talks to your servers, the sing-box release host, and the diagnostic services you trigger yourself (IP lookup and Cloudflare speed test).
- **Everything is stored locally**, in `~/Library/Application Support/com.apzur.kage`, with owner-only permissions and atomic writes. A damaged data file is set aside rather than overwritten.

## Screenshots

All screenshots use made-up demo data: example hosts and documentation-range IP addresses.

| | |
|---|---|
| <img src="docs/images/servers.png" alt="Servers" width="430"><br>**Servers**: grouped, searchable, with live latency | <img src="docs/images/home-dark.png" alt="Home in dark mode" width="430"><br>**Dark mode**: system, light or dark |
| <img src="docs/images/usage.png" alt="Data usage" width="430"><br>**Data usage**: totals, trend and per-app breakdown | <img src="docs/images/diagnostics.png" alt="Diagnostics and speed test" width="430"><br>**Diagnostics**: leak check and speed test |
| <img src="docs/images/routing.png" alt="Routing" width="430"><br>**Routing**: rules, ready-made lists and DNS | <img src="docs/images/connections.png" alt="Live connections" width="430"><br>**Connections**: every open connection, live |
| <img src="docs/images/engine.png" alt="Engine manager" width="430"><br>**Engine**: versions, channels and update policy | <img src="docs/images/logs.png" alt="Logs" width="430"><br>**Logs**: filters, search and error hints |
| <img src="docs/images/settings.png" alt="Settings" width="430"><br>**Settings**: reliability, kill switch and networks | <img src="docs/images/welcome.png" alt="Welcome tour" width="430"><br>**Welcome tour**: first-run setup |
| <img src="docs/images/apps.png" alt="App controls" width="430"><br>**App controls**: VPN, Direct or Block for each app | <img src="docs/images/app-cap-editor.png" alt="Data cap editor" width="430"><br>**Data caps**: a daily, weekly or monthly limit per app |
| <img src="docs/images/home-cap-offer.png" alt="Home with a data cap offer" width="430"><br>**Block now**: offered when an app reaches its cap | <img src="docs/images/connections-apps.png" alt="Connections, Apps view" width="430"><br>**Connections, Apps**: live speeds and a Route button |
| <img src="docs/images/routing-lists.png" alt="Ready-made lists" width="430"><br>**Ready-made lists**: countries and services by name | <img src="docs/images/settings-usage.png" alt="Data limit and usage privacy settings" width="430"><br>**Data limit and privacy**: what is recorded and kept |
| <img src="docs/images/tour-apps.png" alt="Setup guide, per-app step" width="430"><br>**Setup guide**: now covers rules, apps and data caps | |

<div align="center">
<img src="docs/images/menu-bar-popover.png" alt="Menu bar popover" width="240"><br>
<b>Menu-bar popover</b>
</div>

## Build from source

You need macOS, [Node.js](https://nodejs.org) 20 or newer, [Rust](https://rustup.rs) (stable) and the Xcode command line tools (`xcode-select --install`).

```bash
git clone https://github.com/Shashi-Madushan/KageVPN.git
cd KageVPN
npm install
npm run tauri dev          # run the app with hot reload
```

Build an app bundle:

```bash
npm run build:app
```

The result is in `src-tauri/target/release/bundle/macos/`. This is an unsigned local build. A plain `npm run tauri build` asks for the update signing key (`TAURI_SIGNING_PRIVATE_KEY`) because the project is set up to produce signed update archives; use `npm run release` for that (see [docs/RELEASING.md](docs/RELEASING.md)).

### Checks

```bash
npm run check              # Svelte and TypeScript
npm run test:scripts       # release script helpers
(cd src-tauri && cargo test)   # Rust unit tests
```

### Regenerate the screenshots

```bash
npm run screenshots        # needs Chrome, Brave or Edge; writes docs/images
```

### Project layout

| Path | What lives there |
|---|---|
| `src/` | Svelte 5 interface: pages, components and state |
| `src-tauri/src/app_config.rs` | The one place for names, cores, release and feedback endpoints, and outside services |
| `src-tauri/src/core/` | Engine download, verification and version management |
| `src-tauri/src/downloads.rs` | Shared download manager used by the engine and app updater |
| `src-tauri/src/app_update.rs` | Signed app updates |
| `src-tauri/src/feedback.rs` | Feedback reports and redaction |
| `src-tauri/src/profile/` | One module per protocol (link parsing, validation, sing-box outbound) behind a registry in `mod.rs` |
| `src-tauri/src/config.rs`, `session.rs`, `runner/` | sing-box config generation and process control |
| `src-tauri/src/tests/` | Rust unit tests |
| `scripts/` | Release and screenshot tooling |
| `docs/` | Design notes, the release guide and screenshots |

To rename the app or point it at your own release and feedback servers, edit `src-tauri/src/app_config.rs`. For shipping a new version, see [docs/RELEASING.md](docs/RELEASING.md).

## Roadmap

- Apple notarization, so first launch needs no extra steps.
- More engine types alongside sing-box. The download side is already built; each new engine needs a small adapter.
- A published release feed for automatic app updates.

## Contributing

Issues and pull requests are welcome. Please run the checks above before sending a change.

## Author and contact

**Shashi Madushan**, software developer and the maker of Kage VPN.

- GitHub: [@Shashi-Madushan](https://github.com/Shashi-Madushan)
- Email: [shashisulakshan@gmail.com](mailto:shashisulakshan@gmail.com)
<!-- LinkedIn: add your profile link here, for example: - LinkedIn: [Your Name](https://www.linkedin.com/in/your-handle) -->

## License

[MIT](LICENSE) © 2026 Shashi Madushan. sing-box is a separate project with its own license; Kage VPN downloads and runs it but does not modify it.

---

<div align="center">
<sub>Kage by <b>Apzur</b></sub>
</div>
