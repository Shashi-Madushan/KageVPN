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
    { id: "oct-13", label: "13 Oct 2026" },
    { id: "oct-12", label: "12 Oct 2026" },
    { id: "oct-11", label: "11 Oct 2026" },
    { id: "oct-8", label: "8 Oct 2026" },
    { id: "oct-7", label: "7 Oct 2026" },
    { id: "oct-5", label: "5 Oct 2026" },
    { id: "oct-4", label: "4 Oct 2026" },
    { id: "oct-3", label: "3 Oct 2026" },
    { id: "oct-2", label: "2 Oct 2026" },
    { id: "oct-1", label: "1 Oct 2026" }
  ],
  contentHtml: `
    <div class="docs-prose">
      <h2 id="oct-13">13 October 2026</h2>
      <ul>
        <li><strong>New:</strong> <strong>Settings &rarr; Logs</strong>: choose how much the engine writes down. <em>Errors only</em> keeps no list of the sites you visit (apart from ones that failed to load); <em>Normal</em> and <em>Detailed</em> list every connection. The default is Normal, where it used to be Detailed. A second switch empties the log files whenever the VPN turns off.</li>
        <li><strong>Fixed:</strong> Another program (or a second copy of the app) using port 2080 or 19090 no longer stops you from connecting. Kage moves to the next free port, tells you, and in Proxy Only mode shows the address to use on the Home page.</li>
      </ul>

      <h2 id="oct-12">12 October 2026</h2>
      <ul>
        <li><strong>New:</strong> Split tunneling by app. Add an <strong>App</strong> rule on the Routing page by picking from your installed apps, or click an app on <strong>Connections &rarr; Apps</strong> and choose Through VPN, Direct or Block. The rule covers the app's helper processes too, which typing a process name did not.</li>
        <li><strong>New:</strong> Clash and sing-box configuration files can be imported, from the Add server box or as a subscription. Servers Kage cannot run are listed with the reason.</li>
        <li><strong>New:</strong> Subscription options: choose which client the app asks as, for providers that send a different list (or none) depending on the app, and send a device ID for providers that limit how many devices may use a subscription. Subscriptions can now be edited, including their link.</li>
        <li><strong>New:</strong> Clear messages when a provider refuses a subscription: device limit reached, device ID needed, link expired or not found. Error messages no longer include the subscription link, which carries your account token.</li>
        <li><strong>New:</strong> One notification when 90% of your provider's data is used and one when it runs out. The menu bar popover shows the data and days left for the server you use. Plans with unlimited data now show their end date.</li>
        <li><strong>New:</strong> Switch servers from the menu bar popover without disconnecting.</li>
        <li><strong>New:</strong> <strong>Remove unreachable</strong> on the Servers page deletes every server the last test could not reach, after asking.</li>
        <li><strong>New:</strong> Back up your servers, subscriptions, routing and settings to one file, and restore it on this or another Mac (Settings &rarr; Backup).</li>
        <li><strong>Fixed:</strong> While connected, <strong>Test all</strong> reported servers added after connecting (or beyond the first 150) as timed out. They are now measured directly.</li>
      </ul>

      <h2 id="oct-11">11 October 2026</h2>
      <ul>
        <li><strong>Fixed:</strong> Country lists (such as <em>CN</em> or ad lists) are now downloaded by the app, with a backup address, and kept in the data folder. Before, a network that blocks GitHub stopped the connection from starting at all once a list was turned on. A list that cannot be downloaded is now left out, with a notification, and the connection still starts. Lists refresh weekly.</li>
        <li><strong>Fixed:</strong> In Full System (TUN) mode IPv6 traffic could skip the VPN on networks that have IPv6. The tunnel now carries IPv6 as well.</li>
        <li><strong>Fixed:</strong> The app could close itself when it met a server link or a long crash log containing non-English characters. It no longer does.</li>
        <li><strong>Fixed:</strong> If saved data could not be read (for example because of file permissions), the app could replace your servers with an empty list. It now keeps the file and tells you. A single server this version does not understand, such as one added by a newer version, no longer hides all your other servers; it is kept and comes back when it is readable.</li>
        <li><strong>Fixed:</strong> Reconnecting no longer gives up for good after about a minute. If the network is still down it waits and tries again when the network changes or the Mac wakes. A reconnect also stops when you connect or disconnect yourself, and does not keep asking for your password after you cancel it.</li>
        <li><strong>Fixed:</strong> A single blocked website could make a working VPN look dead (and, with <em>Restart a dead tunnel</em> on, restart it). Three different sites are now tried before the tunnel is called down.</li>
        <li><strong>Fixed:</strong> Custom DNS servers keep their address details. A personal link such as <code>https://dns.nextdns.io/your-id</code> used to lose its path and ask the wrong resolver; <code>tls://</code>, <code>quic://</code>, <code>udp://</code> and <code>tcp://</code> work too. An invalid entry is refused instead of silently switching to the system resolver.</li>
        <li><strong>Fixed:</strong> Settings that failed to save used to look as if they had saved. The screen now only changes when the change was kept.</li>
        <li><strong>Fixed:</strong> Downloads of the engine and of country lists no longer hang when the connection drops, and cancelling works at any time. The sing-box log no longer grows without limit during a long connection.</li>
        <li><strong>Fixed:</strong> A subscription link whose server names contain spaces now keeps the whole name, and a first link preceded by an invisible byte-order mark is no longer dropped. Servers using a protocol Kage cannot run yet (such as TUIC) are now reported with the reason instead of vanishing.</li>
        <li><strong>Improved:</strong> The subscription expiry reminder appears once for each expiry date, three days before and again when it ends, instead of on every refresh. If the subscription provider cannot be reached directly, Kage tries through the VPN when one is running.</li>
        <li><strong>Improved:</strong> The menu bar speed readout keeps its unit above 100 MB/s. After an app update, the VPN service script is refreshed with a single password prompt if it is out of date.</li>
        <li><strong>New:</strong> You can cancel a connection that is still starting: click the mascot (or the menu bar item, now labelled <strong>Cancel</strong>) while it says Connecting.</li>
        <li><strong>Fixed:</strong> Connecting right after disconnecting no longer fails with "port already in use" while the previous engine is still shutting down; the app waits a few seconds for it.</li>
        <li><strong>Fixed:</strong> Refreshing a subscription no longer undoes a change you made while it was downloading, and no longer brings back servers of a subscription you removed in the meantime. A subscription that keeps failing is retried less and less often instead of every half hour. Placeholder rows that providers add (such as "Remaining traffic", pointing at 127.0.0.1) are no longer imported as servers.</li>
        <li><strong>Corrected:</strong> The documentation described WebRTC and IPv6 tests and STUN blocking that the app does not have. It now describes only what the Diagnostics page really does.</li>
      </ul>

      <h2 id="oct-8">8 October 2026</h2>
      <ul>
        <li><strong>Fixed:</strong> The app no longer says "You're protected" while the internet is down. It now checks that traffic really flows through the tunnel before showing green.</li>
        <li><strong>New:</strong> The tunnel is monitored for as long as you are connected. If it stops passing traffic you see "Connected, but no internet" with the reason (server not answering, server rejected the connection, or no traffic). It recovers on its own when traffic returns.</li>
        <li><strong>New:</strong> Failed connections now explain what went wrong with a button for the fix. The server is checked before connecting, so an unreachable server fails in seconds.</li>
        <li><strong>New:</strong> A Connection health card on the home screen, a status line and a warning icon in the menu bar, and loading skeletons in the menu bar popover and home screen.</li>
        <li><strong>New:</strong> The Connections page has an <strong>Apps</strong> view: every app using the VPN with its live sending and receiving speed and how much it has sent and received. Click an app to see which servers it is talking to.</li>
        <li><strong>New:</strong> When sing-box or the tunnel stops on its own, you get a notification with the reason (exit status and last error), a banner in the app, and an entry in <code>crashes.log</code> in the data folder. Open it from the banner or the <strong>Crash log</strong> button on the Logs page.</li>
        <li><strong>Changed:</strong> The <strong>Data usage</strong> page is redesigned. It opens on <em>This month</em> and says in plain words how much you used and how that compares with last month. Choose Today, 7 days, This month or Last month with one tap. A new "Where did it go?" list shows which apps, websites and servers used the data. The confusing filters moved into optional sections. The data limit and usage privacy options moved to <strong>Settings</strong>, under their own Data usage and Usage privacy sections.</li>
        <li><strong>Improved:</strong> The welcome tour is updated with the mascot and the new screens. It now has a <strong>Tap the mascot to connect</strong> step (tap to start, tap again to disconnect), a step that explains the mascot's faces (off, checking, protected, no internet), the new menu bar popover and the plain-language usage page. Replay it any time from Settings.</li>
        <li><strong>Improved:</strong> The menu bar popover shows the latency as a pill under the headline, and its usage graph now fills the free space.</li>
        <li><strong>Changed:</strong> The Kage mascot is now the connect button on the home screen and in the menu bar popover, with a face that shows whether you are protected.</li>
        <li><strong>Changed:</strong> "Connection watchdog" is now "Restart a dead tunnel". Monitoring is always on, and the setting only controls whether a dead tunnel is restarted automatically.</li>
      </ul>

      <h2 id="oct-7">7 October 2026</h2>
      <ul>
        <li><strong>New:</strong> The app is now called <strong>Kage VPN</strong>, with a new moon-and-ninja icon and menu bar glyphs. Your servers, settings and usage history move over automatically on first launch.</li>
        <li><strong>Changed:</strong> The URL scheme is now <code>kagevpn://</code> (was <code>apzurvpn://</code>) and the data folder is <code>com.apzur.kage</code>.</li>
        <li><strong>New:</strong> Kage VPN shows a Dock icon while its main window is open, and goes back to the menu bar only when you close it.</li>
        <li><strong>Changed:</strong> The credit line now reads <strong>Kage by Apzur</strong> instead of "Powered by Apzur".</li>
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
