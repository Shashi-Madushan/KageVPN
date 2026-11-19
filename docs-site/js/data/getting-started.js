/**
 * Kage VPN Documentation — Getting Started Pages
 */
window.DOCS_PAGES = window.DOCS_PAGES || {};

window.DOCS_PAGES["getting-started/overview"] = {
  category: "Getting Started",
  title: "Introduction & Overview",
  featureBadge: "v0.2.0 Native",
  lead: "Kage VPN is a client for the sing-box proxy engine on macOS and Windows. It puts VLESS and other proxy protocols behind one <span class='os-mac'>menu bar</span><span class='os-win'>system tray</span> app, built with Tauri 2 and Svelte 5. Use the <strong>macOS / Windows</strong> switch at the top to see the steps for your computer.",
  heroImage: {
    src: "assets/images/home.png",
    alt: "Kage VPN Dashboard Connected State",
    title: "Kage VPN — Main Control Dashboard"
  },
  toc: [
    { id: "what-is-apzur", label: "What is Kage VPN?" },
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
        <strong>System Requirement:</strong> <span class="os-mac">macOS 12 Monterey or later (Apple Silicon M1/M2/M3/M4 and Intel x86_64). One universal app runs natively on both, without Rosetta.</span><span class="os-win">Windows 10 (22H2) or 11, x64. <strong>Windows is a preview:</strong> Proxy Only mode and the Windows system proxy work today; TUN mode and the kill switch come in later releases. See <a href="#/getting-started/windows-install">Install on Windows</a>.</span>
      </div>
    </div>

    <div class="docs-prose">
      <h2>What is Kage VPN?</h2>
      <p>
        Most commercial VPN applications force users into closed subscription networks with proprietary protocols and heavy resource usage. Kage VPN takes a different path: you bring your own server credentials or subscription feed, and Kage provides a high-performance, open-source native client for <span class="os-mac">macOS</span><span class="os-win">Windows</span> that manages the <code>sing-box</code> engine.
      </p>
      <p>
        Whether you are bypassing network firewalls, securing remote public Wi-Fi, or orchestrating multi-protocol server fleets, Kage gives you absolute visibility over your routing, active sockets, and encrypted traffic.
      </p>

      <h2 id="key-capabilities">Key Capabilities</h2>
      <div class="docs-table-wrapper">
        <table class="docs-table">
          <thead>
            <tr>
              <th>Feature</th>
              <th>Kage Implementation</th>
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
              <td><span class="os-mac">Virtual TUN (<code>utun</code>) and Local Proxy Only</span><span class="os-win">Local Proxy Only with the Windows system proxy (TUN mode is coming)</span></td>
              <td>Choose between 100% whole-system encryption or selective proxying.</td>
            </tr>
            <tr class="os-mac">
              <td><strong>macOS pfctl Kill Switch</strong> (experimental)</td>
              <td>Kernel packet filter anchor integration</td>
              <td>Blocks other traffic while the tunnel is down or reconnecting. Tested against a stand-in firewall so far; try it on your Mac before relying on it.</td>
            </tr>
            <tr>
              <td><strong>Routing Rules</strong></td>
              <td>Sites, IP ranges, apps, and ready-made country and service lists</td>
              <td>Send only what you choose through the VPN, let the rest go direct, or block it.</td>
            </tr>
            <tr>
              <td><strong>App Controls &amp; Data Caps</strong></td>
              <td>VPN, Direct or Block per installed app; daily, weekly or monthly limits</td>
              <td>Split tunneling by app without typing executable names, and a warning (or a block, if you choose) before an app eats your data plan.</td>
            </tr>
            <tr>
              <td><strong>Memory Footprint</strong></td>
              <td>Tauri 2 (Rust) + Svelte 5 (~45 MB RAM)</td>
              <td>Runs quietly in the <span class="os-mac">menu bar without draining Apple Silicon battery.</span><span class="os-win">system tray without weighing on your PC.</span></td>
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
          <div class="step-title"><span class="os-mac">Download & Install DMG</span><span class="os-win">Download & Run the Installer</span></div>
          <span class="step-badge-tag">Install</span>
        </div>
        <div class="step-body">
          <p class="os-mac">Download the latest release disk image from GitHub. Drag <code>Kage VPN.app</code> into your macOS <code>/Applications</code> directory.</p>
          <p class="os-win">Download <code>kage-vpn_&lt;version&gt;_windows-x86_64.setup.exe</code> from GitHub Releases and run it. It installs for your account only, so it needs no administrator rights.</p>
        </div>
      </div>

      <div class="step-item done">
        <div class="step-number-badge">2</div>
        <div class="step-head">
          <div class="step-title"><span class="os-mac">Bypass macOS Gatekeeper</span><span class="os-win">Get Past SmartScreen</span></div>
          <span class="step-badge-tag">One-time Action</span>
        </div>
        <div class="step-body">
          <p class="os-mac">Because Kage is distributed directly via GitHub Releases, macOS Gatekeeper may show a warning. Right-click and choose <strong>Open</strong> or approve in <strong>System Settings &rarr; Privacy & Security</strong>.</p>
          <p class="os-win">The installer is not code-signed yet, so Microsoft Defender SmartScreen may say it protected your PC. Click <strong>More info</strong>, then <strong>Run anyway</strong>. Details are in <a href="#/getting-started/windows-install">Install on Windows</a>.</p>
        </div>
      </div>

      <div class="step-item">
        <div class="step-number-badge">3</div>
        <div class="step-head">
          <div class="step-title">Import Server or Subscription</div>
          <span class="step-badge-tag">One Click</span>
        </div>
        <div class="step-body">
          <p>Paste a raw link (e.g. <code>vless://...</code>, <code>hysteria2://...</code>) or paste a provider subscription URL. Kage automatically parses endpoints, latency ping, and security certificates.</p>
        </div>
      </div>

      <div class="step-item">
        <div class="step-number-badge">4</div>
        <div class="step-head">
          <div class="step-title">Connect & Verify</div>
          <span class="step-badge-tag">Ready</span>
        </div>
        <div class="step-body">
          <p>Click the Kage mascot in the dashboard or <span class="os-mac">menu bar</span><span class="os-win">system tray</span> popover. It wakes up and smiles once your connection is verified. <span class="os-mac">Enter your macOS administrator password once if using TUN mode to authorize network interface creation.</span><span class="os-win">On Windows Kage connects in Proxy Only mode; turn on <strong>Settings &rarr; Set system proxy</strong> so browsers and most apps use it without setup.</span></p>
        </div>
      </div>
    </div>

    <div class="quote-callout-card" id="architecture-at-a-glance">
      <div class="quote-callout-text">
        "Kage VPN is built for developers and privacy advocates who want complete control over their proxy tunnels without wrestling with raw configuration files or sluggish Electron apps."
      </div>
      <div class="quote-callout-author">— Shashi Madushan, Maintainer</div>
    </div>
  `
};

window.DOCS_PAGES["getting-started/gatekeeper"] = {
  category: "Getting Started",
  title: "macOS Gatekeeper & Security Fix",
  featureBadge: "Troubleshooting",
  lead: "How to resolve 'Kage VPN cannot be opened because Apple cannot verify malware' or 'damaged and can't be opened' messages on macOS Sequoia, Sonoma, and earlier.",
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
        <strong>Unsigned Application:</strong> Kage VPN is an independent open-source project and is not notarized through a paid Apple Developer subscription. macOS automatically applies an extended quarantine attribute (<code>com.apple.quarantine</code>) to downloaded binaries.
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
          <p>Double-click <code>/Applications/Kage VPN.app</code>. When macOS shows the security warning, click <strong>Done</strong> (or Cancel).</p>
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
          <p>You will see the notice: <em>"Kage VPN was blocked to protect your Mac."</em> Click <strong>Open Anyway</strong>, confirm with your Touch ID or password, and click <strong>Open</strong> in the subsequent prompt.</p>
        </div>
      </div>
    </div>

    <div class="docs-prose">
      <h2 id="option-b">Option B: Remove Quarantine via Terminal (Permanent Fix)</h2>
      <p>If macOS displays <em>"Kage VPN is damaged and can't be opened"</em>, stripping the quarantine attribute immediately resolves the issue:</p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="xattr -dr com.apple.quarantine &quot;/Applications/Kage VPN.app&quot;">Copy Command</button>
      </div>
      <pre><code>xattr -dr com.apple.quarantine "/Applications/Kage VPN.app"</code></pre>
    </div>

    <div class="docs-prose">
      <p>Once executed, open Kage VPN normally from Launchpad or Finder. You will never be prompted with quarantine warnings for that installation again.</p>

      <h2 id="verifying-binary">Verifying Binary Integrity with Minisign</h2>
      <p>Every official release is accompanied by a SHA-256 checksum and Minisign cryptographic signature. To verify that your download has not been tampered with:</p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="minisign -Vm &quot;Kage-VPN.dmg&quot; -P RWTw9...YourPublicKey">Verify Signature</button>
      </div>
      <pre><code>minisign -Vm "Kage-VPN.dmg" -P RWTw9...OfficialPublicKey</code></pre>
    </div>
  `
};

window.DOCS_PAGES["getting-started/windows-install"] = {
  category: "Getting Started",
  title: "Install on Windows",
  featureBadge: "Windows preview",
  lead: "Install Kage on Windows 10 or 11, get past the SmartScreen warning, and see what the Windows preview can do today.",
  toc: [
    { id: "win-status", label: "What works on Windows" },
    { id: "win-install", label: "Install" },
    { id: "win-smartscreen", label: "SmartScreen warning" },
    { id: "win-first-connect", label: "First connection" },
    { id: "win-uninstall", label: "Uninstall" }
  ],
  contentHtml: `
    <div class="callout-banner warn">
      <div><strong>Preview.</strong> Windows support arrives in phases. This release covers Proxy Only mode with the Windows system proxy. TUN mode (every app through the VPN) and the kill switch follow in later releases.</div>
    </div>

    <div class="docs-prose">
      <h2 id="win-status">What works on Windows</h2>
      <div class="docs-table-wrapper">
        <table class="docs-table">
          <thead><tr><th>Feature</th><th>On Windows</th></tr></thead>
          <tbody>
            <tr><td>Servers, subscriptions, QR import, routing rules</td><td>Yes, the same as on macOS</td></tr>
            <tr><td>Proxy Only mode</td><td>Yes. sing-box runs hidden and stops if the app quits or crashes</td></tr>
            <tr><td>Set system proxy</td><td>Yes. Edge, Chrome and most apps follow it; your previous settings come back on disconnect, and also after a crash</td></tr>
            <tr><td>App controls</td><td>Yes, for apps that follow the system proxy. Installed apps are read from the list in Settings &rarr; Apps</td></tr>
            <tr><td>Engine manager</td><td>Yes (Windows x64 builds of sing-box)</td></tr>
            <tr><td>Tray popover, notifications, launch at login, <code>kagevpn://</code> links</td><td>Yes</td></tr>
            <tr><td>Full System (TUN) mode</td><td>Not yet: needs a helper service, planned next</td></tr>
            <tr><td>Kill switch</td><td>Not yet</td></tr>
            <tr><td>Live speed next to the tray icon</td><td>No: Windows tray icons are square</td></tr>
          </tbody>
        </table>
      </div>

      <h2 id="win-install">Install</h2>
      <ol>
        <li>Download <code>kage-vpn_&lt;version&gt;_windows-x86_64.setup.exe</code> from <a href="https://github.com/Shashi-Madushan/KageVPN/releases" target="_blank" rel="noopener noreferrer">GitHub Releases</a>.</li>
        <li>Run it. It installs for your account (under <code>%LOCALAPPDATA%</code>), so no administrator password is needed.</li>
        <li>Kage starts and puts its icon in the system tray. On Windows 11 the icon may be hidden under the <strong>^</strong> arrow by the clock: drag it onto the taskbar to keep it visible.</li>
      </ol>
      <p>Windows 10 needs the Microsoft Edge WebView2 Runtime, which is already present on most PCs. If the window stays blank, install the <em>Evergreen</em> runtime from Microsoft.</p>

      <h2 id="win-smartscreen">SmartScreen warning</h2>
      <p>Kage is not code-signed yet, so Microsoft Defender SmartScreen may show <em>"Windows protected your PC"</em>. Click <strong>More info</strong>, check that the app is <code>kage-vpn_&lt;version&gt;_windows-x86_64.setup.exe</code>, then click <strong>Run anyway</strong>. Updates installed from inside the app are checked against Kage's own signing key, so they do not need this step.</p>

      <h2 id="win-first-connect">First connection</h2>
      <ol>
        <li>The setup guide downloads the sing-box engine and asks for your first server or subscription.</li>
        <li>Turn on <strong>Settings &rarr; Set system proxy</strong>, so browsers and other apps that follow the Windows proxy settings use the VPN.</li>
        <li>Click the mascot to connect. <em>Settings &rarr; Network &amp; internet &rarr; Proxy</em> now shows <code>127.0.0.1</code> and Kage's port; it goes back to what it was when you disconnect.</li>
        <li>Apps that ignore the system proxy (many games, some command-line tools) can be pointed at <code>127.0.0.1:2080</code> by hand, or wait for TUN mode.</li>
      </ol>

      <h2 id="win-uninstall">Uninstall</h2>
      <p>Quit Kage from the tray menu, then remove it in <strong>Settings &rarr; Apps &rarr; Installed apps</strong>. Your servers and settings stay in <code>%APPDATA%\\com.apzur.kage</code>; delete that folder to remove them too.</p>
    </div>
  `
};

window.DOCS_PAGES["getting-started/first-run"] = {
  category: "Getting Started",
  title: "First Run & Onboarding Tour",
  featureBadge: "Workflow",
  lead: "Take a tour through Kage VPN's dual-interface architecture: the swift <span class='os-mac'>menu bar</span><span class='os-win'>system tray</span> popover for day-to-day operations and the comprehensive dashboard for granular controls.",
  heroImage: {
    src: "assets/images/menu-bar-popover.png",
    alt: "Kage Menu Bar Popover",
    title: "Menu Bar Quick Controller"
  },
  toc: [
    { id: "interface-layout", label: "Dual Interface Architecture" },
    { id: "menu-bar-controller", label: "Quick Popover" },
    { id: "setup-guide", label: "The Setup Guide" },
    { id: "main-dashboard", label: "Full Dashboard Window" },
    { id: "permission-setup", label: "Initial Permission Setup" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="interface-layout">Dual Interface Architecture</h2>
      <p>
        <span class="os-mac">Kage VPN is intentionally designed around macOS Human Interface Guidelines. Rather than forcing a persistent desktop window, the app runs as a lightweight menu bar resident with on-demand expansion.</span>
        <span class="os-win">On Windows Kage lives in the system tray (the icons by the clock; it may be under the <strong>^</strong> arrow until you drag it out). Closing the window keeps it running there. The window has its own minimize, maximize and close buttons at the top right, like other Windows 11 apps.</span>
      </p>

      <h2 id="menu-bar-controller"><span class="os-mac">Menu Bar</span><span class="os-win">Tray</span> Quick Popover</h2>
      <p>
        Clicking the Kage icon in the <span class="os-mac">macOS menu bar</span><span class="os-win">Windows system tray</span> opens a quick popover<span class="os-win"> just above the taskbar</span>. Its mascot is also the connect button: it sleeps when you are off, looks busy while connecting, smiles when you are protected, and looks worried when the tunnel is up but traffic is not flowing.
      </p>
      <ul>
        <li><strong>Toggle Connection:</strong> One-tap connect or disconnect by clicking the mascot.</li>
        <li><strong>Status at a glance:</strong> A headline ("You're protected", "Checking…", "No internet") with a latency pill you can tap to re-test.</li>
        <li><strong>Server, timer and IP:</strong> The current server, how long you have been connected, and your exit IP (hidden until you click the eye). While disconnected this row is a server picker.</li>
        <li><strong>Data usage:</strong> Live download and upload speed, today's total and a live graph.</li>
        <li class="os-mac"><strong>Mode switch:</strong> Full System TUN or Proxy Only, locked while connected.</li>
        <li><strong><span class="os-mac">Menu bar menu</span><span class="os-win">Tray menu</span>:</strong> Right-click the icon for a status line, Connect, Ping Server and Quit.</li>
      </ul>

      <h2 id="setup-guide">The Setup Guide</h2>
      <p>
        On first launch a short setup guide gets the engine, adds your first server and chooses a connection mode, then introduces the mascot, the <span class="os-mac">menu bar</span><span class="os-win">tray</span> popover, data usage, routing rules, per-app controls and data caps for apps. Each of the last steps shows the real controls you will find in the app. Replay it any time from <strong>Settings, Help, Run setup again</strong>.
      </p>
      <figure class="shot"><img src="assets/images/tour-apps.png" alt="The setup guide step about controlling each app, showing VPN, Direct and Block choices" decoding="async"></figure>

      <h2 id="main-dashboard">Full Dashboard Window</h2>
      <p>
        Clicking <strong>Open Dashboard</strong> or selecting it from the <span class="os-mac">menu bar</span><span class="os-win">tray menu</span> launches the complete management console:
      </p>
      <ul>
        <li><strong>Servers:</strong> Comprehensive node organizer, latency pings, and subscription manager.</li>
        <li><strong>Routing:</strong> Rule-based traffic routing editor with direct GeoIP / GeoSite bypasses.</li>
        <li><strong>App controls:</strong> VPN, Direct or Block for each installed app, and data caps for apps. See <a href="#/user-guide/app-controls">App Controls &amp; Data Caps</a>.</li>
        <li><strong>Connections:</strong> An <em>Apps</em> view showing which apps are sending and receiving data right now (with live speeds), and a <em>Connections</em> view with every open socket, powered by sing-box's Clash API.</li>
        <li><strong>Diagnostics:</strong> One-click comparison of the tunnel with your normal route, DNS resolver check and latency graphs.</li>
        <li><strong>Data usage:</strong> This month's total, a day-by-day chart and what used the data.</li>
        <li><strong>Settings:</strong> <span class="os-mac">Kill switch, </span>reliability, the system proxy, connecting by network, logs, the monthly data limit, usage privacy, backup and app updates.</li>
        <li><strong>Engine</strong> and <strong>Logs:</strong> sing-box versions and update policy, and the engine's log with filters and hints.</li>
      </ul>

      <h2 id="permission-setup">Initial Permission Authorization</h2>
      <p class="os-win">
        On Windows nothing needs administrator rights yet: Proxy Only mode and the system proxy run as you. The installer is per-user too. TUN mode, which will need a one-time approval (UAC) to install a small helper service, comes in a later release.
      </p>
      <p class="os-mac">
        When you connect in <strong>Full System (TUN)</strong> mode for the first time, macOS requests root authorization. This is required because creating a virtual network adapter (<code>utun</code>) and binding system routes requires elevated kernel privileges:
      </p>
    </div>

    <div class="callout-banner info os-mac">
      <div class="callout-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
      </div>
      <div>
        <strong>Root Privileges Scoped to Tunnel Daemon:</strong> Kage does not run the entire graphical interface as root. Elevated privileges are strictly scoped to the sing-box helper process responsible for virtual interface allocation.
      </div>
    </div>
  `
};

window.DOCS_PAGES["getting-started/engine-setup"] = {
  category: "Getting Started",
  title: "Engine Setup & Version Manager",
  featureBadge: "sing-box Core",
  lead: "Manage the underlying sing-box core binary. Learn about managed core downloads, checksum verification<span class='os-mac'>, and switching to a custom Homebrew installation</span>.",
  heroImage: {
    src: "assets/images/engine.png",
    alt: "Engine Management Screen",
    title: "sing-box Core Engine Manager"
  },
  toc: [
    { id: "engine-modes", label: "Managed Core vs System Core" },
    { id: "managed-download", label: "Automatic Core Downloader" },
    { id: "custom-binary", label: "Using Homebrew or Custom Binary", os: "macos" },
    { id: "required-tags", label: "Required Build Tags" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="engine-modes">Managed Core vs System Core</h2>
      <p>
        Kage VPN decouples its graphical client from the proxy engine. You can choose between:
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
              <td><code class="os-mac">~/Library/Application Support/com.apzur.kage/cores/sing-box/&lt;version&gt;/sing-box</code><code class="os-win">%APPDATA%\\com.apzur.kage\\cores\\sing-box\\&lt;version&gt;\\sing-box.exe</code></td>
              <td>Downloaded and verified automatically within the app with SHA-256 checks.</td>
            </tr>
            <tr class="os-mac">
              <td><strong>System Core</strong></td>
              <td><code>/opt/homebrew/bin/sing-box</code> or <code>/usr/local/bin/sing-box</code></td>
              <td>Managed externally via Homebrew (<code>brew install sing-box</code>).</td>
            </tr>
          </tbody>
        </table>
      </div>

      <h2 id="managed-download">Automatic Core Downloader</h2>
      <p>
        On first launch, if no valid sing-box binary is detected, Kage displays a setup prompt. Clicking <strong>Download Engine</strong> pulls the latest verified release from GitHub Releases (<code class="os-mac">sing-box-&lt;version&gt;-darwin-&lt;cpu&gt;.tar.gz</code><code class="os-win">sing-box-&lt;version&gt;-windows-amd64.zip</code>), checks its official SHA-256 checksum, unpacks <code class="os-mac">sing-box</code><code class="os-win">sing-box.exe</code><span class="os-mac">, marks the executable with <code>chmod +x</code></span>, and stores it in your app data folder.
      </p>

      <p class="os-win">On Windows the engine is always the managed one: there is no Homebrew, and Kage does not look for a <code>sing-box.exe</code> elsewhere on the PC.</p>
    </div>

    <div class="docs-prose os-mac">
      <h2 id="custom-binary">Using Homebrew or Custom Binary</h2>
      <p>
        If you prefer managing your system tools through Homebrew:
      </p>
    </div>

    <div class="code-block-container os-mac">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="brew install sing-box">Copy</button>
      </div>
      <pre><code>brew install sing-box
which sing-box
# Output: /opt/homebrew/bin/sing-box</code></pre>
    </div>

    <div class="docs-prose">
      <p class="os-mac">Navigate to <strong>Settings &rarr; Core Engine</strong>, select <strong>System Core</strong>, and select or input the path to your installed binary.</p>

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
