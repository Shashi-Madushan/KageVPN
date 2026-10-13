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
        <li>Kage launches sing-box with a mixed SOCKS5/HTTP inbound listener on <code>127.0.0.1:2080</code>. If another program already uses that port, Kage picks the next free one and shows the address on the Home page.</li>
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
          <p>Drag any <code>.txt</code> file of links onto the <strong>Add server</strong> box to import many servers at once. Clash and Clash Meta configuration files (<code>.yaml</code>, the <code>proxies</code> list) and sing-box configuration files (<code>.json</code>, the <code>outbounds</code> list) work too: every server Kage can run is added, and the ones it cannot (such as TUIC, WireGuard or servers chained through another server) are listed with the reason.</p>
        </div>
      </div>
    </div>

    <div class="docs-prose">
      <h2 id="subscriptions">Managing Subscriptions</h2>
      <p>
        If your provider supplies a subscription URL, click <strong>Add subscription</strong>. Kage reads plain or Base64 lists of share links, Clash YAML and sing-box JSON. Servers keep the names, groups and favorites you gave them across refreshes, and servers the provider dropped are removed.
      </p>
      <ul>
        <li><strong>Refresh:</strong> subscriptions refresh every 12 hours on their own; <strong>Refresh</strong> does it now. A subscription that keeps failing is retried less and less often.</li>
        <li><strong>Your plan:</strong> when the provider reports it, the subscription shows how much data is used and when it ends, and the menu bar popover shows what is left for the server you use. You get one notification at 90% of the data, one when it runs out, and one three days before the end date.</li>
        <li><strong>Edit:</strong> change the name or the link (for example after your provider gives you a new one), and the options below. Saving refreshes the subscription.</li>
        <li><strong>Ask as:</strong> some providers send a different list, or none, depending on the app asking. If a subscription comes back empty, try <em>v2rayN</em>, <em>sing-box</em> or <em>Clash Meta</em>, or type the client name your provider expects.</li>
        <li><strong>Send device ID:</strong> some providers limit how many devices may use one subscription and refuse apps that do not identify the device. Turn this on for them. Kage sends a random ID made for this Mac (not a hardware serial), the macOS version and the Mac model, and only to that subscription's address. If the provider says its device limit is reached, remove a device you no longer use in the provider's panel.</li>
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
      <p>
        After a test, <strong>Remove unreachable</strong> deletes every server that did not answer, after asking. The server you are connected through is never included. Servers that came from a subscription return at its next refresh if the provider still lists them. A network that blocks your servers makes them all look dead, so run the test on your usual network first.
      </p>

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
        <li><strong>App:</strong> pick an installed app, such as Zoom &rarr; <code>direct</code> or a game launcher &rarr; <code>proxy</code>. The rule covers every process inside the app, including the helper processes most browsers and chat apps send their traffic from. Safari and apps built on its engine send traffic from a shared system process, so they cannot be routed by app; use domain rules for them.</li>
        <li><strong>Process name:</strong> an executable outside an app, such as <code>curl</code> or <code>ssh</code>.</li>
      </ul>
      <p>
        You can also add an app rule from <strong>Connections &rarr; Apps</strong>: click an app and choose <strong>Through VPN</strong>, <strong>Direct</strong> or <strong>Block</strong>. If routing was in Global or Direct mode, it switches to Rules mode with the same default for everything else, so only that app changes. The VPN reconnects to apply it.
      </p>

      <h2 id="dns-resolution-rules">Route Precedence</h2>
      <p>
        Rules are evaluated from top to bottom. The first rule that matches a socket connection takes precedence. Unmatched traffic falls through to the <strong>Default Outbound</strong> specified on your active connection profile.
      </p>
    </div>
  `
};

window.DOCS_PAGES["user-guide/kill-switch"] = {
  category: "User Guide",
  title: "Kill Switch & Connection Health",
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
    { id: "connection-health", label: "Connection Health Monitor" },
    { id: "crash-log", label: "When the VPN Stops Unexpectedly" },
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
        The Rust backend polls the sing-box child PID. If the core terminates unexpectedly, the app flags it in the menu bar and starts the usual reconnect attempts.
      </p>
      <p>
        Separately, a health monitor checks that traffic really flows through the tunnel: once right after connecting, then with one small request every 15 seconds. When checks fail, the app shows "Connected, but no internet" with the reason, and backs off to at most one check a minute. Turn on <strong>Restart a dead tunnel</strong> in Settings to have it restart automatically. It also re-checks right away when your Mac wakes or changes network.
      </p>

      <h2 id="connection-health">Connection Health Monitor</h2>
      <p>
        "Connected" does not always mean "working": sing-box opens its local port even when the server is down. Kage therefore only shows <strong>You're protected</strong> after a real request has gone through the tunnel. While it checks, you see <strong>Checking your connection…</strong>.
      </p>
      <ul>
        <li><strong>Protected (green):</strong> traffic flows. The latency pill shows the measured round trip.</li>
        <li><strong>Connected, but no internet (red):</strong> the tunnel is up but traffic does not pass. The reason is one of <em>server not answering</em> (offline, blocked, wrong address or port), <em>server rejected the connection</em> (credentials, TLS or protocol settings) or <em>no traffic passes</em> (the server answers but DNS or routing is broken).</li>
      </ul>
      <p>
        A red state offers <strong>Try another server</strong>, <strong>Reconnect</strong>, <strong>Diagnostics</strong> and <strong>Disconnect</strong>. The menu bar icon switches to a warning icon and the menu shows the status on its first line. When traffic returns, the status turns green on its own.
      </p>
      <p>
        Before connecting, Kage also checks that the server name resolves and (for TCP protocols) accepts a connection, so an unusable server fails in seconds with a clear message. Hysteria2 uses UDP and skips the TCP check.
      </p>

      <h2 id="crash-log">When the VPN Stops Unexpectedly</h2>
      <p>
        If sing-box exits on its own, or the tunnel stops answering while sing-box is still running, Kage shows a notification with the reason (for example the exit status and the last error), a red banner on the home screen, and writes an entry to <code>crashes.log</code> in the data folder. Open it in the app from the banner's <strong>View crash log</strong> button or the <strong>Crash log</strong> button on the Logs page. It has Copy and Clear buttons.
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
  lead: "See what the internet sees through the tunnel and through your Mac's normal route, which DNS resolver answers, and how fast the tunnel is.",
  heroImage: {
    src: "assets/images/diagnostics.png",
    alt: "Diagnostics Screen Showing Leak Test Results",
    title: "Diagnostics & Privacy Auditing"
  },
  toc: [
    { id: "leak-tests", label: "Automated Privacy Audits" },
    { id: "dns-leak-prevention", label: "DNS Leak Testing" },
    { id: "ipv6-webrtc-shield", label: "IPv6 and WebRTC" },
    { id: "speed-benchmarks", label: "Speed & Throughput Benchmarks" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="leak-tests">Automated Privacy Audits</h2>
      <p>
        The Diagnostics tab runs a one-click check that shows three things side by side while you are connected:
      </p>
      <ul>
        <li><strong>Tunnel:</strong> the IP address, place and network the internet sees for traffic sent through the VPN.</li>
        <li><strong>System route:</strong> the same, for a request made the way any other app on your Mac makes it. In Full System (TUN) mode both should match; if they differ, the page says <em>Possible leak</em>.</li>
        <li><strong>DNS resolver:</strong> the address and place of the DNS resolver that answered.</li>
      </ul>

      <h2 id="dns-leak-prevention">DNS Leak Testing</h2>
      <p>
        Kage asks a public service (<code>edns.ip-api.com</code>) which DNS resolver reached it and shows the answer. The page does not mark it as passed or failed, so compare it with what you expect: with a DNS server chosen in Routing it should be that provider or the VPN server's network, and not your ISP.
      </p>

      <h2 id="ipv6-webrtc-shield">IPv6 and WebRTC</h2>
      <p>
        In Full System (TUN) mode the tunnel carries both IPv4 and IPv6, so traffic to IPv6 addresses goes through the VPN too instead of leaving through your Mac's normal connection. If your VPN server has no IPv6, apps fall back to IPv4 by themselves.
      </p>
      <p>
        Kage does not test for WebRTC leaks. Because the tunnel carries all traffic, the public address a browser's WebRTC reports is the tunnel's, but to be sure, open a WebRTC test page in your browser while connected.
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
  title: "Data Usage & Limits",
  featureBadge: "Telemetry",
  lead: "See in plain words how much data you used, which apps and servers used it, and stay under your monthly limit. Everything is stored locally on your Mac.",
  heroImage: {
    src: "assets/images/usage.png",
    alt: "Data Usage page",
    title: "Data Usage"
  },
  toc: [
    { id: "reading-the-page", label: "Reading the Page" },
    { id: "where-did-it-go", label: "Where Did It Go?" },
    { id: "limits-privacy", label: "Limits & Privacy (Settings)" },
    { id: "usage-tracking", label: "Local Telemetry Engine" },
    { id: "sqlite-wal", label: "SQLite WAL Performance" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="reading-the-page">Reading the Page</h2>
      <p>
        The page opens on <strong>This month</strong>. Use the buttons in the header to switch to <strong>Today</strong>, <strong>7 days</strong> or <strong>Last month</strong>; <strong>More</strong> adds Yesterday, Last 30 days, your billing cycle and custom dates.
      </p>
      <ul>
        <li><strong>You used…</strong> shows the total, how it compares with the previous period, and the split between downloaded and uploaded data.</li>
        <li><strong>Per day, busiest day and time connected</strong> give you the shape of your usage at a glance.</li>
        <li><strong>Monthly limit</strong> appears when you set one, with a bar showing how much is left.</li>
        <li><strong>Day by day</strong> is a bar chart. Hover a bar for its total; click it to zoom in (a "Back to this month" link brings you back). <em>Chart options</em> lets you group by hour, day, week or month and compare with the previous period.</li>
      </ul>

      <h2 id="where-did-it-go">Where Did It Go?</h2>
      <p>
        Switch between <strong>Apps</strong>, <strong>Websites</strong> and <strong>Servers</strong> to see what used the data, with each one's share and its download and upload split. Click a row to show only that item; a "Showing only…" bar appears with a <strong>Show everything</strong> link. A <em>Filter by server or connection type</em> section at the bottom narrows the whole page further.
      </p>

      <h2 id="limits-privacy">Limits & Privacy (Settings)</h2>
      <p>
        The data limit and tracking options live in <strong>Settings</strong>, under <em>Data usage</em> and <em>Usage privacy</em>:
      </p>
      <ul>
        <li><strong>Monthly data limit</strong>, the day the month starts, and what happens when the limit is reached (notify, or notify and disconnect).</li>
        <li><strong>Track apps</strong> and <strong>Track websites</strong> (off by default), <strong>Pause counting</strong>, and how long app and website history is kept.</li>
        <li><strong>Clear app and website history</strong> and <strong>Reset all usage</strong> (asks you to confirm).</li>
      </ul>
      <p>
        You are alerted as you approach the limit. Per-app and per-website figures are estimates.
      </p>

      <h2 id="usage-tracking">Local Telemetry Engine</h2>
      <p>
        Kage VPN periodically samples upload and download byte counters from sing-box's Clash API memory buffer. Unlike applications that clear statistics upon exit, Kage writes counters to an embedded local SQLite database located at:
      </p>
      <code>~/Library/Application Support/com.apzur.kage/usage.db</code>

      <h2 id="sqlite-wal">SQLite Write-Ahead Logging (WAL)</h2>
      <p>
        To prevent battery drain and disk thrashing on Mac laptops, the SQLite engine is configured with:
      </p>
      <ul>
        <li><code>PRAGMA journal_mode = WAL;</code> — Allows concurrent read operations while writes happen asynchronously.</li>
        <li><code>PRAGMA synchronous = NORMAL;</code> — Minimizes disk sync operations while guaranteeing atomic consistency.</li>
        <li><code>PRAGMA cache_size = -2000;</code> — Constrains memory footprint to just 2 MB RAM.</li>
      </ul>
    </div>
  `
};
