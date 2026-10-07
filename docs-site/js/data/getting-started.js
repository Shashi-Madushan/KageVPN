/**
 * Apzur VPN Documentation — Getting Started Pages
 */
window.DOCS_PAGES = window.DOCS_PAGES || {};

window.DOCS_PAGES["getting-started/overview"] = {
  category: "Getting Started",
  title: "Introduction & Overview",
  featureBadge: "v0.2.0 Native",
  lead: "Apzur VPN is a modern, lightweight, native-feeling macOS VPN client engineered around the sing-box proxy engine. It unites enterprise-grade proxy protocols with an elegant macOS interface built using Tauri 2 and Svelte 5.",
  heroImage: {
    src: "assets/images/home.png",
    alt: "Apzur VPN Dashboard Connected State",
    title: "Apzur VPN — Main Control Dashboard"
  },
  toc: [
    { id: "what-is-apzur", label: "What is Apzur VPN?" },
    { id: "key-capabilities", label: "Key Capabilities" },
    { id: "quick-start-steps", label: "Quick Start Workflow" },
    { id: "architecture-at-a-glance", label: "Architecture at a Glance" }
  ],
  contentHtml: `
    <div class="callout-banner info" id="what-is-apzur">
      <div class="callout-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
      </div>
      <div>
        <strong>System Requirement:</strong> macOS 12 Monterey or later (Apple Silicon M1/M2/M3/M4 and Intel x86_64). Apzur runs natively on both architectures without Rosetta emulation.
      </div>
    </div>

    <div class="docs-prose">
      <h2>What is Apzur VPN?</h2>
      <p>
        Most commercial VPN applications force users into closed subscription networks with proprietary protocols and heavy resource usage. Apzur VPN takes a different path: you bring your own server credentials or subscription feed, and Apzur provides a high-performance, open-source macOS native client that manages the <code>sing-box</code> engine.
      </p>
      <p>
        Whether you are bypassing network firewalls, securing remote public Wi-Fi, or orchestrating multi-protocol server fleets, Apzur gives you absolute visibility over your routing, active sockets, and encrypted traffic.
      </p>

      <h2 id="key-capabilities">Key Capabilities</h2>
      <div class="docs-table-wrapper">
        <table class="docs-table">
          <thead>
            <tr>
              <th>Feature</th>
              <th>Apzur Implementation</th>
              <th>Benefit</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><strong>Supported Protocols</strong></td>
              <td>VLESS (Reality/Vision), VMess, Trojan, Shadowsocks 2022, Hysteria 2</td>
              <td>State-of-the-art censorship resistance and wire-speed throughput.</td>
            </tr>
            <tr>
              <td><strong>Dual Tunnel Modes</strong></td>
              <td>Virtual TUN (<code>utun</code>) and Local Proxy Only</td>
              <td>Choose between 100% whole-system encryption or selective proxying.</td>
            </tr>
            <tr>
              <td><strong>macOS pfctl Kill Switch</strong></td>
              <td>Kernel packet filter anchor integration</td>
              <td>Guarantees 0-byte packet leaks if the proxy tunnel unexpectedly drops.</td>
            </tr>
            <tr>
              <td><strong>Memory Footprint</strong></td>
              <td>Tauri 2 (Rust) + Svelte 5 (~45 MB RAM)</td>
              <td>Runs quietly in the menu bar without draining Apple Silicon battery.</td>
            </tr>
            <tr>
              <td><strong>Privacy Redaction</strong></td>
              <td>Automated log sanitizer</td>
              <td>Export diagnostic logs safely without leaking private keys or server IPs.</td>
            </tr>
          </tbody>
        </table>
      </div>

      <h2 id="quick-start-steps">Quick Start Workflow</h2>
    </div>

    <div class="step-sequence">
      <div class="step-item done">
        <div class="step-number-badge">1</div>
        <div class="step-head">
          <div class="step-title">Download & Install DMG</div>
          <span class="step-badge-tag">macOS Disk Image</span>
        </div>
        <div class="step-body">
          <p>Download the latest release disk image from GitHub. Drag <code>Apzur VPN.app</code> into your macOS <code>/Applications</code> directory.</p>
        </div>
      </div>

      <div class="step-item done">
        <div class="step-number-badge">2</div>
        <div class="step-head">
          <div class="step-title">Bypass macOS Gatekeeper</div>
          <span class="step-badge-tag">One-time Action</span>
        </div>
        <div class="step-body">
          <p>Because Apzur is distributed directly via GitHub Releases, macOS Gatekeeper may show a warning. Right-click and choose <strong>Open</strong> or approve in <strong>System Settings &rarr; Privacy & Security</strong>.</p>
        </div>
      </div>

      <div class="step-item">
        <div class="step-number-badge">3</div>
        <div class="step-head">
          <div class="step-title">Import Server or Subscription</div>
          <span class="step-badge-tag">One Click</span>
        </div>
        <div class="step-body">
          <p>Paste a raw link (e.g. <code>vless://...</code>, <code>hysteria2://...</code>) or paste a provider subscription URL. Apzur automatically parses endpoints, latency ping, and security certificates.</p>
        </div>
      </div>

      <div class="step-item">
        <div class="step-number-badge">4</div>
        <div class="step-head">
          <div class="step-title">Connect & Verify</div>
          <span class="step-badge-tag">Ready</span>
        </div>
        <div class="step-body">
          <p>Click the circular Power button in the dashboard or menu bar. Enter your macOS administrator password once if using TUN mode to authorize network interface creation.</p>
        </div>
      </div>
    </div>

    <div class="quote-callout-card" id="architecture-at-a-glance">
      <div class="quote-callout-text">
        "Apzur VPN is built for developers and privacy advocates who want complete control over their proxy tunnels without wrestling with raw configuration files or sluggish Electron apps."
      </div>
      <div class="quote-callout-author">— Shashi Madushan, Maintainer</div>
    </div>
  `
};

