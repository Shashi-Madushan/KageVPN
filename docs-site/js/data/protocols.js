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

window.DOCS_PAGES["protocol-reference/tuic"] = {
  category: "Protocol Reference",
  title: "TUIC v5",
  featureBadge: "QUIC/UDP",
  lead: "A QUIC-based proxy protocol that signs in with a UUID and a password and lets you pick the congestion control and how UDP is carried.",
  toc: [
    { id: "tuic-overview", label: "Overview" },
    { id: "tuic-options", label: "Options" },
    { id: "tuic-limits", label: "What Kage refuses" },
    { id: "tuic-uri", label: "URI Format" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="tuic-overview">Overview</h2>
      <p>
        TUIC runs over QUIC (UDP), so it keeps working well on lossy links and sets up connections quickly. Kage supports version 5, which every current TUIC server speaks. Because it uses UDP, the check Kage runs before connecting skips the TCP test for TUIC servers.
      </p>

      <h2 id="tuic-options">Options</h2>
      <ul>
        <li><strong>User ID and password:</strong> both are required.</li>
        <li><strong>Congestion control:</strong> BBR (default), CUBIC or NewReno. Use what your provider recommends.</li>
        <li><strong>UDP relay:</strong> Native (default) sends each UDP packet as a QUIC datagram; QUIC sends them over streams, which is slower but works on networks that drop datagrams.</li>
        <li><strong>0-RTT handshake:</strong> connects faster, but a recorded handshake can be replayed. Off unless the server asks for it.</li>
        <li><strong>ALPN:</strong> <code>h3</code> when the link names none, since QUIC needs one and TUIC servers almost always expect it.</li>
      </ul>

      <h2 id="tuic-limits">What Kage refuses</h2>
      <p>These are listed as skipped with the reason, rather than imported in a way that would not work:</p>
      <ul>
        <li>TUIC v4 links (a token instead of a UUID and password).</li>
        <li><code>disable_sni=1</code>: sing-box always sends the server name.</li>
        <li>Port hopping (a port list such as <code>443,5000-6000</code>).</li>
      </ul>

      <h2 id="tuic-uri">URI Format</h2>
      <p>Links from v2rayN, NekoBox and Hiddify use this form. Kage also reads <code>congestion_controller</code>, <code>reduce_rtt</code> and <code>insecure</code>.</p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">uri</span>
        <button class="btn-copy-code" data-code="tuic://7e123456-789a-4cde-8f01-3456789abcde:MyPassword@tuic.example.com:443?sni=tuic.example.com&alpn=h3&congestion_control=bbr&udp_relay_mode=native#Tokyo-TUIC">Copy</button>
      </div>
      <pre><code>tuic://7e123456-789a-4cde-8f01-3456789abcde:MyPassword@tuic.example.com:443
  ?sni=tuic.example.com
  &amp;alpn=h3
  &amp;congestion_control=bbr
  &amp;udp_relay_mode=native#Tokyo-TUIC</code></pre>
    </div>
  `
};

window.DOCS_PAGES["protocol-reference/hysteria"] = {
  category: "Protocol Reference",
  title: "Hysteria (v1)",
  featureBadge: "QUIC/UDP",
  lead: "The first version of Hysteria. It has been replaced by Hysteria 2, but some providers still hand it out, so Kage can run it.",
  toc: [
    { id: "hysteria1-overview", label: "Overview" },
    { id: "hysteria1-speeds", label: "Upload and download speeds" },
    { id: "hysteria1-limits", label: "What Kage refuses" },
    { id: "hysteria1-uri", label: "URI Format" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="hysteria1-overview">Overview</h2>
      <p>
        Hysteria runs over QUIC (UDP) and sends as fast as the speeds you declare, which keeps it quick on lossy links. The password is optional (some servers accept anyone), and <code>xplus</code> obfuscation is supported. If your provider offers Hysteria 2 as well, prefer it.
      </p>

      <h2 id="hysteria1-speeds">Upload and download speeds</h2>
      <p>
        Hysteria needs to know your upload and download speed. Set them close to what your connection really manages: much higher wastes bandwidth and can slow everything down, much lower caps your speed. When a link leaves them out, Kage uses 10 Mbps up and 50 Mbps down, says so after the import, and shows the speeds in the server list (for example <em>Hysteria · 10/50 Mbps</em>). Change them in the server editor. You can type <code>100</code>, <code>100 Mbps</code> or <code>1 Gbps</code>.
      </p>

      <h2 id="hysteria1-limits">What Kage refuses</h2>
      <ul>
        <li><code>faketcp</code> and <code>wechat-video</code> modes: only plain UDP runs.</li>
        <li>Obfuscation other than <code>xplus</code>, or <code>xplus</code> without a password.</li>
        <li>Port hopping (<code>mport</code> or a port list).</li>
        <li>In sing-box configs, an <code>auth</code> key given only as binary (base64); use <code>auth_str</code>.</li>
      </ul>

      <h2 id="hysteria1-uri">URI Format</h2>
      <p>Kage reads the official Hysteria link form, and also a password before the host, <code>sni</code> for <code>peer</code> and <code>up</code>/<code>down</code> for the speeds.</p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">uri</span>
        <button class="btn-copy-code" data-code="hysteria://hy.example.com:443?protocol=udp&auth=MyPassword&peer=hy.example.com&upmbps=20&downmbps=100&alpn=hysteria&obfs=xplus&obfsParam=MyMask#Tokyo-Hysteria">Copy</button>
      </div>
      <pre><code>hysteria://hy.example.com:443
  ?protocol=udp
  &amp;auth=MyPassword
  &amp;peer=hy.example.com
  &amp;upmbps=20
  &amp;downmbps=100
  &amp;alpn=hysteria
  &amp;obfs=xplus
  &amp;obfsParam=MyMask#Tokyo-Hysteria</code></pre>
    </div>
  `
};

window.DOCS_PAGES["protocol-reference/ssh"] = {
  category: "Protocol Reference",
  title: "SSH",
  featureBadge: "TCP",
  lead: "Use any SSH server you can log in to as a proxy. Handy for a server you already run, without installing anything on it.",
  toc: [
    { id: "ssh-overview", label: "Overview" },
    { id: "ssh-login", label: "Password or private key" },
    { id: "ssh-host-key", label: "Host key" },
    { id: "ssh-uri", label: "URI Format" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="ssh-overview">Overview</h2>
      <p>
        Kage opens an SSH session to the server and sends your connections through it, the same way <code>ssh -D</code> does. The server needs nothing beyond SSH itself, though its configuration must allow forwarding (<code>AllowTcpForwarding yes</code>, the default). SSH is fine for browsing and light use; for speed, a purpose-built protocol is better.
      </p>

      <h2 id="ssh-login">Password or private key</h2>
      <ul>
        <li><strong>Password:</strong> type it in the server editor.</li>
        <li><strong>Private key:</strong> paste the whole key, from <code>-----BEGIN</code> to <code>-----END</code>, into the <em>Private key</em> box. If the key has a passphrase, enter it under <em>Advanced options</em>. Key files on disk are not read; the pasted key is stored with your other server details.</li>
      </ul>

      <h2 id="ssh-host-key">Host key</h2>
      <p>
        The host key is how Kage knows it reached your server and not an impostor. Paste the server's full public host key line (for example <code>ssh-ed25519 AAAA…</code>), which you can read with <code>ssh-keyscan your.server</code> or from <code>/etc/ssh/ssh_host_ed25519_key.pub</code> on the server. A fingerprint (<code>SHA256:…</code>) is not enough. Without a host key Kage still connects, but the editor warns you and the server list shows <em>host not checked</em>.
      </p>

      <h2 id="ssh-uri">URI Format</h2>
      <p>There is no single standard for SSH links. Kage reads and writes this form; <code>private_key</code>, <code>passphrase</code> and <code>host_key</code> are optional (and may be written <code>pk</code>, <code>pkp</code> and <code>hk</code>). Several host keys are separated by commas.</p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">uri</span>
        <button class="btn-copy-code" data-code="ssh://deploy:MyPassword@ssh.example.com:22?host_key=ssh-ed25519%20AAAAC3NzaC1lZDI1NTE5AAAAIExample#My-SSH">Copy</button>
      </div>
      <pre><code>ssh://deploy:MyPassword@ssh.example.com:22
  ?host_key=ssh-ed25519%20AAAAC3NzaC1lZDI1NTE5AAAAIExample#My-SSH</code></pre>
    </div>
  `
};

window.DOCS_PAGES["protocol-reference/wireguard"] = {
  category: "Protocol Reference",
  title: "WireGuard",
  featureBadge: "UDP",
  lead: "Import WireGuard servers from links or from the .conf files many providers hand out, including Cloudflare WARP.",
  toc: [
    { id: "wg-overview", label: "Overview" },
    { id: "wg-conf", label: "Importing a .conf file" },
    { id: "wg-fields", label: "Settings" },
    { id: "wg-limits", label: "What Kage refuses" },
    { id: "wg-uri", label: "URI Format" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="wg-overview">Overview</h2>
      <p>
        Kage runs WireGuard inside sing-box, so it needs no extra permissions in Proxy Only mode and works with your routing rules like any other server. WireGuard uses UDP, so the check before connecting skips its TCP test. Your sing-box must be built with WireGuard support (the official builds are); if it is not, connecting says so and points to the Engine page.
      </p>

      <h2 id="wg-conf">Importing a .conf file</h2>
      <p>
        Drop the file on <em>Add server</em> (or paste its text). The server is named after the file, for example <em>warp-tokyo</em> for <code>warp-tokyo.conf</code>; pasted text uses a <code># Name =</code> line if there is one, else the server address.
      </p>
      <p>
        Some lines in a <code>.conf</code> are for the WireGuard app itself and Kage does not use them: <code>DNS</code>, an <code>AllowedIPs</code> narrower than everything, <code>Table</code>, <code>PostUp</code> and similar. They do not stop the import; the import result names them. Kage's own DNS and routing settings decide those things.
      </p>

      <h2 id="wg-fields">Settings</h2>
      <ul>
        <li><strong>Private key</strong> and <strong>server public key</strong>: required.</li>
        <li><strong>Local addresses</strong>: this device's addresses inside the tunnel (the file's <code>Address</code>).</li>
        <li><strong>Pre-shared key</strong>: optional.</li>
        <li><strong>MTU</strong>: 1408 unless the server says otherwise. Try 1280 if large pages stall.</li>
        <li><strong>Keepalive</strong>: optional, in seconds.</li>
        <li><strong>Reserved bytes</strong>: only for Cloudflare WARP, which uses them to tell clients apart. The list shows such servers as <em>WireGuard · WARP</em>.</li>
      </ul>

      <h2 id="wg-limits">What Kage refuses</h2>
      <ul>
        <li>Configs with more than one <code>[Peer]</code>.</li>
        <li>AmneziaWG options (<code>Jc</code>, <code>Jmin</code>, <code>S1</code> and so on): sing-box runs plain WireGuard.</li>
        <li>A file with no <code>Address</code> or no peer <code>Endpoint</code>.</li>
      </ul>

      <h2 id="wg-uri">URI Format</h2>
      <p>Kage reads and writes the form v2rayN and Hiddify use. <code>presharedkey</code>, <code>mtu</code>, <code>keepalive</code> and <code>reserved</code> are optional.</p>
    </div>

    <div class="code-block-container">
      <div class="code-header">
        <span class="code-lang-label">uri</span>
        <button class="btn-copy-code" data-code="wireguard://YOUR_PRIVATE_KEY@engage.cloudflareclient.com:2408?publickey=SERVER_PUBLIC_KEY&address=172.16.0.2%2F32%2Cfd01%3A%3A2%2F128&mtu=1280&reserved=12%2C34%2C56#WARP">Copy</button>
      </div>
      <pre><code>wireguard://YOUR_PRIVATE_KEY@engage.cloudflareclient.com:2408
  ?publickey=SERVER_PUBLIC_KEY
  &amp;address=172.16.0.2/32,fd01::2/128
  &amp;mtu=1280
  &amp;reserved=12,34,56#WARP</code></pre>
    </div>
  `
};
