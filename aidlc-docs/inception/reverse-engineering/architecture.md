# Architecture (condensed)

Kage VPN is a macOS menu-bar VPN manager. A Tauri 2 shell (Rust) drives a downloaded sing-box engine; a SvelteKit UI shows state.

```mermaid
flowchart LR
    UI["SvelteKit UI (src/)"] -->|invoke| CMD["commands.rs"]
    CMD --> SES["session.rs connect/disconnect"]
    SES --> CFG["config.rs builds sing-box JSON"]
    SES --> RUN["runner/ helper.sh, supervisor.sh, daemon.rs"]
    RUN --> SB["sing-box (root, TUN)"]
    CMD --> STORE["store.rs profiles and settings"]
    CMD --> SUBS["subscriptions.rs"]
    SES --> REL["reliability.rs, network_watch.rs, health.rs"]
    TRAY["tray.rs and routes/tray"] --> SES
```

Text alternative: UI calls Tauri commands; session builds a config via config.rs and starts sing-box through the root runner; store holds data; reliability modules watch network and health; tray is a second UI.

Key facts: profile protocols are modules under src-tauri/src/profile (vless, vmess, trojan, shadowsocks, hysteria2; see docs/ADDING-A-PROTOCOL.md). Settings live in settings.rs (RoutingSettings, ReliabilitySettings, UsageSettings). Kill switch uses pfctl via the root helper. Root helper trusts user-writable files (audit O1).
