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
    { id: "oct-20", label: "20 Oct 2026" },
    { id: "oct-19", label: "19 Oct 2026" },
    { id: "oct-16", label: "16 Oct 2026" },
    { id: "oct-15", label: "15 Oct 2026" },
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
      <h2 id="oct-20">20 October 2026</h2>
      <ul>
        <li><strong>New:</strong> An <strong>Apps</strong> tab on the Routing page lists your installed apps, each with <em>VPN</em>, <em>Direct</em> and <em>Block</em> buttons, app icons, and the data the app used this period. Two presets set whether everything uses the VPN except the apps you pick, or only the apps you pick use it. Blocked apps are listed with an <em>Unblock</em> button. This app and its engine cannot be blocked.</li>
        <li><strong>Improved:</strong> <strong>Connections</strong> (Apps tab) is redesigned. Columns are wide enough that numbers no longer get cut off, each app has an initial and a connection count, and a thin bar shows how busy it is compared with the others. Click a column heading to sort.</li>
        <li><strong>Fixed:</strong> Speeds under 1 KB could show as long raw numbers such as <code>896.0519740&hellip;</code>. They are now rounded, and an idle app shows a dash.</li>
        <li><strong>Improved:</strong> Checking for and downloading app updates goes through the VPN while it is connected, so a network that blocks GitHub no longer stops updates. It falls back to a direct request if that fails. When no release has been published yet, the message now says so.</li>
      </ul>

      <h2 id="oct-19">19 October 2026</h2>
      <ul>
        <li><strong>New:</strong> A <strong>setup guide</strong> replaces the welcome tour. It walks through downloading the engine, adding a server or subscription (paste a link right in the guide) and choosing the connection mode, then explains the mascot, the menu-bar popover and the usage pages. Run it again any time from Settings, or from Home when no server has been added yet.</li>
        <li><strong>New:</strong> <strong>Route by site name (FakeIP)</strong> (Routing, DNS), for Full system mode only and off by default. Sites are routed by their name instead of an address looked up on your Mac, which keeps domain rules exact. Local names (<code>.local</code>, <code>.lan</code> and similar) still resolve normally. Turn it off if an app that remembers addresses misbehaves.</li>
        <li><strong>Improved:</strong> Pressing <em>Disconnect</em> now shows <strong>Disconnecting&hellip;</strong> on the mascot, Home, the menu-bar popover and the menu until the engine has actually stopped, and further presses are ignored. Before, the button looked connected for a few seconds and extra clicks could queue more stops.</li>
        <li><strong>Fixed:</strong> <em>Cancel</em> while connecting did nothing until the current step finished. It now closes the administrator password prompt, withdraws the request to the VPN service, and abandons a port wait or list download straight away.</li>
      </ul>

      <h2 id="oct-16">16 October 2026</h2>
      <ul>
        <li><strong>Improved:</strong> The menu-bar popover's server list shows each server's measured latency. While the VPN is connected, macOS no longer slows the app's background timers (App Nap), so the health check and automatic reconnect run on time.</li>
        <li><strong>New:</strong> <strong>Networks</strong> (Settings). Connect or disconnect the VPN automatically on each network. A network is recognised by its router's hardware address, so no location permission is needed. A network you have not used before asks once; the choice is saved and can be changed or forgotten in Settings. A <em>Disconnect</em> action also lifts the kill switch there. Connecting or disconnecting by hand wins until the network changes.</li>
        <li><strong>New:</strong> <strong>Allow IPv6</strong> (Routing, Network). Turn it off if a server has no IPv6 and some apps are slow to connect: IPv6 traffic is then blocked, never sent around the VPN.</li>
        <li><strong>New:</strong> <strong>Export diagnostics</strong> (Diagnostics) saves a zip to your Downloads folder with the logs, the engine configuration and a summary. Server addresses, passwords, keys, user IDs, subscription links and your home folder are replaced with placeholders, and nothing is sent anywhere.</li>
        <li><strong>New:</strong> <strong>Repair VPN service</strong> (Diagnostics) installs the VPN service again and clears the leftovers of an interrupted connection, with one password prompt.</li>
        <li><strong>New:</strong> <strong>Roll back</strong> (Engine) goes back to the engine version that last carried a working connection. <strong>Remove old versions</strong> deletes the other versions kept on disk.</li>
      </ul>

      <h2 id="oct-15">15 October 2026</h2>
      <ul>
        <li><strong>Redesigned:</strong> The <strong>Routing</strong> page is easier to use. Three cards choose how traffic is handled, an <strong>Everything else</strong> card sets the default, and three tabs hold <strong>Your rules</strong>, <strong>Ready-made lists</strong> and <strong>Try a website</strong>. Rules and lists use plain words (<em>Use the VPN</em>, <em>Go direct</em>, <em>Block</em>), and the network settings sit in their own section.</li>
        <li><strong>New:</strong> Ready-made lists are picked by name instead of codes: countries, advertising, Google, Netflix, Telegram and dozens more, each with a sensible default action. The old "country code" box is still there under <em>Other&hellip;</em>.</li>
        <li><strong>New:</strong> Lists can come from other sources. Under <strong>Downloads and sources</strong> choose SagerNet (the default), MetaCubeX, v2rayN, Loyalsoldier (IP lists) or runetfreedom (Russia), or add your own address that contains <code>{name}</code>. Sources must publish sing-box <code>.srs</code> files; V2Ray <code>.dat</code> files cannot be used.</li>
        <li><strong>New:</strong> <em>Downloaded lists</em> shows each list's size and age, with buttons to download, update or delete one or all of them.</li>
        <li><strong>Improved:</strong> Lists are downloaded through the VPN first when you are connected, then directly. A list that could not be fetched before connecting (for example because GitHub is blocked) is fetched through the tunnel as soon as it is verified, and you are told to reconnect to apply it.</li>
        <li><strong>Improved:</strong> The kill switch (still experimental) keeps blocking while a reconnect is in progress and while the tunnel is up but carrying no traffic, instead of letting traffic out after the first failed attempt. It lets IPv6 neighbour discovery through, keeps a refreshed list of your server's addresses, and Home shows <strong>Internet blocked by the kill switch</strong> with an <strong>Unblock internet</strong> button. For up to 20 seconds while a reconnect starts, DNS lookups are allowed so the server name can be found. It has been tested against a stand-in firewall only, so try it on your own Mac before relying on it.</li>
        <li><strong>New:</strong> An optional password for the local proxy (Settings, Reliability), for Proxy Only mode and other users on your Mac.</li>
        <li><strong>Changed:</strong> A dead tunnel is now restarted automatically by default on new installs. Existing installs keep their setting.</li>
        <li><strong>Fixed:</strong> The kill switch marker could say "released" a moment before the firewall really was. Several tests that could fail by chance now run reliably.</li>
      </ul>

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