window.DOCS_PAGES["getting-started/gatekeeper"] = {
  category: "Getting Started",
  title: "macOS Gatekeeper & Security Fix",
  featureBadge: "Troubleshooting",
  lead: "How to resolve 'Apzur VPN cannot be opened because Apple cannot verify malware' or 'damaged and can't be opened' messages on macOS Sequoia, Sonoma, and earlier.",
  heroImage: {
    src: "assets/images/welcome.png",
    alt: "macOS Gatekeeper Warning and Welcome Screen",
    title: "macOS Security Verification"
  },
  toc: [
    { id: "why-this-happens", label: "Why This Happens" },
    { id: "option-a", label: "Option A: System Settings (Recommended)" },
    { id: "option-b", label: "Option B: Terminal Quarantine Removal" },
    { id: "verifying-binary", label: "Verifying Binary Integrity" }
  ],
  contentHtml: `
    <div class="callout-banner warn" id="why-this-happens">
      <div class="callout-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
      </div>
      <div>
        <strong>Unsigned Application:</strong> Apzur VPN is an independent open-source project and is not notarized through a paid Apple Developer subscription. macOS automatically applies an extended quarantine attribute (<code>com.apple.quarantine</code>) to downloaded binaries.
      </div>
    </div>

    <div class="docs-prose">
      <h2 id="option-a">Option A: Open Anyway via System Settings</h2>
      <p>This is the standard graphical workflow on macOS 14 Sonoma and macOS 15 Sequoia:</p>
    </div>

    <div class="step-sequence">
      <div class="step-item">
        <div class="step-number-badge">1</div>
        <div class="step-head">
          <div class="step-title">Trigger the Gatekeeper Dialog</div>
        </div>
        <div class="step-body">
          <p>Double-click <code>/Applications/Apzur VPN.app</code>. When macOS shows the security warning, click <strong>Done</strong> (or Cancel).</p>
        </div>
      </div>

      <div class="step-item">
        <div class="step-number-badge">2</div>
        <div class="step-head">
          <div class="step-title">Navigate to Privacy & Security</div>
        </div>
        <div class="step-body">
          <p>Open <strong>Apple Menu &rarr; System Settings &rarr; Privacy & Security</strong>. Scroll down to the <strong>Security</strong> section.</p>
        </div>
      </div>

      <div class="step-item done">
        <div class="step-number-badge">3</div>
        <div class="step-head">
          <div class="step-title">Click 'Open Anyway'</div>
          <span class="step-badge-tag">Authorize</span>
        </div>
        <div class="step-body">
          <p>You will see the notice: <em>"Apzur VPN was blocked to protect your Mac."</em> Click <strong>Open Anyway</strong>, confirm with your Touch ID or password, and click <strong>Open</strong> in the subsequent prompt.</p>
        </div>
      </div>
    </div>

    <div class="docs-prose">
      <h2 id="option-b">Option B: Remove Quarantine via Terminal (Permanent Fix)</h2>
      <p>If macOS displays <em>"Apzur VPN is damaged and can't be opened"</em>, stripping the quarantine attribute immediately resolves the issue:</p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="xattr -dr com.apple.quarantine &quot;/Applications/Apzur VPN.app&quot;">Copy Command</button>
      </div>
      <pre><code>xattr -dr com.apple.quarantine "/Applications/Apzur VPN.app"</code></pre>
    </div>

    <div class="docs-prose">
      <p>Once executed, open Apzur VPN normally from Launchpad or Finder. You will never be prompted with quarantine warnings for that installation again.</p>

      <h2 id="verifying-binary">Verifying Binary Integrity with Minisign</h2>
      <p>Every official release is accompanied by a SHA-256 checksum and Minisign cryptographic signature. To verify that your download has not been tampered with:</p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="minisign -Vm &quot;Apzur-VPN.dmg&quot; -P RWTw9...YourPublicKey">Verify Signature</button>
      </div>
      <pre><code>minisign -Vm "Apzur-VPN.dmg" -P RWTw9...OfficialPublicKey</code></pre>
    </div>
  `
};

