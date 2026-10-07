/**
 * Kage VPN Documentation — User Guide Pages
 */
window.DOCS_PAGES = window.DOCS_PAGES || {};

window.DOCS_PAGES["user-guide/connection-modes"] = {
  category: "User Guide",
  title: "TUN vs Proxy Only Modes",
  featureBadge: "Networking",
  lead: "Understand the differences between Full System Virtual TUN (utun) mode and local Proxy Only mode, and determine the optimal setup for your workflow.",
  heroImage: {
    src: "assets/images/home-dark.png",
    alt: "Kage Connection Mode Selection in Dark Theme",
    title: "Connection Mode Configuration"
  },
  toc: [
    { id: "modes-comparison", label: "Mode Comparison" },
    { id: "tun-mode-deep-dive", label: "Full System TUN Mode" },
    { id: "proxy-only-mode", label: "Proxy Only Mode" },
    { id: "dns-handling", label: "DNS Hijacking & Resolution" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="modes-comparison">Mode Comparison</h2>
      <p>
        Kage VPN supports two primary operation modes depending on whether you require transparent system-wide tunneling or application-specific proxy redirection:
      </p>

      <div class="docs-table-wrapper">
        <table class="docs-table">
          <thead>
            <tr>
              <th>Dimension</th>
              <th>Full System (TUN Mode)</th>
              <th>Proxy Only Mode</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><strong>Scope</strong></td>
              <td>100% of all apps (browsers, Terminal, background daemons, games)</td>
              <td>Apps respecting macOS system proxy settings (browsers, curl)</td>
            </tr>
            <tr>
              <td><strong>Virtual Adapter</strong></td>
              <td>Allocates <code>utunX</code> interface via macOS kernel</td>
              <td>None (binds local SOCKS5 / HTTP loopback ports)</td>
            </tr>
            <tr>
              <td><strong>Privileges</strong></td>
              <td>Requires root permission once via helper daemon</td>
              <td>Runs entirely in user-space without root</td>
            </tr>
            <tr>
              <td><strong>UDP / DNS Traffic</strong></td>
              <td>Intercepts all UDP, ICMP ping, and DNS queries</td>
              <td>Only TCP proxied; UDP depends on client SOCKS5 support</td>
            </tr>
            <tr>
              <td><strong>Kill Switch Support</strong></td>
              <td>Fully protected via <code>pfctl</code> packet filter</td>
              <td>Application-dependent; packets can bypass proxy</td>
            </tr>
          </tbody>
        </table>
      </div>

      <h2 id="tun-mode-deep-dive">Full System TUN Mode</h2>
      <p>
        In TUN mode, Kage instructs sing-box to spin up a virtual point-to-point network adapter (e.g. <code>utun3</code>). It modifies the macOS kernel routing table to route <code>0.0.0.0/1</code> and <code>128.0.0.0/1</code> through the virtual adapter, overriding your default gateway without altering the underlying physical Wi-Fi or Ethernet routing tables.
      </p>
      <p>
        This ensures zero configuration is needed across terminal commands, Git pushes, Docker daemons, or non-proxy aware desktop software.
      </p>

      <h2 id="proxy-only-mode">Proxy Only Mode</h2>
      <p>
        If you are on a restricted machine without administrator privileges or only need a browser to browse through an encrypted server:
      </p>
      <ul>
        <li>Kage launches sing-box with a mixed SOCKS5/HTTP inbound listener on <code>127.0.0.1:2080</code>.</li>
        <li>It utilizes macOS <code>networksetup</code> to toggle the HTTP and SOCKS proxy state on the active network adapter (e.g. Wi-Fi).</li>
        <li>When disconnected, Kage automatically turns off proxy settings in System Settings.</li>
      </ul>

      <h2 id="dns-handling">DNS Hijacking & Resolution</h2>
      <p>
        In TUN mode, sing-box captures port 53 traffic, eliminating DNS poisoning and ISP snooping. All DNS queries are routed directly through your remote outbound server via DNS-over-HTTPS (DoH) or DNS-over-TLS (DoT).
      </p>
    </div>
  `
};

window.DOCS_PAGES["user-guide/servers-subscriptions"] = {
  category: "User Guide",
  title: "Servers & Subscriptions Management",
  featureBadge: "Node Fleet",
  lead: "Import, test, organize, and automate updates for multi-protocol node fleets and subscription links.",
  heroImage: {
    src: "assets/images/servers.png",
    alt: "Server List with Latency and Protocol Badges",
    title: "Server & Node Manager"
  },
  toc: [
    { id: "adding-servers", label: "Adding Servers" },
    { id: "subscriptions", label: "Managing Subscriptions" },
    { id: "latency-testing", label: "Latency & Ping Testing" },
    { id: "server-tagging", label: "Favorites & Node Renaming" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="adding-servers">Adding Servers</h2>
      <p>Kage VPN provides flexible import mechanisms to accommodate various configurations:</p>
    </div>

    <div class="step-sequence">
      <div class="step-item">
        <div class="step-number-badge">1</div>
        <div class="step-head">
          <div class="step-title">Paste Raw Share Links</div>
          <span class="step-badge-tag">Clipboard</span>
        </div>
        <div class="step-body">
          <p>Copy any standard proxy link to your clipboard and hit <strong>Import</strong> or press <kbd>⌘V</kbd> inside the Servers tab. Kage supports:</p>
          <ul>
            <li><code>vless://...#ServerName</code> (including Reality, Vision, and Flow parameters)</li>
            <li><code>vmess://...</code> (standard Base64 JSON configurations)</li>
            <li><code>trojan://...#ServerName</code></li>
            <li><code>ss://...#ServerName</code> (SIP002 and 2022 AEAD ciphers)</li>
            <li><code>hysteria2://...#ServerName</code> (including Salamander obfuscation)</li>
          </ul>
        </div>
      </div>

      <div class="step-item">
        <div class="step-number-badge">2</div>
        <div class="step-head">
          <div class="step-title">Batch Import via File</div>
          <span class="step-badge-tag">Drag & Drop</span>
        </div>
        <div class="step-body">
          <p>Drag any <code>.txt</code> or <code>.json</code> file containing multiple links onto the application window to batch-import dozens of servers in seconds.</p>
        </div>
      </div>
    </div>

    <div class="docs-prose">
      <h2 id="subscriptions">Managing Subscriptions</h2>
      <p>
        If your provider supplies a subscription URL, click <strong>Add Subscription</strong>. Kage fetches the remote endpoint, decodes Base64 encoded payload manifests, extracts individual nodes, and tags them with the provider group name.
      </p>
      <ul>
        <li><strong>Auto Update Schedule:</strong> Configure intervals (Every 6 hours, 12 hours, or Daily).</li>
        <li><strong>Manual Refresh:</strong> Click the refresh icon beside the subscription group to pull updated node addresses.</li>
        <li><strong>Stale Pruning:</strong> Automatically archives decommissioned nodes without losing custom user tags.</li>
      </ul>

      <h2 id="latency-testing">Latency & Ping Testing</h2>
      <p>
        Click <strong>Test All Pings</strong> to initiate concurrent TCP handshake and TLS handshake tests across all servers. Nodes are annotated with color-coded response times:
      </p>
      <ul>
        <li><strong style="color: #10b981;">&lt; 100ms (Green):</strong> High-performance nodes suitable for gaming and live calls.</li>
        <li><strong style="color: #f59e0b;">100ms - 250ms (Yellow):</strong> Standard international routes for video and streaming.</li>
        <li><strong style="color: #ef4444;">Timeout / Error (Red):</strong> Unreachable node or blocked port.</li>
      </ul>

      <h2 id="server-tagging">Favorites & Node Renaming</h2>
      <p>
        Star your favorite nodes to keep them pinned to the top of the menu bar popover for instant single-click connection switching.
      </p>
    </div>
  `
};

window.DOCS_PAGES["user-guide/routing-rules"] = {
  category: "User Guide",
  title: "Routing & Rules Engine",
  featureBadge: "Traffic Engine",
  lead: "Harness the power of intelligent traffic splitting. Direct domestic sites without latency penalties while tunneling blocked or international destinations.",
  heroImage: {
    src: "assets/images/routing.png",
    alt: "Routing Rules Configuration Screen",
    title: "Traffic Splitting & Routing Rules"
  },
  toc: [
    { id: "routing-philosophy", label: "Smart Routing Overview" },
    { id: "built-in-rules", label: "Built-in Rule Categories" },
    { id: "custom-rules", label: "Custom Domain & IP Rules" },
    { id: "dns-resolution-rules", label: "Route Precedence" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="routing-philosophy">Smart Routing Overview</h2>
      <p>
        Running all network traffic through an encrypted proxy introduces unnecessary latency for domestic services (banking apps, local streaming, local LAN printers). Kage VPN solves this through sing-box's declarative route rule engine.
      </p>

      <h2 id="built-in-rules">Built-in Rule Categories</h2>
      <div class="docs-table-wrapper">
        <table class="docs-table">
          <thead>
            <tr>
              <th>Rule Target</th>
              <th>Action</th>
              <th>Behavior</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><strong>Private / LAN Addresses</strong></td>
              <td><code>direct</code></td>
              <td>Bypasses proxy for <code>10.0.0.0/8</code>, <code>192.168.0.0/16</code>, and <code>*.local</code> mDNS.</td>
            </tr>
            <tr>
              <td><strong>Domestic GeoIP & GeoSite</strong></td>
              <td><code>direct</code></td>
              <td>Keeps local country web traffic on normal ISP route for lowest latency.</td>
            </tr>
            <tr>
              <td><strong>Blocked GeoSite (GFW / Censorship)</strong></td>
              <td><code>proxy</code></td>
              <td>Routes Google, YouTube, GitHub, Twitter, and OpenAI through the encrypted tunnel.</td>
            </tr>
            <tr>
              <td><strong>Ad & Tracker Blocking</strong></td>
              <td><code>block</code></td>
              <td>Silently drops telemetry and malicious tracker domains.</td>
            </tr>
          </tbody>
        </table>
      </div>

      <h2 id="custom-rules">Custom Domain & IP Rules</h2>
      <p>
        You can add specific override rules via the UI:
      </p>
      <ul>
        <li><strong>Domain Suffix:</strong> Match <code>.company.internal</code> &rarr; <code>direct</code>.</li>
        <li><strong>Domain Keyword:</strong> Match any domain containing <code>netflix</code> &rarr; <code>proxy</code>.</li>
        <li><strong>IP CIDR:</strong> Route specific subnet <code>1.2.3.0/24</code> &rarr; <code>direct</code>.</li>
      </ul>

      <h2 id="dns-resolution-rules">Route Precedence</h2>
      <p>
        Rules are evaluated from top to bottom. The first rule that matches a socket connection takes precedence. Unmatched traffic falls through to the <strong>Default Outbound</strong> specified on your active connection profile.
      </p>
    </div>
  `
};

window.DOCS_PAGES["user-guide/kill-switch"] = {
  category: "User Guide",
  title: "Kill Switch & Connection Watchdog",
  featureBadge: "Fail-Safe",
  lead: "Prevent accidental IP leaks when a server disconnects or your Wi-Fi renegotiates using macOS packet filter (pfctl) locks.",
  heroImage: {
    src: "assets/images/settings.png",
    alt: "Kill Switch Configuration in Settings",
    title: "Kill Switch & Security Controls"
  },
  toc: [
    { id: "threat-model", label: "The Leak Threat Model" },
    { id: "pfctl-anchor-mechanism", label: "macOS pfctl Anchor Mechanism" },
    { id: "watchdog-monitoring", label: "Process Watchdog Loop" },
    { id: "manual-recovery", label: "Emergency Network Recovery" }
  ],
  contentHtml: `
    <div class="callout-banner danger" id="threat-model">
      <div class="callout-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
      </div>
      <div>
        <strong>Why Standard Kill Switches Fail:</strong> When a VPN process crashes, macOS naturally falls back to the default physical network interface (en0). Sensitive applications can broadcast raw requests revealing your real residential IP before you realize the VPN stopped.
      </div>
    </div>

    <div class="docs-prose">
      <h2 id="pfctl-anchor-mechanism">macOS pfctl Anchor Mechanism</h2>
      <p>
        Kage VPN does not rely on fragile userspace sleep loops. Instead, it utilizes macOS's built-in FreeBSD packet filter (<code>pfctl</code>) by registering a dedicated firewall anchor:
      </p>
      <code>com.apple/250.KageVPN</code>
      <p>
        When the Kill Switch is activated:
      </p>
      <ul>
        <li>All outbound IPv4 and IPv6 packets from physical interfaces (<code>en0</code>, <code>en1</code>) are blocked.</li>
        <li>An explicit pass rule is preserved only for the IP and port of your active proxy server.</li>
        <li>Traffic exiting the virtual <code>utun</code> adapter is permitted.</li>
      </ul>
      <p>
        If the sing-box process terminates unexpectedly, the packet filter rules remain locked in the kernel, ensuring <strong>zero packets</strong> leave your machine in plaintext.
      </p>

      <h2 id="watchdog-monitoring">Process Watchdog Loop</h2>
      <p>
        The Rust backend runs an asynchronous tokio timer polling the sing-box child PID. If the core terminates unexpectedly, the watchdog immediately flags an alert in the menu bar and prevents reconnection loops until verified by the user.
      </p>

      <h2 id="manual-recovery">Emergency Network Recovery</h2>
      <p>
        If the app is force-killed or uninstalled while the Kill Switch anchor is active, you can restore full internet access with a single command:
      </p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="sudo pfctl -a &quot;com.apple/250.KageVPN&quot; -F all">Copy Command</button>
      </div>
      <pre><code>sudo pfctl -a "com.apple/250.KageVPN" -F all</code></pre>
    </div>

    <div class="docs-prose">
      <p>Alternatively, launch Kage VPN and click <strong>Settings &rarr; Restore Internet Access</strong>.</p>
    </div>
  `
};

window.DOCS_PAGES["user-guide/diagnostics-speedtest"] = {
  category: "User Guide",
  title: "Leak Diagnostics & Speed Testing",
  featureBadge: "Audit Tools",
  lead: "Verify the integrity of your encrypted tunnel with automated DNS leak, IPv6 leak, and WebRTC exposure tests, accompanied by wire-speed throughput checks.",
  heroImage: {
    src: "assets/images/diagnostics.png",
    alt: "Diagnostics Screen Showing Leak Test Results",
    title: "Diagnostics & Privacy Auditing"
  },
  toc: [
    { id: "leak-tests", label: "Automated Privacy Audits" },
    { id: "dns-leak-prevention", label: "DNS Leak Testing" },
    { id: "ipv6-webrtc-shield", label: "IPv6 & WebRTC Shield" },
    { id: "speed-benchmarks", label: "Speed & Throughput Benchmarks" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="leak-tests">Automated Privacy Audits</h2>
      <p>
        The Diagnostics tab provides an instant one-click security audit to ensure your identity remains protected while connected:
      </p>

      <h2 id="dns-leak-prevention">DNS Leak Testing</h2>
      <p>
        When you click <strong>Run Audit</strong>, Kage requests unique random cryptographic domain identifiers (e.g. <code>audit-4a9f.check.apzur.io</code>). It verifies which recursive resolver contacts the authoritative nameserver:
      </p>
      <ul>
        <li><strong style="color: #10b981;">Passed:</strong> Resolver matches the remote server IP or configured DoH server.</li>
        <li><strong style="color: #ef4444;">Failed:</strong> Resolver belongs to your local ISP (e.g. Comcast, Vodafone), indicating a DNS leak.</li>
      </ul>

      <h2 id="ipv6-webrtc-shield">IPv6 & WebRTC Shield</h2>
      <p>
        If your residential ISP issues an IPv6 block but your proxy server only routes IPv4, browsers can inadvertently leak your residential IPv6 address via dual-stack lookups or WebRTC STUN handshakes.
      </p>
      <p>
        Kage automatically disables unrouted IPv6 in TUN mode and drops rogue STUN queries to prevent browser-level deanonymization.
      </p>

      <h2 id="speed-benchmarks">Speed & Throughput Benchmarks</h2>
      <p>
        Measure realtime downstream and upstream bandwidth through the active tunnel. Kage performs multi-stream chunk downloads to test server link capacity without exposing your real IP.
      </p>
    </div>
  `
};

window.DOCS_PAGES["user-guide/data-usage"] = {
  category: "User Guide",
  title: "Persistent Data Usage & Quotas",
  featureBadge: "Telemetry",
  lead: "Monitor your bandwidth consumption across hours, days, and months. Store telemetry safely in a local SQLite database with write-ahead logging.",
  heroImage: {
    src: "assets/images/usage.png",
    alt: "Data Usage Analytics and Charts",
    title: "Data Usage & Quota Tracking"
  },
  toc: [
    { id: "usage-tracking", label: "Local Telemetry Engine" },
    { id: "sqlite-wal", label: "SQLite WAL Performance" },
    { id: "quota-warnings", label: "Data Cap Alerts" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="usage-tracking">Local Telemetry Engine</h2>
      <p>
        Kage VPN periodically samples upload and download byte counters from sing-box's Clash API memory buffer. Unlike applications that clear statistics upon exit, Kage writes counters to an embedded local SQLite database located at:
      </p>
      <code>~/Library/Application Support/com.apzur.kage/data/usage.db</code>

      <h2 id="sqlite-wal">SQLite Write-Ahead Logging (WAL)</h2>
      <p>
        To prevent battery drain and disk thrashing on Mac laptops, the SQLite engine is configured with:
      </p>
      <ul>
        <li><code>PRAGMA journal_mode = WAL;</code> — Allows concurrent read operations while writes happen asynchronously.</li>
        <li><code>PRAGMA synchronous = NORMAL;</code> — Minimizes disk sync operations while guaranteeing atomic consistency.</li>
        <li><code>PRAGMA cache_size = -2000;</code> — Constrains memory footprint to just 2 MB RAM.</li>
      </ul>

      <h2 id="quota-warnings">Data Cap Alerts</h2>
      <p>
        If your node provider or mobile hotspot imposes monthly limits (e.g., 200 GB / month), set a quota threshold in <strong>Settings &rarr; Usage Limits</strong>. Kage alerts you at 80% and 95% consumption.
      </p>
    </div>
  `
};
