/**
 * Kage VPN Documentation — Navigation Tree Structure
 */
window.DOCS_NAVIGATION = [
  {
    category: "Getting Started",
    icon: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>`,
    items: [
      {
        slug: "getting-started/overview",
        title: "Overview & Features",
        badge: "Guide",
        description: "What Kage VPN is, core features, and architectural principles."
      },
      {
        slug: "getting-started/gatekeeper",
        title: "macOS Gatekeeper Bypass",
        badge: "Security",
        description: "How to fix 'Kage VPN cannot be opened' or 'damaged' quarantine errors on macOS."
      },
      {
        slug: "getting-started/first-run",
        title: "First Run & Tour",
        badge: "Setup",
        description: "Initial startup, menu bar popover, and permission authorization."
      },
      {
        slug: "getting-started/engine-setup",
        title: "Engine Setup & Manager",
        badge: "Core",
        description: "Downloading the official sing-box engine with SHA-256 checks or using Homebrew."
      }
    ]
  },
  {
    category: "User Guide",
    icon: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/></svg>`,
    items: [
      {
        slug: "user-guide/connection-modes",
        title: "TUN vs Proxy Modes",
        badge: "Networking",
        description: "Difference between Full System Virtual TUN (utun) and local Proxy Only mode."
      },
      {
        slug: "user-guide/servers-subscriptions",
        title: "Servers & Subscriptions",
        badge: "Config",
        description: "Importing links, subscription URLs, auto-updating, and format validation."
      },
      {
        slug: "user-guide/routing-rules",
        title: "Routing & Rules Engine",
        badge: "Traffic",
        description: "Smart traffic splitting, GeoIP/GeoSite bypass, and custom domain rules."
      },
      {
        slug: "user-guide/kill-switch",
        title: "Kill Switch & Connection Health",
        badge: "Safety",
        description: "macOS pfctl anchor locks preventing traffic leaks during disconnects."
      },
      {
        slug: "user-guide/diagnostics-speedtest",
        title: "Leak Tests & Speedtest",
        badge: "Audit",
        description: "Compare the tunnel with your normal route, see the DNS resolver, and measure speed."
      },
      {
        slug: "user-guide/data-usage",
        title: "Data Usage & Limits",
        badge: "Telemetry",
        description: "Tracking hourly, daily, and monthly bandwidth with SQLite persistence."
      }
    ]
  },
  {
    category: "Architecture",
    icon: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><path d="M13 6h3a2 2 0 0 1 2 2v7"/><line x1="6" y1="9" x2="6" y2="21"/></svg>`,
    items: [
      {
        slug: "architecture/system-overview",
        title: "Subsystem Overview",
        badge: "Design",
        description: "High-level topology: Svelte 5, Tauri 2 IPC, Rust Core, helper daemon, sing-box."
      },
      {
        slug: "architecture/tauri-ipc",
        title: "Tauri 2 IPC & State",
        badge: "Rust",
        description: "IPC invoke commands, asynchronous event emitters, and thread-safe Mutex state."
      },
      {
        slug: "architecture/core-supervision",
        title: "Core Supervision & Stopfile",
        badge: "Process",
        description: "Rootless supervision, stopfile handshake, PID monitoring, and zombie process cleanup."
      },
      {
        slug: "architecture/tun-pfctl",
        title: "TUN & macOS pfctl Routing",
        badge: "Kernel",
        description: "utun adapter creation, system default gateway overrides, and packet filter anchors."
      },
      {
        slug: "architecture/clash-ring-buffer",
        title: "Clash API & Ring Buffer",
        badge: "Memory",
        description: "In-memory circular buffer feeding realtime connections and bandwidth graphs."
      },
      {
        slug: "architecture/config-pipeline",
        title: "Config Generator Pipeline",
        badge: "Engine",
        description: "Translating GUI models into valid, deterministic sing-box JSON runtime configurations."
      },
      {
        slug: "architecture/privacy-redaction",
        title: "Secret Redaction Engine",
        badge: "Security",
        description: "Sanitizing passwords, UUIDs, server IPs, and keys from diagnostic logs."
      }
    ]
  },
  {
    category: "Developer Guide",
    icon: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>`,
    items: [
      {
        slug: "developer-guide/local-setup",
        title: "Local Development Setup",
        badge: "Dev",
        description: "Setting up Node, Rust toolchain, Tauri CLI, and compiling the app on macOS."
      },
      {
        slug: "developer-guide/svelte-frontend",
        title: "Svelte 5 Frontend",
        badge: "Frontend",
        description: "Svelte 5 runes ($state, $derived, $effect), Tailwind CSS, and reactive stores."
      },
      {
        slug: "developer-guide/rust-backend",
        title: "Rust Backend & app_config",
        badge: "Backend",
        description: "State persistence, Tauri command handlers, and atomic file writes."
      },
      {
        slug: "developer-guide/adding-protocols",
        title: "Adding a Protocol (Trait)",
        badge: "Extending",
        description: "Implementing ProtocolDef trait, URI parsing, sing-box outbound formatting, and UI."
      },
      {
        slug: "developer-guide/releasing",
        title: "Releasing & Signing",
        badge: "CI/CD",
        description: "Minisign key signatures, GitHub Releases automation, and DMG bundling."
      },
      {
        slug: "developer-guide/testing",
        title: "Testing & Conformance",
        badge: "QA",
        description: "Running cargo test, svelte-check, and proxy protocol conformance test suite."
      }
    ]
  },
  {
    category: "Protocol Reference",
    icon: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>`,
    items: [
      {
        slug: "protocol-reference/vless",
        title: "VLESS (Reality & Vision)",
        badge: "Protocol",
        description: "Zero-decryption proxy protocol with TLS camouflage (Reality) and flow control."
      },
      {
        slug: "protocol-reference/vmess",
        title: "VMess",
        badge: "Protocol",
        description: "Classic V2Ray protocol supporting WebSocket, gRPC, TLS, and multiplexing."
      },
      {
        slug: "protocol-reference/trojan",
        title: "Trojan",
        badge: "Protocol",
        description: "Unidentifiable HTTPS proxy protocol imitating standard TLS traffic."
      },
      {
        slug: "protocol-reference/shadowsocks",
        title: "Shadowsocks 2022",
        badge: "Protocol",
        description: "Modernized AEAD ciphers (2022-blake3-aes-128-gcm) with replay resistance."
      },
      {
        slug: "protocol-reference/hysteria2",
        title: "Hysteria 2 (Salamander)",
        badge: "Protocol",
        description: "High-throughput QUIC/UDP protocol with Salamander port obfuscation."
      }
    ]
  },
  {
    category: "Changelog",
    items: [
      {
        slug: "changelog/changelog",
        title: "Changelog",
        description: "What changed in each version of Kage VPN."
      }
    ]
  }
];
