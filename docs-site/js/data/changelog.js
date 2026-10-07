/**
 * Kage VPN Documentation — Changelog
 * Newest first. Keep entries short and user-facing; add a new <h2> block per release or date.
 */
window.DOCS_PAGES = window.DOCS_PAGES || {};

window.DOCS_PAGES["changelog/changelog"] = {
  category: "Changelog",
  title: "Changelog",
  lead: "What changed in Kage VPN, newest first. The app is pre-1.0 and not yet tagged, so entries are grouped by date.",
  toc: [
    { id: "oct-7", label: "7 Oct 2026" },
    { id: "oct-5", label: "5 Oct 2026" },
    { id: "oct-4", label: "4 Oct 2026" },
    { id: "oct-3", label: "3 Oct 2026" },
    { id: "oct-2", label: "2 Oct 2026" },
    { id: "oct-1", label: "1 Oct 2026" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="oct-7">7 October 2026</h2>
      <ul>
        <li><strong>New:</strong> The app is now called <strong>Kage VPN</strong>, with a new moon-and-ninja icon and menu bar glyphs. Your servers, settings and usage history move over automatically on first launch.</li>
        <li><strong>Changed:</strong> The URL scheme is now <code>kagevpn://</code> (was <code>apzurvpn://</code>) and the data folder is <code>com.apzur.kage</code>.</li>
      </ul>

      <h2 id="oct-5">5 October 2026</h2>
      <ul>
        <li><strong>New:</strong> Ping any server from the Servers page and see its latency in the list.</li>
        <li><strong>New:</strong> Separate privacy toggles for your IP address and server addresses.</li>
        <li><strong>New:</strong> App logs rotate automatically, and the last run's log is kept for diagnostics.</li>
        <li><strong>Improved:</strong> A root helper script now manages the sing-box lifecycle and its logging.</li>
      </ul>

      <h2 id="oct-4">4 October 2026</h2>
      <ul>
        <li><strong>New:</strong> Import Trojan, Shadowsocks, Hysteria2 and VMess servers, in addition to VLESS.</li>
        <li><strong>New:</strong> Server and IP addresses are blurred until you click the eye icon.</li>
        <li><strong>New:</strong> Apzur reconnects after your Mac wakes from sleep or the network changes.</li>
        <li><strong>Improved:</strong> The server editor shows the fields that belong to each protocol.</li>
        <li><strong>Improved:</strong> The menu bar popover uses less memory, and the usage database is faster.</li>
        <li><strong>Fixed:</strong> Add server no longer shows a stray Paste bubble.</li>
        <li><strong>Fixed:</strong> The speed test requests 25 MB per call and reports when the server refuses.</li>
      </ul>

      <h2 id="oct-3">3 October 2026</h2>
      <ul>
        <li><strong>New:</strong> The app is now called <strong>Apzur VPN</strong>. Data from earlier folders is migrated automatically.</li>
        <li><strong>New:</strong> A guided tour on first launch replaces the Home welcome card.</li>
        <li><strong>New:</strong> The speed test shows a live gauge for ping, download and upload.</li>
        <li><strong>New:</strong> System, Light and Dark theme switch in the sidebar.</li>
        <li><strong>New:</strong> First-run engine download with progress and cancel, plus an update policy (ask, auto or off).</li>
        <li><strong>New:</strong> A download manager in the title bar, and in-app updates for Apzur itself.</li>
        <li><strong>New:</strong> A feedback form with a redacted preview before anything is sent.</li>
        <li><strong>Improved:</strong> Redesigned Servers page and dialogs.</li>
        <li><strong>Improved:</strong> Data usage shows a flat, ranked table.</li>
        <li><strong>Improved:</strong> The main window opens wider so Home shows two columns.</li>
      </ul>

      <h2 id="oct-2">2 October 2026</h2>
      <ul>
        <li><strong>New:</strong> Native macOS notifications for connection events.</li>
        <li><strong>New:</strong> A status-first Home with a responsive layout and accessibility fixes.</li>
        <li><strong>Improved:</strong> Redesigned Logs, Data usage, Routing, Diagnostics and Engine pages.</li>
        <li><strong>Improved:</strong> Redesigned menu bar popover; the menu bar icon no longer flickers.</li>
        <li><strong>Improved:</strong> Routing presets, grouped sidebar navigation and leak remediation tips.</li>
        <li><strong>Fixed:</strong> Saves are atomic and owner-only; unreadable data is kept aside, and duplicate imports are skipped.</li>
      </ul>

      <h2 id="oct-1">1 October 2026</h2>
      <ul>
        <li><strong>New:</strong> First version: a menu bar manager for sing-box.</li>
        <li><strong>New:</strong> Engine management, data usage tracking and a richer VLESS client.</li>
      </ul>
    </div>
  `
};