window.DOCS_PAGES["getting-started/first-run"] = {
  category: "Getting Started",
  title: "First Run & Onboarding Tour",
  featureBadge: "Workflow",
  lead: "Take a tour through Apzur VPN's dual-interface architecture: the swift menu bar popover for day-to-day operations and the comprehensive dashboard for granular controls.",
  heroImage: {
    src: "assets/images/menu-bar-popover.png",
    alt: "Apzur Menu Bar Popover",
    title: "Menu Bar Quick Controller"
  },
  toc: [
    { id: "interface-layout", label: "Dual Interface Architecture" },
    { id: "menu-bar-controller", label: "Menu Bar Quick Popover" },
    { id: "main-dashboard", label: "Full Dashboard Window" },
    { id: "permission-setup", label: "Initial Permission Setup" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="interface-layout">Dual Interface Architecture</h2>
      <p>
        Apzur VPN is intentionally designed around macOS Human Interface Guidelines. Rather than forcing a persistent desktop window, the app runs as a lightweight menu bar resident with on-demand expansion.
      </p>

      <h2 id="menu-bar-controller">Menu Bar Quick Popover</h2>
      <p>
        Clicking the shield icon in the macOS menu bar triggers an ultra-fast popover allowing you to:
      </p>
      <ul>
        <li><strong>Toggle Connection:</strong> One-tap connect or disconnect without interrupting your active workspace.</li>
        <li><strong>Server Switcher:</strong> Quickly swap between favorite nodes with live latency indicators.</li>
        <li><strong>Realtime Speeds:</strong> Instant glance at current upload (TX) and download (RX) throughput.</li>
        <li><strong>Mode Indicator:</strong> See whether Full System TUN or Proxy Only mode is currently operational.</li>
      </ul>

      <h2 id="main-dashboard">Full Dashboard Window</h2>
      <p>
        Clicking <strong>Open Dashboard</strong> or selecting it from the menu bar launches the complete management console:
      </p>
      <ul>
        <li><strong>Servers:</strong> Comprehensive node organizer, latency pings, and subscription manager.</li>
        <li><strong>Routing:</strong> Rule-based traffic routing editor with direct GeoIP / GeoSite bypasses.</li>
        <li><strong>Connections:</strong> Live socket inspector powered by sing-box's Clash API ring buffer.</li>
        <li><strong>Diagnostics:</strong> One-click DNS and IPv6 leak diagnostics and latency graphs.</li>
        <li><strong>Settings:</strong> Kill switch activation, core engine updater, and privacy toggles.</li>
      </ul>

      <h2 id="permission-setup">Initial Permission Authorization</h2>
      <p>
        When you connect in <strong>Full System (TUN)</strong> mode for the first time, macOS requests root authorization. This is required because creating a virtual network adapter (<code>utun</code>) and binding system routes requires elevated kernel privileges:
      </p>
    </div>

    <div class="callout-banner info">
      <div class="callout-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
      </div>
      <div>
        <strong>Root Privileges Scoped to Tunnel Daemon:</strong> Apzur does not run the entire graphical interface as root. Elevated privileges are strictly scoped to the sing-box helper process responsible for virtual interface allocation.
      </div>
    </div>
  `
};

window.DOCS_PAGES["getting-started/engine-setup"] = {
  category: "Getting Started",
  title: "Engine Setup & Version Manager",
  featureBadge: "sing-box Core",
  lead: "Manage the underlying sing-box core binary. Learn about managed core downloads, checksum verification, and switching to a custom Homebrew installation.",
  heroImage: {
    src: "assets/images/engine.png",
    alt: "Engine Management Screen",
    title: "sing-box Core Engine Manager"
  },
  toc: [
    { id: "engine-modes", label: "Managed Core vs System Core" },
    { id: "managed-download", label: "Automatic Core Downloader" },
    { id: "custom-binary", label: "Using Homebrew or Custom Binary" },
    { id: "required-tags", label: "Required Build Tags" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="engine-modes">Managed Core vs System Core</h2>
      <p>
        Apzur VPN decouples its graphical client from the proxy engine. You can choose between:
      </p>
      <div class="docs-table-wrapper">
        <table class="docs-table">
          <thead>
            <tr>
              <th>Engine Mode</th>
              <th>Binary Location</th>
              <th>Management Method</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><strong>Managed Core (Default)</strong></td>
              <td><code>~/Library/Application Support/com.apzur.vpn/bin/sing-box</code></td>
              <td>Downloaded and verified automatically within the app with SHA-256 checks.</td>
            </tr>
            <tr>
              <td><strong>System Core</strong></td>
              <td><code>/opt/homebrew/bin/sing-box</code> or <code>/usr/local/bin/sing-box</code></td>
              <td>Managed externally via Homebrew (<code>brew install sing-box</code>).</td>
            </tr>
          </tbody>
        </table>
      </div>

      <h2 id="managed-download">Automatic Core Downloader</h2>
      <p>
        On first launch, if no valid sing-box binary is detected, Apzur displays a setup prompt. Clicking <strong>Download Engine</strong> pulls the latest verified release from GitHub Releases, checks its official SHA-256 checksum, marks the executable with <code>chmod +x</code>, and stores it in your application support folder.
      </p>

      <h2 id="custom-binary">Using Homebrew or Custom Binary</h2>
      <p>
        If you prefer managing your system tools through Homebrew:
      </p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="brew install sing-box">Copy</button>
      </div>
      <pre><code>brew install sing-box
which sing-box
# Output: /opt/homebrew/bin/sing-box</code></pre>
    </div>

    <div class="docs-prose">
      <p>Navigate to <strong>Settings &rarr; Core Engine</strong>, select <strong>System Core</strong>, and select or input the path to your installed binary.</p>

      <h2 id="required-tags">Required Build Tags</h2>
      <p>
        If compiling sing-box yourself from source, ensure the following tags are active:
      </p>
      <ul>
        <td><code>with_clash_api</code> — Enables real-time traffic statistics and connection telemetry.</td>
        <td><code>with_utls</code> — Provides TLS fingerprint randomization for VLESS Reality.</td>
        <td><code>with_quic</code> — Enables Hysteria 2 and TUIC high-speed transport engines.</td>
      </ul>
    </div>
  `
};
