/**
 * Kage VPN Documentation — Protocol Reference Pages
 */
window.DOCS_PAGES = window.DOCS_PAGES || {};

window.DOCS_PAGES["protocol-reference/vless"] = {
  category: "Protocol Reference",
  title: "VLESS (Reality & Vision)",
  featureBadge: "Next-Gen TLS",
  lead: "Comprehensive reference for the lightweight, zero-encryption VLESS protocol paired with TLS Reality camouflage and XTLS Vision flow control.",
  toc: [
    { id: "vless-overview", label: "VLESS Protocol Overview" },
    { id: "reality-camouflage", label: "Reality TLS Camouflage" },
    { id: "vision-flow", label: "XTLS Vision Flow Control" },
    { id: "uri-schema", label: "URI Schema Specification" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="vless-overview">VLESS Protocol Overview</h2>
      <p>
        Unlike legacy protocols that duplicate encryption layers, VLESS is a state-free protocol that relies on the outer TLS transport for privacy. By eliminating duplicate symmetric encryption handshakes, VLESS achieves near-native wire speeds and lower CPU consumption on Apple Silicon devices.
      </p>

      <h2 id="reality-camouflage">Reality TLS Camouflage</h2>
      <p>
        Reality eliminates the need for purchasing a domain name or managing Let's Encrypt certificates on your proxy server. Instead, it "borrows" the TLS certificates of popular public websites (such as <code>gateway.icloud.com</code>, <code>www.microsoft.com</code>, or <code>dl.google.com</code>):
      </p>
      <ul>
        <li><strong>Server Name Indication (SNI):</strong> Pointed to the legitimate target server.</li>
        <li><strong>Public Key:</strong> Reality server's Curve25519 public key.</li>
        <li><strong>Short ID:</strong> Optional hex identifier for user identification.</li>
        <li><strong>SpiderX:</strong> Path crawling filter to mimic real browser requests.</li>
      </ul>

      <h2 id="vision-flow">XTLS Vision Flow Control</h2>
      <p>
        The <code>xtls-rprx-vision</code> flow dynamically eliminates TLS-in-TLS packet patterns, shielding traffic from deep packet inspection (DPI) classification algorithms.
      </p>

      <h2 id="uri-schema">URI Schema Specification</h2>
      <p>Example of a standard VLESS Reality URI parsed by Kage VPN:</p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">uri</span>
        <button class="btn-copy-code" data-code="vless://7e123456-789a-bcde-f012-3456789abcde@198.51.100.1:443?security=reality&encryption=none&pbk=7aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890abcdef&headerType=none&fp=chrome&type=tcp&flow=xtls-rprx-vision&sni=gateway.icloud.com&sid=0123456789abcdef#Tokyo-Reality">Copy URI</button>
      </div>
      <pre><code>vless://7e123456-789a-bcde-f012-3456789abcde@198.51.100.1:443
  ?security=reality
  &amp;encryption=none
  &amp;pbk=7aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890abcdef
  &amp;fp=chrome
  &amp;type=tcp
  &amp;flow=xtls-rprx-vision
  &amp;sni=gateway.icloud.com
  &amp;sid=0123456789abcdef#Tokyo-Reality</code></pre>
    </div>
  `
};

window.DOCS_PAGES["protocol-reference/vmess"] = {
  category: "Protocol Reference",
  title: "VMess Protocol",
  featureBadge: "V2Ray Core",
  lead: "The venerable proxy protocol from the V2Ray project, supporting WebSocket/gRPC transports, dynamic IDs, and payload obfuscation.",
  toc: [
    { id: "vmess-overview", label: "Protocol Overview" },
    { id: "supported-transports", label: "Supported Transports" },
    { id: "security-ciphers", label: "Security Ciphers" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="vmess-overview">Protocol Overview</h2>
      <p>
        VMess is an encrypted transmission protocol designed to prevent active probing attacks. Kage VPN supports the standard v2rayN Base64 JSON URI format.
      </p>

      <h2 id="supported-transports">Supported Transports</h2>
      <ul>
        <li><strong>TCP:</strong> Standard raw stream, optionally wrapped in TLS.</li>
        <li><strong>WebSocket (ws):</strong> Allows proxying behind Content Delivery Networks (Cloudflare, Fastly) using custom host headers and paths.</li>
        <li><strong>gRPC:</strong> Multiplexed HTTP/2 streams utilizing standard gRPC service names.</li>
        <li><strong>HTTPUpgrade:</strong> Efficient HTTP/1.1 connection upgrading.</li>
      </ul>

      <h2 id="security-ciphers">Security Ciphers</h2>
      <p>Kage negotiates modern AEAD encryption for VMess connections:</p>
      <ul>
        <li><code>auto</code> (Negotiates AES-128-GCM or ChaCha20-Poly1305 based on hardware CPU AES-NI instructions).</li>
        <li><code>aes-128-gcm</code> (Hardware accelerated on Apple Silicon).</li>
        <li><code>chacha20-poly1305</code> (Constant-time software cipher).</li>
      </ul>
    </div>
  `
};

window.DOCS_PAGES["protocol-reference/trojan"] = {
  category: "Protocol Reference",
  title: "Trojan Protocol",
  featureBadge: "TLS Mimicry",
  lead: "Mimic genuine HTTPS web server traffic to bypass state-level firewalls with the Trojan protocol.",
  toc: [
    { id: "trojan-concept", label: "The Trojan Concept" },
    { id: "alpn-negotiation", label: "ALPN Negotiation & Certificates" },
    { id: "uri-format", label: "URI Format" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="trojan-concept">The Trojan Concept</h2>
      <p>
        Trojan avoids defining an identifiable packet header. Instead, client connections initiate an authentic TLS handshake against a remote domain holding a valid CA certificate. If authentication succeeds, the remote server proxies the request; if an active probe is detected, it returns a standard web page (like an Apache or Nginx 404), rendering the proxy indistinguishable from a normal HTTPS website.
      </p>

      <h2 id="alpn-negotiation">ALPN Negotiation & Certificates</h2>
      <p>
        Kage negotiates ALPN protocols (<code>h2, http/1.1</code>) and checks TLS certificates against macOS's system Keychain root store.
      </p>

      <h2 id="uri-format">URI Format</h2>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">uri</span>
        <button class="btn-copy-code" data-code="trojan://YourStrongPassword@node1.example.com:443?security=tls&sni=node1.example.com#Frankfurt-Trojan">Copy</button>
      </div>
      <pre><code>trojan://YourStrongPassword@node1.example.com:443?security=tls&amp;sni=node1.example.com#Frankfurt-Trojan</code></pre>
    </div>
  `
};

window.DOCS_PAGES["protocol-reference/shadowsocks"] = {
  category: "Protocol Reference",
  title: "Shadowsocks 2022",
  featureBadge: "AEAD Protocol",
  lead: "Reference for Shadowsocks 2022 specification, incorporating BLAKE3 key derivation, session subkeys, and active probing replay mitigation.",
  toc: [
    { id: "shadowsocks-2022", label: "The 2022 Modernization" },
    { id: "supported-ciphers", label: "Supported AEAD Ciphers" },
    { id: "uri-specification", label: "URI Specification" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="shadowsocks-2022">The 2022 Modernization</h2>
      <p>
        The Shadowsocks 2022 specification resolves historic vulnerabilities in original Shadowsocks implementations. It mandates separate header and payload keys derived via BLAKE3, includes monotonic packet timestamps to thwart replay attacks, and enforces strict padding.
      </p>

      <h2 id="supported-ciphers">Supported AEAD Ciphers</h2>
      <ul>
        <li><code>2022-blake3-aes-128-gcm</code> (Default, optimal for 64-bit systems)</li>
        <li><code>2022-blake3-aes-256-gcm</code></li>
        <li><code>2022-blake3-chacha8-poly1305</code> (High throughput)</li>
        <li>Legacy: <code>aes-128-gcm</code>, <code>aes-256-gcm</code>, <code>chacha20-ietf-poly1305</code></li>
      </ul>

      <h2 id="uri-specification">URI Specification</h2>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">uri</span>
        <button class="btn-copy-code" data-code="ss://MjAyMi1ibGFrZTMtYWVzLTEyOC1nY206eW91cl9wYXNzd29yZA==@198.51.100.4:8388#Singapore-SS2022">Copy</button>
      </div>
      <pre><code>ss://MjAyMi1ibGFrZTMtYWVzLTEyOC1nY206eW91cl9wYXNzd29yZA==@198.51.100.4:8388#Singapore-SS2022</code></pre>
    </div>
  `
};

window.DOCS_PAGES["protocol-reference/hysteria2"] = {
  category: "Protocol Reference",
  title: "Hysteria 2 (Salamander)",
  featureBadge: "QUIC/UDP",
  lead: "High-throughput proxy protocol customized for lossy networks, powered by modified QUIC congestion control and Salamander packet obfuscation.",
  toc: [
    { id: "hysteria-concept", label: "Why Hysteria 2?" },
    { id: "salamander-obfuscation", label: "Salamander Obfuscation" },
    { id: "bandwidth-tuning", label: "Bandwidth Tuning" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="hysteria-concept">Why Hysteria 2?</h2>
      <p>
        Traditional TCP proxies throttle down drastically when packet loss occurs due to TCP congestion backoff. Hysteria 2 uses a customized UDP QUIC protocol (Brutal congestion control) that maintains wire-speed data rates even on long-distance links experiencing 10% to 30% packet loss.
      </p>

      <h2 id="salamander-obfuscation">Salamander Obfuscation</h2>
      <p>
        Some ISPs intentionally throttle standard UDP QUIC traffic. Kage supports Hysteria 2's Salamander obfuscation: packets are XOR-masked with a pre-shared password before hitting the wire, preventing ISP deep-packet inspection from recognizing QUIC headers.
      </p>

      <h2 id="bandwidth-tuning">Bandwidth Tuning</h2>
      <p>
        Hysteria 2 allows declaring peak upload and download bandwidth constraints to maximize throughput without exceeding local interface capacities:
      </p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">uri</span>
        <button class="btn-copy-code" data-code="hysteria2://MyAuthPassword@node.hy2.example.com:443?obfs=salamander&obfs-password=SecretXor&upmbps=50&downmbps=200&sni=node.hy2.example.com#US-Hysteria2">Copy</button>
      </div>
      <pre><code>hysteria2://MyAuthPassword@node.hy2.example.com:443
  ?obfs=salamander
  &amp;obfs-password=SecretXor
  &amp;upmbps=50
  &amp;downmbps=200
  &amp;sni=node.hy2.example.com#US-Hysteria2</code></pre>
    </div>
  `
};
