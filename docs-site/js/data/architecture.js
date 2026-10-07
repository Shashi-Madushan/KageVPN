/**
 * Apzur VPN Documentation — Architecture Deep Dive Pages
 */
window.DOCS_PAGES = window.DOCS_PAGES || {};

window.DOCS_PAGES["architecture/system-overview"] = {
  category: "Architecture",
  title: "Subsystem Architecture & Overview",
  featureBadge: "System Design",
  lead: "Examine how Apzur VPN orchestrates its reactive frontend, Tauri 2 Rust supervisor, root helper script, and the sing-box core into a resilient, high-speed macOS application.", 
  toc: [
    { id: "component-topology", label: "Component Topology" },
    { id: "data-control-flow", label: "Data & Control Flow" },
    { id: "process-isolation", label: "Process Isolation & Security" },
    { id: "ipc-boundaries", label: "IPC Boundaries" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="component-topology">Component Topology</h2>
      <p>
        Apzur VPN avoids monolithic design by segregating responsibilities into distinct execution domains:
      </p>

      <div class="docs-table-wrapper">
        <table class="docs-table">
          <thead>
            <tr>
              <th>Subsystem</th>
              <th>Tech Stack</th>
              <th>Execution Context</th>
              <th>Primary Role</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><strong>Frontend Shell</strong></td>
              <td>Svelte 5 + Tailwind CSS + Vite</td>
              <td>WKWebView (Userspace)</td>
              <td>Reactive UI, menu bar popover, interactive charts, and settings.</td>
            </tr>
            <tr>
              <td><strong>Application Core</strong></td>
              <td>Tauri 2 + Rust (tokio)</td>
              <td>Standard User Process</td>
              <td>State coordination, SQLite storage, ping testing, config compiler.</td>
            </tr>
            <tr>
              <td><strong>Privileged Helper</strong></td>
              <td>POSIX Shell (<code>helper.sh</code>)</td>
              <td>Elevated Root Daemon</td>
              <td>Kernel <code>utun</code> creation, route injection, and <code>pfctl</code> rules.</td>
            </tr>
            <tr>
              <td><strong>Proxy Engine</strong></td>
              <td>sing-box (Go)</td>
              <td>Managed Child Process</td>
              <td>Encrypted protocol encapsulation, TLS camouflage, packet multiplexing.</td>
            </tr>
          </tbody>
        </table>
      </div>

      <h2 id="data-control-flow">Data & Control Flow</h2>
      <p>The system follows an event-driven flow:</p>
    </div>

    <div class="step-sequence">
      <div class="step-item done">
        <div class="step-number-badge">1</div>
        <div class="step-head">
          <div class="step-title">User Action Dispatched</div>
          <span class="step-badge-tag">Frontend</span>
        </div>
        <div class="step-body">
          <p>User clicks 'Connect' in Svelte 5. An asynchronous Tauri invoke call (<code>invoke("start_engine")</code>) is transmitted across the WebKit IPC bridge.</p>
        </div>
      </div>

      <div class="step-item done">
        <div class="step-number-badge">2</div>
        <div class="step-head">
          <div class="step-title">Configuration Compilation & Validation</div>
          <span class="step-badge-tag">Rust</span>
        </div>
        <div class="step-body">
          <p>Rust merges the selected server model with routing rules and DNS preferences, validating JSON schema before writing the runtime config to a temporary descriptor.</p>
        </div>
      </div>

      <div class="step-item">
        <div class="step-number-badge">3</div>
        <div class="step-head">
          <div class="step-title">Privileged Helper Bootstrap</div>
          <span class="step-badge-tag">Root Helper</span>
        </div>
        <div class="step-body">
          <p>If in TUN mode, Rust executes <code>scripts/helper.sh</code> via macOS <code>osascript</code> (or authorized daemon), passing config paths and process supervisory parameters.</p>
        </div>
      </div>

      <div class="step-item">
        <div class="step-number-badge">4</div>
        <div class="step-head">
          <div class="step-title">Telemetry Stream Initialized</div>
          <span class="step-badge-tag">Clash API</span>
        </div>
        <div class="step-body">
          <p>Once sing-box binds to the Clash REST API socket, Rust opens a persistent WebSocket stream, piping realtime bitrates and active connection states directly to Svelte runes.</p>
        </div>
      </div>
    </div>
  `
};

window.DOCS_PAGES["architecture/tauri-ipc"] = {
  category: "Architecture",
  title: "Tauri 2 IPC & State Store",
  featureBadge: "Rust Backend",
  lead: "Discover how Apzur leverages Tauri 2's secure inter-process communication bridge and thread-safe Rust Mutex stores to guarantee responsive UI interactions.",
  toc: [
    { id: "command-dispatch", label: "Command Invocation" },
    { id: "event-streams", label: "Asynchronous Event Streams" },
    { id: "state-locking", label: "State Synchronization & Mutexes" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="command-dispatch">Command Invocation</h2>
      <p>
        Tauri 2 provides strongly typed bindings between the frontend TypeScript/JavaScript code and the native Rust backend. Every command is exposed as a rust function decorated with <code>#[tauri::command]</code>:
      </p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">rust</span>
        <button class="btn-copy-code" data-code="#[tauri::command]&#10;pub async fn connect_server(server_id: String, state: State<'_, AppState>) -> Result&lt;ConnectionStatus, String&gt; {&#10;    let mut engine = state.engine.lock().await;&#10;    engine.start(&server_id).await.map_err(|e| e.to_string())&#10;}">Copy</button>
      </div>
      <pre><code>#[tauri::command]
pub async fn connect_server(
    server_id: String, 
    state: State&lt;'_, AppState&gt;
) -&gt; Result&lt;ConnectionStatus, String&gt; {
    let mut engine = state.engine.lock().await;
    engine.start(&amp;server_id).await.map_err(|e| e.to_string())
}</code></pre>
    </div>

    <div class="docs-prose">
      <h2 id="event-streams">Asynchronous Event Streams</h2>
      <p>
        Telemetry such as current upload/download transfer rates and connection socket counts are pushed to the frontend using Tauri events. This eliminates continuous polling and keeps CPU usage under 0.1% while idle:
      </p>
      <ul>
        <li><code>engine://traffic-update</code> — Payload containing <code>{ up: 142050, down: 4892100 }</code> bytes/sec.</li>
        <li><code>engine://state-change</code> — Broadcasts <code>Connecting</code>, <code>Connected</code>, or <code>Disconnected</code>.</li>
        <li><code>engine://log-entry</code> — Streamed stdout/stderr lines with secret redaction applied.</li>
      </ul>

      <h2 id="state-locking">State Synchronization & Mutexes</h2>
      <p>
        The shared application state is maintained in memory via <code>tokio::sync::Mutex&lt;AppState&gt;</code>. File persistence is executed asynchronously to ensure that reading UI state never blocks the main rendering thread.
      </p>
    </div>
  `
};

window.DOCS_PAGES["architecture/core-supervision"] = {
  category: "Architecture",
  title: "Core Supervision & Stopfile Protocol",
  featureBadge: "Lifecycle",
  lead: "Deep dive into Apzur's zero-orphan daemon supervisor and the file-based handshake protocol that guarantees clean process termination.",
  toc: [
    { id: "orphan-problem", label: "The Orphan Process Risk" },
    { id: "stopfile-handshake", label: "The .stop Handshake Protocol" },
    { id: "pid-monitoring", label: "PID Reaping & Signal Traps" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="orphan-problem">The Orphan Process Risk</h2>
      <p>
        In applications that launch privileged daemons, if the parent GUI crashes or terminates abruptly, the child daemon can easily continue running in the background. This leaves the virtual TUN interface locked, keeping your internet redirected even after the app is closed.
      </p>

      <h2 id="stopfile-handshake">The .stop Handshake Protocol</h2>
      <p>
        Apzur VPN solves this via a robust, file-based stopfile protocol implemented between Rust and <code>scripts/helper.sh</code>:
      </p>
    </div>

    <div class="step-sequence">
      <div class="step-item done">
        <div class="step-number-badge">1</div>
        <div class="step-head">
          <div class="step-title">Runtime Directory & PID File Creation</div>
        </div>
        <div class="step-body">
          <p>On startup, the helper script writes the child sing-box PID to <code>singbox.pid</code> inside the designated runtime directory.</p>
        </div>
      </div>

      <div class="step-item">
        <div class="step-number-badge">2</div>
        <div class="step-head">
          <div class="step-title">Stopfile Signal Generation</div>
        </div>
        <div class="step-body">
          <p>When disconnecting or quitting, Apzur touches a sentinel file named <code>singbox.stop</code> in the runtime directory and sends <code>SIGTERM</code>.</p>
        </div>
      </div>

      <div class="step-item">
        <div class="step-number-badge">3</div>
        <div class="step-head">
          <div class="step-title">Graceful Socket Teardown & Clean-up</div>
        </div>
        <div class="step-body">
          <p>The helper script catches the sentinel file or signal, allows sing-box 1.5 seconds to flush TCP buffers, tears down the <code>utun</code> adapter, removes the stopfile, and exits cleanly.</p>
        </div>
      </div>
    </div>

    <div class="docs-prose">
      <h2 id="pid-monitoring">PID Reaping & Signal Traps</h2>
      <p>
        If sing-box does not terminate within the grace period, <code>helper.sh</code> issues a forced <code>SIGKILL</code> (<code>kill -9</code>) and runs a thorough cleanup on all leftover pfctl rules, guaranteeing that the user's internet is restored without manual rebooting.
      </p>
    </div>
  `
};

window.DOCS_PAGES["architecture/tun-pfctl"] = {
  category: "Architecture",
  title: "TUN Interface & macOS pfctl Routing",
  featureBadge: "Kernel Networking",
  lead: "Learn how Apzur creates the virtual utun device, installs system gateway overrides, and enforces packet filter anchors for zero-leak security.",
  toc: [
    { id: "utun-allocation", label: "Virtual utun Allocation" },
    { id: "route-table-manipulation", label: "Default Gateway Overrides" },
    { id: "pfctl-anchor-rules", label: "pfctl Anchor Rules" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="utun-allocation">Virtual utun Allocation</h2>
      <p>
        On macOS, point-to-point tunnel devices are created dynamically using the kernel's <code>utun</code> driver. sing-box opens <code>/dev/net/tun</code> or requests a dynamic socket from <code>PF_SYSTEM</code>:
      </p>
      <code>ioctl(fd, CTLS_IOC_G_NETIF_NAME, &name); // yields utun3</code>

      <h2 id="route-table-manipulation">Default Gateway Overrides</h2>
      <p>
        Instead of modifying the primary default gateway (which could break physical link renegotiation when switching Wi-Fi networks), Apzur utilizes a dual subnet mask technique:
      </p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="route add -net 0.0.0.0/1 -interface utun3&#10;route add -net 128.0.0.0/1 -interface utun3">Copy</button>
      </div>
      <pre><code>route add -net 0.0.0.0/1 -interface utun3
route add -net 128.0.0.0/1 -interface utun3</code></pre>
    </div>

    <div class="docs-prose">
      <p>
        Because <code>/1</code> is more specific than the default <code>0.0.0.0/0</code> route, the macOS kernel automatically routes all outbound internet packets into the tunnel while keeping the true gateway available for physical adapter handshakes.
      </p>

      <h2 id="pfctl-anchor-rules">pfctl Anchor Rules</h2>
      <p>
        When the Kill Switch is active, Apzur injects rules into the <code>com.apple/250.ApzurVPN</code> anchor table:
      </p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">pfctl rules</span>
        <button class="btn-copy-code" data-code="block drop out on en0 all&#10;pass out on en0 proto tcp to 198.51.100.25 port 443">Copy</button>
      </div>
      <pre><code># Drop all egress on physical interfaces
block drop out on en0 all
block drop out on en1 all

# Allow traffic strictly destined to the active proxy endpoint
pass out on en0 proto tcp to 198.51.100.25 port 443
pass out on en0 proto udp to 198.51.100.25 port 443</code></pre>
    </div>
  `
};

window.DOCS_PAGES["architecture/clash-ring-buffer"] = {
  category: "Architecture",
  title: "Clash API & Ring Buffer Telemetry",
  featureBadge: "Real-time Metrics",
  lead: "Explore how Apzur queries sing-box's built-in Clash REST API and manages an in-memory ring buffer to provide instant connection telemetry without eating CPU.", 
  toc: [
    { id: "clash-api-socket", label: "Clash REST API Socket" },
    { id: "circular-buffer", label: "Traffic Ring Buffer" },
    { id: "connection-tracking", label: "Live Sockets Inspector" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="clash-api-socket">Clash REST API Socket</h2>
      <p>
        sing-box integrates Clash API compatibility. When launched by Apzur, it starts a local REST endpoint on <code>127.0.0.1:9090</code> secured with an ephemeral bearer token generated at runtime.
      </p>
      <ul>
        <li><code>GET /traffic</code> — Realtime byte streams (1 Hz broadcast).</li>
        <li><code>GET /connections</code> — Active and closed socket connections list.</li>
        <li><code>DELETE /connections/{id}</code> — Closes specific problematic sockets immediately.</li>
      </ul>

      <h2 id="circular-buffer">Traffic Ring Buffer</h2>
      <p>
        To draw historical 60-second bandwidth charts on the dashboard, the Rust backend maintains a fixed-capacity circular ring buffer (<code>VecDeque&lt;TrafficSample&gt;</code>) with a hard cap of 60 items. New telemetry samples push old ones out, maintaining an exact zero-allocation, bounded memory footprint.
      </p>

      <h2 id="connection-tracking">Live Sockets Inspector</h2>
      <p>
        The Connections tab provides a live breakdown of every open socket: destination domain/IP, protocol rule triggered (direct vs proxy), upload/download byte total, and connection duration.
      </p>
    </div>
  `
};

window.DOCS_PAGES["architecture/config-pipeline"] = {
  category: "Architecture",
  title: "Configuration Generator Pipeline",
  featureBadge: "Compiler",
  lead: "Understand the deterministic JSON configuration pipeline that transforms user interface models into fully compliant sing-box configuration files.",
  toc: [
    { id: "pipeline-phases", label: "Compiler Pipeline Phases" },
    { id: "inbound-generation", label: "Inbound & Outbound Generation" },
    { id: "validation-step", label: "sing-box Check Validation" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="pipeline-phases">Compiler Pipeline Phases</h2>
      <p>
        Generating an invalid configuration file can cause sing-box to crash on startup. Apzur uses a three-stage compilation pipeline before any process is executed:
      </p>
    </div>

    <div class="step-sequence">
      <div class="step-item done">
        <div class="step-number-badge">1</div>
        <div class="step-head">
          <div class="step-title">State AST Extraction</div>
        </div>
        <div class="step-body">
          <p>Extract active server configuration, DNS settings, and routing rules from the local SQLite/JSON state store.</p>
        </div>
      </div>

      <div class="step-item done">
        <div class="step-number-badge">2</div>
        <div class="step-head">
          <div class="step-title">JSON Assembly & Inbound Binding</div>
        </div>
        <div class="step-body">
          <p>Merge templates: creates TUN inbounds (with MTU 9000 and stack <code>mixed</code> or <code>system</code>) and outbound protocols.</p>
        </div>
      </div>

      <div class="step-item">
        <div class="step-number-badge">3</div>
        <div class="step-head">
          <div class="step-title">Static Schema Verification</div>
        </div>
        <div class="step-body">
          <p>Executes <code>sing-box check -c /tmp/config.json</code> in dry-run mode. If any schema error occurs, the process halts safely and reports the exact JSON line error to the user.</p>
        </div>
      </div>
    </div>
  `
};

window.DOCS_PAGES["architecture/privacy-redaction"] = {
  category: "Architecture",
  title: "Secret Redaction & Diagnostic Privacy",
  featureBadge: "Security",
  lead: "Explore Apzur's privacy sanitization engine that prevents passwords, secret UUIDs, private keys, and server IPs from ever leaking into diagnostic logs.", 
  toc: [
    { id: "privacy-commitment", label: "Privacy Commitment" },
    { id: "redaction-filters", label: "Redaction Filters & Regex" },
    { id: "privacy-toggles", label: "Interactive Privacy Toggles" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="privacy-commitment">Privacy Commitment</h2>
      <p>
        When debugging connection drops, users often need to export diagnostic logs to GitHub issues or community forums. Unsanitized proxy logs frequently contain sensitive metadata: user UUIDs, TLS SNI destinations, server IPs, and authentication passwords.
      </p>

      <h2 id="redaction-filters">Redaction Filters & Regex</h2>
      <p>
        All raw stdout and stderr output from the sing-box process passes through an in-memory redaction pipe before being written to disk:
      </p>
      <ul>
        <li><strong>UUID Masking:</strong> <code>[0-9a-f]{8}-[0-9a-f]{4}...</code> &rarr; <code>[REDACTED_UUID]</code></li>
        <li><strong>Password / Secret Masking:</strong> Reality keys, Trojan passwords, and Hysteria passwords &rarr; <code>[REDACTED_SECRET]</code></li>
        <li><strong>IPv4 / IPv6 Masking:</strong> Server remote addresses are obfuscated to <code>198.51.***.***</code></li>
      </ul>

      <h2 id="privacy-toggles">Interactive Privacy Toggles</h2>
      <p>
        The UI includes individual privacy toggles for both IP addresses and server domains, allowing users to mask or reveal addresses temporarily when copying debug diagnostics.
      </p>
    </div>
  `
};
