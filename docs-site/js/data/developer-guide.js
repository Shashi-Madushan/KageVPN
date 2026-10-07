/**
 * Kage VPN Documentation — Developer Guide Pages
 */
window.DOCS_PAGES = window.DOCS_PAGES || {};

window.DOCS_PAGES["developer-guide/local-setup"] = {
  category: "Developer Guide",
  title: "Local Development Setup",
  featureBadge: "Environment",
  lead: "Set up your macOS development workstation to compile, run, debug, and package Kage VPN from source code.",
  toc: [
    { id: "prerequisites", label: "Prerequisites & Toolchain" },
    { id: "clone-repo", label: "Cloning & Installing Dependencies" },
    { id: "running-dev", label: "Starting Development Server" },
    { id: "building-dmg", label: "Building Release Artifacts" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="prerequisites">Prerequisites & Toolchain</h2>
      <p>
        Building Kage VPN requires macOS (Intel or Apple Silicon) with the following developer dependencies:
      </p>
      <ul>
        <li><strong>Xcode Command Line Tools:</strong> <code>xcode-select --install</code></li>
        <li><strong>Node.js:</strong> Version 20 LTS or later (with <code>npm</code>)</li>
        <li><strong>Rust:</strong> 1.78 or newer via <code>rustup</code> (<code>curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh</code>)</li>
        <li><strong>sing-box:</strong> Recommended installed via <code>brew install sing-box</code> for local development testing.</li>
      </ul>

      <h2 id="clone-repo">Cloning & Installing Dependencies</h2>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="git clone https://github.com/shashimadushan/singbox-gui.git&#10;cd singbox-gui&#10;npm install">Copy</button>
      </div>
      <pre><code># Clone repository
git clone https://github.com/shashimadushan/singbox-gui.git
cd singbox-gui

# Install frontend JavaScript & Vite dependencies
npm install</code></pre>
    </div>

    <div class="docs-prose">
      <h2 id="running-dev">Starting Development Server</h2>
      <p>
        Launch the live hot-reloading development environment with Tauri 2:
      </p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="npm run tauri dev">Copy</button>
      </div>
      <pre><code>npm run tauri dev</code></pre>
    </div>

    <div class="docs-prose">
      <p>
        This boots the Vite local server for Svelte 5 and compiles the Rust backend in debug mode with hot reloading on frontend changes.
      </p>

      <h2 id="building-dmg">Building Release Artifacts</h2>
      <p>
        To produce a standalone production <code>.dmg</code> installer:
      </p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="npm run tauri build">Copy</button>
      </div>
      <pre><code>npm run tauri build</code></pre>
    </div>

    <div class="docs-prose">
      <p>The compiled macOS bundle is placed in <code>src-tauri/target/release/bundle/dmg/</code>.</p>
    </div>
  `
};

window.DOCS_PAGES["developer-guide/svelte-frontend"] = {
  category: "Developer Guide",
  title: "Svelte 5 Frontend Architecture",
  featureBadge: "Svelte 5 Runes",
  lead: "Understand the reactive architecture powering Kage's GUI, utilizing modern Svelte 5 runes ($state, $derived, $effect) and modular components.",
  toc: [
    { id: "runes-paradigm", label: "Svelte 5 Runes Paradigm" },
    { id: "component-hierarchy", label: "Component Hierarchy" },
    { id: "styling-tokens", label: "Tailwind CSS Styling" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="runes-paradigm">Svelte 5 Runes Paradigm</h2>
      <p>
        Kage VPN is built on Svelte 5, taking full advantage of fine-grained signal reactivity via runes:
      </p>
      <ul>
        <li><code>$state()</code>: Manages mutable component state (e.g., active server, connection status, selected tab).</li>
        <li><code>$derived()</code>: Automatically computes derived metrics like active server latency tag and formatted bitrate labels.</li>
        <li><code>$effect()</code>: Synchronizes theme settings and window dimension listeners with Tauri event subscriptions.</li>
      </ul>

      <h2 id="component-hierarchy">Component Hierarchy</h2>
      <div class="docs-table-wrapper">
        <table class="docs-table">
          <thead>
            <tr>
              <th>Component</th>
              <th>Location</th>
              <th>Responsibility</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><code>Navbar</code> / <code>TabBar</code></td>
              <td><code>src/lib/components/</code></td>
              <td>Top navigation switching between Servers, Routing, Connections, Diagnostics, Settings.</td>
            </tr>
            <tr>
              <td><code>ServerCard</code></td>
              <td><code>src/lib/components/</code></td>
              <td>Displays server protocol tag, ping status badge, menu dropdown, and quick connect.</td>
            </tr>
            <tr>
              <td><code>TrafficChart</code></td>
              <td><code>src/lib/components/</code></td>
              <td>Realtime SVG/Canvas bandwidth visualizer reading 1 Hz Clash API ring buffer events.</td>
            </tr>
            <tr>
              <td><code>PrivacyToggle</code></td>
              <td><code>src/lib/components/</code></td>
              <td>Masks/unmasks server addresses and sensitive IPs in diagnostics and logs view.</td>
            </tr>
          </tbody>
        </table>
      </div>

      <h2 id="styling-tokens">Tailwind CSS Styling</h2>
      <p>
        The UI uses Tailwind CSS with dark mode classes (<code>dark:bg-slate-900</code>) bound to the root <code>[data-theme]</code> attribute, ensuring seamless light and dark mode transitions matching macOS appearances.
      </p>
    </div>
  `
};

window.DOCS_PAGES["developer-guide/rust-backend"] = {
  category: "Developer Guide",
  title: "Rust Backend & app_config",
  featureBadge: "Rust Core",
  lead: "Explore the Rust application layer in src-tauri, handling persistent settings, atomic configuration files, and privileged process execution.",
  toc: [
    { id: "backend-structure", label: "Backend Structure" },
    { id: "app-config", label: "The app_config.rs Module" },
    { id: "atomic-file-io", label: "Atomic File I/O Safety" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="backend-structure">Backend Structure</h2>
      <p>
        The Rust codebase is located in <code>src-tauri/src/</code>:
      </p>
      <ul>
        <li><code>main.rs</code>: Tauri application builder, plugin registration, and window management.</li>
        <li><code>app_config.rs</code>: Central configuration constants, update public keys, and default paths.</li>
        <li><code>profile/</code>: Pluggable protocol registry and <code>ProtocolDef</code> trait implementations.</li>
        <li><code>core/</code>: Process lifecycle management, ping tester, and Clash API client.</li>
        <li><code>helper/</code>: Integration with <code>helper.sh</code> and root privilege handshakes.</li>
      </ul>

      <h2 id="app-config">The app_config.rs Module</h2>
      <p>
        All application-level constants and update configurations are consolidated in <code>app_config.rs</code>:
      </p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">rust</span>
        <button class="btn-copy-code" data-code="pub const APP_ID: &str = &quot;com.apzur.kage&quot;;&#10;pub const APP_UPDATE_PUBKEY: &str = &quot;...&quot;;&#10;pub const DEFAULT_SOCKS_PORT: u16 = 2080;">Copy</button>
      </div>
      <pre><code>pub const APP_ID: &amp;str = "com.apzur.kage";
pub const APP_UPDATE_PUBKEY: &amp;str = "RWTw9f...";
pub const DEFAULT_SOCKS_PORT: u16 = 2080;
pub const DEFAULT_CLASH_PORT: u16 = 9090;</code></pre>
    </div>

    <div class="docs-prose">
      <h2 id="atomic-file-io">Atomic File I/O Safety</h2>
      <p>
        To prevent corrupted configurations if a laptop suddenly runs out of battery, state persistence writes to a temporary sibling file (e.g. <code>config.json.tmp</code>) followed by an atomic filesystem rename (<code>std::fs::rename</code>).
      </p>
    </div>
  `
};

window.DOCS_PAGES["developer-guide/adding-protocols"] = {
  category: "Developer Guide",
  title: "Adding a Protocol (ProtocolDef Trait)",
  featureBadge: "Extending",
  lead: "Step-by-step walkthrough of implementing the ProtocolDef trait to add a brand new proxy protocol without touching the frontend UI.",
  toc: [
    { id: "trait-overview", label: "The ProtocolDef Trait" },
    { id: "step-by-step-impl", label: "Implementation Steps" },
    { id: "registering-protocol", label: "Registering in REGISTRY" },
    { id: "conformance-testing", label: "Conformance Verification" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="trait-overview">The ProtocolDef Trait</h2>
      <p>
        Each protocol in Kage is an isolated module in <code>src-tauri/src/profile/</code> implementing the <code>ProtocolDef</code> trait. The editor UI, subscription parser, validation, duplicate detection, and config generator are all generated dynamically from this trait!
      </p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">rust</span>
        <button class="btn-copy-code" data-code="pub trait ProtocolDef: Send + Sync {&#10;    fn id(&self) -> &'static str;&#10;    fn label(&self) -> &'static str;&#10;    fn schemes(&self) -> &'static [&'static str];&#10;    fn parse(&self, link: &str) -> Result&lt;Profile, ParseError&gt;;&#10;    fn to_link(&self, profile: &Profile) -> Result&lt;String, LinkError&gt;;&#10;    fn validate(&self, profile: &Profile) -> Vec&lt;ValidationError&gt;;&#10;    fn outbound(&self, profile: &Profile) -> serde_json::Value;&#10;    fn secrets(&self, profile: &Profile) -> Vec&lt;String&gt;;&#10;    fn form(&self) -> Vec&lt;Field&gt;;&#10;    fn blank(&self) -> Profile;&#10;}">Copy</button>
      </div>
      <pre><code>pub trait ProtocolDef: Send + Sync {
    fn id(&amp;self) -&gt; &amp;'static str;
    fn label(&amp;self) -&gt; &amp;'static str;
    fn schemes(&amp;self) -&gt; &amp;'static [&amp;'static str];
    fn parse(&amp;self, link: &amp;str) -&gt; Result&lt;Profile, ParseError&gt;;
    fn to_link(&amp;self, profile: &amp;Profile) -&gt; Result&lt;String, LinkError&gt;;
    fn validate(&amp;self, profile: &amp;Profile) -&gt; Vec&lt;ValidationError&gt;;
    fn outbound(&self, profile: &amp;Profile) -&gt; serde_json::Value;
    fn secrets(&amp;self, profile: &amp;Profile) -&gt; Vec&lt;String&gt;;
    fn form(&amp;self) -&gt; Vec&lt;Field&gt;;
    fn blank(&amp;self) -&gt; Profile;
}</code></pre>
    </div>

    <div class="step-sequence">
      <div class="step-item">
        <div class="step-number-badge">1</div>
        <div class="step-head"><div class="step-title">Create profile/&lt;name&gt;.rs</div></div>
        <div class="step-body"><p>Define your settings struct with <code>#[serde(rename_all = "camelCase", default)]</code> and implement the trait.</p></div>
      </div>
      <div class="step-item">
        <div class="step-number-badge">2</div>
        <div class="step-head"><div class="step-title">Register in profile/mod.rs</div></div>
        <div class="step-body"><p>Add your unit struct instance to the global <code>REGISTRY</code> slice.</p></div>
      </div>
      <div class="step-item">
        <div class="step-number-badge">3</div>
        <div class="step-head"><div class="step-title">Add Conformance Sample</div></div>
        <div class="step-body"><p>Provide a valid test link in <code>sample()</code>. The automated test suite immediately covers round-trip parsing, duplicate detection, and sing-box JSON check!</p></div>
      </div>
    </div>
  `
};

window.DOCS_PAGES["developer-guide/releasing"] = {
  category: "Developer Guide",
  title: "Releasing & Minisign Signing",
  featureBadge: "Release Ops",
  lead: "How production releases are compiled, cryptographically signed with Minisign, and staged for GitHub Releases.",
  toc: [
    { id: "signing-keys", label: "Private Signing Key Setup" },
    { id: "release-script", label: "The release.mjs Script" },
    { id: "github-actions", label: "GitHub Releases Automation" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="signing-keys">Private Signing Key Setup</h2>
      <p>
        Kage VPN builds enforce end-to-end cryptographic verification: installed copies refuse any auto-update that is not signed by the designated Minisign private key matching <code>APP_UPDATE_PUBKEY</code> in <code>app_config.rs</code>.
      </p>

      <h2 id="release-script">The release.mjs Script</h2>
      <p>
        A dedicated Node release orchestrator (<code>scripts/release.mjs</code>) manages version bumps and packaging:
      </p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="npm run release -- --version 0.2.0 --github shashimadushan/singbox-gui --notes &quot;Feature update&quot;">Copy</button>
      </div>
      <pre><code># 1. Build, sign and stage locally
npm run release -- --version 0.2.0 --github shashimadushan/singbox-gui --notes "Feature update"

# 2. Inspect release/0.2.0/, then publish
npm run release -- --version 0.2.0 --github shashimadushan/singbox-gui --notes "Feature update" --skip-build --publish</code></pre>
    </div>

    <div class="docs-prose">
      <h2 id="github-actions">GitHub Releases Automation</h2>
      <p>
        The script verifies that git working tree is clean, checks public key parity, signs the <code>.dmg</code> and tarballs, writes <code>latest.json</code> manifest, and creates the GitHub release via <code>gh release create</code>.
      </p>
    </div>
  `
};

window.DOCS_PAGES["developer-guide/testing"] = {
  category: "Developer Guide",
  title: "Testing & Conformance Verification",
  featureBadge: "Quality Assurance",
  lead: "Run the comprehensive Rust unit tests, frontend type checks, and protocol conformance verification suites.",
  toc: [
    { id: "rust-tests", label: "Rust Unit & Integration Tests" },
    { id: "frontend-checks", label: "Frontend Type Checks" },
    { id: "conformance-suite", label: "Protocol Conformance Suite" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="rust-tests">Rust Unit & Integration Tests</h2>
      <p>Run all backend tests covering configuration serialization and state management:</p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="cargo test --manifest-path src-tauri/Cargo.toml">Copy</button>
      </div>
      <pre><code>cargo test --manifest-path src-tauri/Cargo.toml</code></pre>
    </div>

    <div class="docs-prose">
      <h2 id="frontend-checks">Frontend Type Checks</h2>
      <p>Verify Svelte 5 rune types and TypeScript declarations:</p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">bash</span>
        <button class="btn-copy-code" data-code="npm run check">Copy</button>
      </div>
      <pre><code>npm run check</code></pre>
    </div>

    <div class="docs-prose">
      <h2 id="conformance-suite">Protocol Conformance Suite</h2>
      <p>
        The conformance suite in <code>tests/profile_conformance.rs</code> validates every registered protocol against <code>sing-box check</code>, ensuring zero syntactical bugs can reach users.
      </p>
    </div>
  `
};
