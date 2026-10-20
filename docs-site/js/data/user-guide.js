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
        <li><strong>Your plan:</strong> when the provider reports it, the subscription shows how much data is used and when it ends, and the menu bar popover shows what is left for the server you use. You get one notification at 90% of the data and one when it runs out. For the end date there are notifications 7 days, 3 days and 1 day before it, and one when the plan ends; each is sent once, and the end date is checked every half hour even if the subscription itself cannot be refreshed. The plan line in the popover turns to the warning colour at 10% of the data or 7 days left.</li>
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
    { id: "routing-modes", label: "Choosing a Mode" },
    { id: "built-in-rules", label: "Ready-made Lists" },
    { id: "list-sources", label: "List Sources & Downloads" },
    { id: "custom-rules", label: "Your Own Rules" },
    { id: "app-rules", label: "Choosing by App" },
    { id: "app-caps", label: "Data Caps for Apps" },
    { id: "dns-resolution-rules", label: "Route Precedence" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="routing-philosophy">Smart Routing Overview</h2>
      <p>
        Running all network traffic through an encrypted proxy introduces unnecessary latency for domestic services (banking apps, local streaming, local LAN printers). Kage VPN solves this through sing-box's declarative route rule engine.
      </p>

      <h2 id="routing-modes">Choosing a Mode</h2>
      <p>
        <strong>Routing</strong> opens with three cards: <strong>Everything through the VPN</strong> (the simplest choice), <strong>Choose by rules</strong>, and <strong>VPN idle</strong> (the VPN stays on but nothing uses it, which is handy for testing). In rules mode, an <strong>Everything else</strong> card decides what happens to traffic that no rule or list matches: <em>Use the VPN</em> or <em>Go direct</em>. Below it, three tabs hold <strong>Your rules</strong>, <strong>Ready-made lists</strong> and <strong>Try a website</strong>. Choices for whole apps and data caps are on their own <strong>Apps</strong> page in the sidebar. <strong>Network</strong>, at the bottom, has the local-network and DNS settings that apply in every mode. Changes apply when you press <strong>Save changes</strong> and connect again.
      </p>

      <h2 id="built-in-rules">Ready-made Lists</h2>
      <p>
        Instead of typing addresses, switch on lists that are kept up to date for you. Open <strong>Ready-made lists</strong> and press <strong>Add a list</strong>:
      </p>
      <ul>
        <li><strong>Countries:</strong> every network address registered in a country (for example Sri Lanka, China or Russia). <em>Go direct</em> is the usual choice for your own country.</li>
        <li><strong>Websites &amp; services:</strong> advertising and trackers, local and private networks, Google, YouTube, Apple, Microsoft, GitHub, OpenAI, Anthropic, Netflix, Disney+, Spotify, Telegram, WhatsApp, Discord, X, Facebook, TikTok, Steam, PayPal, Amazon and more. Each one suggests a sensible action, such as <em>Block</em> for advertising.</li>
        <li><strong>Other&hellip;</strong> accepts any list name the chosen source publishes.</li>
      </ul>
      <p>
        Each list shows whether it is <strong>Ready</strong> or <strong>Not downloaded</strong>. A list that could not be downloaded is left out and the connection still starts, so a blocked download never stops you from connecting.
      </p>

      <h2 id="list-sources">List Sources & Downloads</h2>
      <p>
        Lists are downloaded by the app and refreshed weekly. Open <strong>Downloads and sources</strong> under the lists to manage them:
      </p>
      <ul>
        <li><strong>Where lists come from:</strong> choose <em>SagerNet</em> (the default, the sing-box authors' own lists), <em>MetaCubeX</em>, <em>v2rayN</em>, <em>Loyalsoldier (IP lists)</em> or <em>runetfreedom (Russia)</em>. If one is slow or blocked where you are, pick another. Choosing a different source and saving downloads your lists again from it.</li>
        <li><strong>Add your own source:</strong> give a name and an address that contains <code>{name}</code> where the list name goes, for example <code>https://example.com/geoip-{name}.srs</code>. Site lists are optional. The files must be sing-box rule lists (<code>.srs</code>); V2Ray <code>.dat</code> files cannot be used by sing-box.</li>
        <li><strong>Downloaded lists:</strong> see each list's size and age, and download, update or delete one list or all of them. These buttons use your saved settings, so save first.</li>
      </ul>
      <p>
        <strong>Blocked networks.</strong> Kage tries GitHub, then a jsDelivr copy of the same file. Once you are connected, downloads go through the VPN first, then directly, so a network that blocks GitHub no longer matters. A list that could not be fetched before connecting is fetched through the tunnel as soon as it is verified, and you get a notification to reconnect to apply it.
      </p>

      <h2 id="custom-rules">Your Own Rules</h2>
      <p>
        Open <strong>Your rules</strong> and use <strong>Add a rule</strong>: choose what to match, type the address, and choose <em>Use the VPN</em>, <em>Go direct</em> or <em>Block</em>. Rules are numbered and can be moved up or down. New users also see ready-made sets (streaming direct, AI through the VPN, block ads). <strong>Try a website</strong> tells you what your rules would do with an address. The options are:
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

      <h2 id="app-rules">Choosing by App</h2>
      <p>
        The <strong>Apps</strong> page (in the sidebar, right after Routing) lists the apps installed on your Mac. Next to each one, choose <em>VPN</em>, <em>Direct</em> or <em>Block</em>, or press the reset arrow to follow the default again. Two buttons at the top set that default: <em>VPN for everything except&hellip;</em> (all traffic uses the VPN, and you pick the exceptions) or <em>VPN only for&hellip;</em> (nothing uses the VPN until you pick an app). Blocked apps are listed above the search box with an <em>Unblock</em> button, and each row shows how much data the app used through the VPN this billing period when app tracking is on. Programs without an app icon, and rules for apps you have since removed, appear at the bottom and are edited under <strong>Your rules</strong>. This app and its engine cannot be blocked, because the VPN itself needs the network. Changes apply when you press <strong>Save changes</strong>.
      </p>

      <h2 id="app-caps">Data Caps for Apps</h2>
      <p>
        Open <strong>Apps</strong> in the sidebar and press <strong>Cap</strong> next to an app to give it its own data limit. Choose the size (in MB or GB), how often it starts over (<em>Day</em>, <em>Week</em> or <em>Month</em>, always from midnight, from Monday for weeks and the 1st for months) and what happens at the limit: <em>Tell me</em> or <em>Offer to block</em>. You get a notification at 80% and again at 100%.
      </p>
      <p>
        With <em>Offer to block</em>, a card on Home and the app's row show <strong>Block now</strong> and <strong>Not now</strong> at 100%. Block now stops the app using the network until the period ends and reconnects the VPN once; it is lifted by itself when the period starts over (with a notification), or any time with <strong>Unblock</strong>. Nothing is blocked unless you press Block now.
      </p>
      <p>
        Caps count only what the app sends through the VPN, taken from the connection list every few seconds, so figures are approximate: very short connections can be missed. Counting needs <em>app usage</em> to be on (Settings, Usage); the cap editor offers to turn it on. Clearing your usage history does not change a cap. Kage VPN and its engine cannot be capped.
      </p>

      <h2 id="dns-resolution-rules">Route Precedence</h2>
      <p>
        Your rules are evaluated from top to bottom and the first one that matches decides. Ready-made lists come after your rules. Anything still unmatched follows <strong>Everything else</strong>.
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
    { id: "reconnect-block", label: "While Reconnecting" },
    { id: "unblock", label: "Unblock Internet" },
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
        Kage VPN uses macOS's built-in packet filter (<code>pfctl</code>) through a dedicated anchor, <code>com.apple/250.KageVPN</code>. The kill switch is <strong>experimental</strong>, works in TUN mode only, and is off by default. The firewall rules are managed by the root helper that runs sing-box, so they stay in place even if the app itself crashes. While a block is up, the helper allows only:
      </p>
      <ul>
        <li>Loopback traffic and local-network ranges (<code>10.0.0.0/8</code>, <code>172.16.0.0/12</code>, <code>192.168.0.0/16</code>, link-local and multicast).</li>
        <li>IPv6 router and neighbour discovery and link-local traffic, so IPv6 networks keep working.</li>
        <li>The addresses of your server. Kage looks the server name up again on every connect and every five minutes while connected, and remembers the last three answers, so a server whose name starts resolving elsewhere is already allowed.</li>
      </ul>
      <p>
        Everything else is dropped. The block goes up when sing-box exits unexpectedly and when the tunnel is up but has stopped carrying traffic.
      </p>

      <h2 id="reconnect-block">While Reconnecting</h2>
      <p>
        The block is not released between reconnect attempts. A failed attempt, or a restart of a dead tunnel, keeps it up until the new tunnel has been verified (a real request went through it). A failed <em>first</em> connect, for example a wrong password or a bad server, does not block anything.
      </p>
      <ul>
        <li><strong>DNS:</strong> for up to 20 seconds while a reconnect starts, DNS lookups (port 53) are allowed so sing-box can find your server's name. During those seconds any app's domain-name lookups can reach a resolver; traffic itself stays blocked.</li>
        <li><strong>Safety valve:</strong> a block that the app never releases is dropped after 10 minutes.</li>
        <li><strong>Offline:</strong> while blocked, Kage does not try to download lists or look up names. It checks your server using its remembered addresses and resumes the reconnect when the server answers.</li>
      </ul>

      <h2 id="unblock">Unblock Internet</h2>
      <p>
        While a block is up, Home shows <strong>Internet blocked by the kill switch</strong> with an <strong>Unblock internet</strong> button. It needs no password: it stops the reconnect in progress and lets traffic out. If the app crashed and your internet stays blocked, use <strong>Settings &rarr; Restore internet access</strong> (asks for your password) or the command under Emergency Network Recovery.
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
