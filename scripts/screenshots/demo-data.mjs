// Fake backend for the screenshots in the README. Everything here is made up: documentation-range
// IP addresses (RFC 5737), example.com hosts and a placeholder UUID. No real server or account.
// The protocol descriptions come from the real backend (protocols.json, kept current by a test).

import { readFileSync } from 'node:fs';

const protocols = JSON.parse(readFileSync(new URL('./protocols.json', import.meta.url), 'utf8'));

/** Returns JavaScript that installs a fake Tauri backend before the app loads. */
export function demoScript({ connected = true, withServers = true, capOffer = false, name = 'Kage', version = '0.2.0' } = {}) {
	return `(${install.toString()})(${JSON.stringify({ connected, withServers, capOffer, name, version, protocols })});`;
}

function install(opts) {
	const MB = 1048576;
	const now = Math.floor(Date.now() / 1000);
	let seed = 7;
	const rnd = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
	const UUID = '11111111-2222-4333-8444-555555555555';

	const tcp = { kind: 'tcp', path: null, host: null, serviceName: null };
	const tls = (extra = {}) => ({ sni: null, fingerprint: 'chrome', alpn: [], allowInsecure: false, pinnedKey: null, reality: null, ...extra });
	const server = (id, name, host, group, protocol, settings, summary, extra = {}) => ({
		id, protocol, name, server: host, port: 443, settings, group, favorite: false, subscriptionId: null, summary, ...extra
	});
	const city = (id, name, host, group, extra = {}) =>
		server(id, name, host, group, 'vless', {
			uuid: UUID, security: 'reality', flow: 'xtls-rprx-vision', packetEncoding: null, transport: tcp,
			tls: tls({ sni: 'www.example.org', reality: { publicKey: 'DEMOPUBLICKEYDEMOPUBLICKEYDEMOPUBLICKEY00', shortId: 'a1b2c3d4' } })
		}, 'Reality · tcp', extra);
	const profiles = opts.withServers
		? [
				city('p1', 'Frankfurt, Germany', 'de1.example.com', 'Europe', { favorite: true, subscriptionId: 's1' }),
				city('p2', 'Amsterdam, Netherlands', 'nl1.example.com', 'Europe'),
				server('p3', 'London, United Kingdom', 'uk1.example.com', 'Europe', 'trojan', { password: 'demo', transport: { kind: 'ws', path: '/', host: null, serviceName: null }, tls: tls() }, 'Trojan · ws'),
				city('p4', 'New York, United States', 'us1.example.com', 'Americas', { favorite: true }),
				server('p5', 'Singapore', 'sg1.example.com', 'Asia', 'hysteria2', { password: 'demo', obfs: null, tls: tls({ fingerprint: null }) }, 'Hysteria2'),
				server('p6', 'Tokyo, Japan', 'jp1.example.com', 'Asia', 'shadowsocks', { method: 'aes-256-gcm', password: 'demo' }, 'Shadowsocks'),
				city('p7', 'Sydney, Australia', 'au1.example.com', 'Oceania')
			]
		: [];
	const pings = { p1: 38, p2: 41, p3: 52, p4: 118, p5: 164, p6: 187, p7: 242 };

	const history = Array.from({ length: 60 }, (_, i) => ({
		down: Math.round((900000 + 700000 * Math.sin(i / 6) + 500000 * rnd()) * (i > 40 ? 1.6 : 1)),
		up: Math.round(90000 + 60000 * rnd())
	}));

	const status = opts.connected
		? { state: 'connected', mode: 'tun', since: now - 5400, error: null, health: 'healthy', reason: null, latencyMs: 38, failure: null, proxyPort: null }
		: { state: 'disconnected', mode: 'tun', since: null, error: null, health: 'idle', reason: null, latencyMs: null, failure: null, proxyPort: null };
	const logLine = (level, text) => ({ level, text, hint: null });
	const logs = opts.connected
		? [
				logLine('info', 'sing-box 1.12.9 started'),
				logLine('info', 'inbound/tun[tun-in]: started at utun233'),
				logLine('info', 'outbound/vless[proxy]: connected to de1.example.com:443'),
				logLine('info', 'router: loaded 6 rules, 2 rule sets'),
				logLine('debug', 'dns: exchanged cloudflare-dns.com A'),
				logLine('info', 'inbound/tun[tun-in]: inbound connection from 10.0.0.2:51710'),
				logLine('warn', 'dns: upstream took 812ms to answer, retrying'),
				logLine('info', 'outbound/vless[proxy]: outbound connection to www.example.org:443'),
				logLine('info', 'router: match[domain_suffix=netflix.com] => proxy'),
				logLine('info', 'router: match[process_name=Spotify] => direct'),
				logLine('error', 'outbound/vless[proxy]: dial tcp 203.0.113.99:443: i/o timeout'),
				logLine('info', 'outbound/vless[proxy]: connection recovered after 1 retry')
			]
		: [];

	const snapshot = {
		profiles, activeId: profiles.length ? 'p1' : null, mode: 'tun', status, logs,
		traffic: opts.connected ? { up: 96000, down: 1750000, totalUp: 74 * MB, totalDown: 612 * MB } : { up: 0, down: 0, totalUp: 0, totalDown: 0 },
		history: opts.connected ? history : [],
		settings: { showSpeed: true, autoConnect: false, logLevel: 'normal', clearLogs: false }, launchAtLogin: true, auto: { enabled: false, group: null },
		subscriptions: [{ id: 's1', name: 'Example provider', url: 'https://example.com/sub', intervalHours: 24, lastUpdated: now - 7200, lastError: null, userInfo: { upload: 8e9, download: 41e9, total: 200e9, expire: now + 86400 * 18 }, userAgent: null, sendDeviceId: false }],
		routing: {
			mode: 'rule', bypassLan: true, ipv6: false, finalAction: 'proxy',
			rules: [
				// Put in place by Telegram's data cap; the Apps page shows it as "blocked by its data cap".
				{ id: 'cap:/Applications/Telegram.app', enabled: true, kind: 'app', value: '/Applications/Telegram.app', action: 'block', origin: 'cap' },
				{ id: 'r1', enabled: true, kind: 'domainSuffix', value: 'netflix.com', action: 'proxy', origin: 'manual' },
				{ id: 'r2', enabled: true, kind: 'app', value: '/Applications/Spotify.app', action: 'direct', origin: 'manual' },
				{ id: 'r5', enabled: true, kind: 'app', value: '/Applications/Slack.app', action: 'proxy', origin: 'manual' },
				{ id: 'r6', enabled: true, kind: 'app', value: '/Applications/zoom.us.app', action: 'direct', origin: 'manual' },
				{ id: 'r3', enabled: true, kind: 'domainKeyword', value: 'doubleclick', action: 'block', origin: 'manual' },
				{ id: 'r4', enabled: true, kind: 'ipCidr', value: '192.168.50.0/24', action: 'direct', origin: 'manual' }
			],
			ruleSets: [
				{ id: 'g1', enabled: true, kind: 'geoip', country: 'lk', action: 'direct' },
				{ id: 'g2', enabled: true, kind: 'geosite', country: 'category-ads-all', action: 'block' },
				{ id: 'g3', enabled: true, kind: 'geosite', country: 'netflix', action: 'proxy' }
			],
			ruleProvider: 'sagernet', customProviders: [],
			dns: { provider: 'cloudflare', custom: '', fakeIp: false }
		},
		reliability: { autoReconnect: true, watchdog: true, killSwitch: true, hotkey: true, proxyAuth: false, proxyPassword: '' },
		networks: {
			enabled: true,
			networks: [
				{ id: 'gw-home', name: 'Home Wi-Fi', action: 'ignore', lastSeen: now - 600 },
				{ id: 'gw-cafe', name: 'Café Wi-Fi', action: 'connect', lastSeen: now - 86400 * 2 }
			]
		},
		hotkey: '⌥⌘V'
	};

	// Usage over the last 30 days, with a weekly rhythm.
	const day = 86400;
	const start = now - (now % day) - 29 * day;
	const series = Array.from({ length: 30 }, (_, i) => {
		const weekend = [0, 6].includes(new Date((start + i * day) * 1000).getDay());
		const base = (weekend ? 2600 : 1400) * MB * 0.01 * (30 + 60 * rnd());
		return { start: start + i * day, down: Math.round(base), up: Math.round(base * (0.12 + 0.1 * rnd())) };
	});
	const sum = series.reduce((a, p) => ({ up: a.up + p.up, down: a.down + p.down }), { up: 0, down: 0 });
	const rows = (list) => list.map(([key, down, up]) => ({ key, down: Math.round(down * MB), up: Math.round(up * MB) }));
	const topApps = rows([
		['Safari (you)', 5200, 380], ['Slack Helper (you)', 1900, 640], ['Spotify (you)', 1650, 20], ['Mail (you)', 820, 210],
		['Xcode (you)', 640, 95], ['Messages (you)', 310, 120], ['App Store (you)', 280, 6], ['mDNSResponder (root)', 22, 18]
	]);
	const topDomains = rows([
		['www.youtube.com', 3900, 40], ['netflix.com', 2200, 8], ['github.com', 960, 410], ['cdn.example.net', 720, 12], ['api.slack.com', 340, 220]
	]);
	const topProfiles = rows([['p1', 6400, 820], ['p4', 3100, 410], ['p5', 800, 90]]);

	const release = (v, size, days) => ({ version: v, prerelease: false, publishedAt: new Date((now - days * day) * 1000).toISOString(), notes: 'Bug fixes and improvements.', size, url: 'https://example.com/x', sha256: 'abc' });
	const releases = [release('1.12.9', 24.5e6, 5), release('1.12.8', 24.3e6, 19), release('1.12.7', 24.2e6, 33), release('1.12.6', 24.0e6, 47)];
	const coreRef = { id: 'sing-box', name: 'sing-box', runnable: true, minVersion: '1.12.0', requiredFeatures: ['with_clash_api', 'with_utls'] };
	const coreStatus = {
		core: coreRef, cores: [coreRef],
		active: { path: '~/Library/Application Support/com.apzur.kage/core/sing-box/1.12.9/sing-box', version: '1.12.9', tags: ['with_clash_api', 'with_utls'], source: 'managed', missingTags: [], outdated: false },
		error: null, system: null,
		installed: [{ version: '1.12.9', path: '/x', installedAt: now - 5 * day }, { version: '1.12.8', path: '/y', installedAt: now - 19 * day }],
		settings: { managedVersion: '1.12.9', channel: 'stable', autoCheck: true, lastCheck: now - 3600, updatePolicy: 'ask', pendingVersion: null }, busy: false
	};

	const appPaths = { Spotify: '/Applications/Spotify.app', 'Slack Helper': '/Applications/Slack.app', Discord: '/Applications/Discord.app', Mail: '/System/Applications/Mail.app', zoom: '/Applications/zoom.us.app' };
	const conns = [
		['www.youtube.com', '142.250.0.1', '443', 'Safari', 'tcp', 18 * MB, 0.6 * MB, 'proxy', 'domain_suffix: youtube.com'],
		['netflix.com', '198.51.100.20', '443', 'Safari', 'tcp', 42 * MB, 1 * MB, 'proxy', 'domain_suffix: netflix.com'],
		['api.spotify.com', '198.51.100.31', '443', 'Spotify', 'tcp', 6 * MB, 0.2 * MB, 'direct', 'process_path_regex: ^/Applications/Spotify\\.app/'],
		['github.com', '140.82.0.4', '22', 'ssh', 'tcp', 0.4 * MB, 0.3 * MB, 'proxy', 'final'],
		['slack-edge.com', '198.51.100.44', '443', 'Slack Helper', 'tcp', 2.2 * MB, 0.9 * MB, 'proxy', 'final'],
		['gateway.discord.gg', '198.51.100.52', '443', 'Discord', 'tcp', 9 * MB, 1.4 * MB, 'proxy', 'final'],
		['imap.example.com', '198.51.100.61', '993', 'Mail', 'tcp', 3.1 * MB, 0.5 * MB, 'proxy', 'final'],
		['zoom.us', '198.51.100.70', '443', 'zoom', 'udp', 26 * MB, 11 * MB, 'direct', 'process_path_regex: ^/Applications/zoom\\.us\\.app/']
	].map((c, i) => ({ id: 'c' + i, host: c[0], destinationIp: c[1], destinationPort: c[2], process: c[3], appPath: appPaths[c[3]] ?? null, network: c[4], down: Math.round(c[5]), up: Math.round(c[6]), start: new Date((now - 120 - i * 300) * 1000).toISOString(), chain: c[7], rule: c[8] }));
	const apps = [
		['App Store', '/System/Applications/App Store.app'], ['Discord', '/Applications/Discord.app'], ['Google Chrome', '/Applications/Google Chrome.app'],
		['Mail', '/System/Applications/Mail.app'], ['Safari', '/Applications/Safari.app'], ['Slack', '/Applications/Slack.app'],
		['Spotify', '/Applications/Spotify.app'], ['Telegram', '/Applications/Telegram.app'], ['Xcode', '/Applications/Xcode.app'], ['Zoom', '/Applications/zoom.us.app']
	].map(([name, path]) => ({ name, path, bundleId: null }));

	// Data caps for three apps: one close to its limit, one blocked by its cap, and one that reached
	// its limit and offers a block (only with capOffer, so the main Home picture stays plain).
	const GB = 1024 * MB;
	const week = now - (now % day) - 3 * day;
	const cap = (name, path, limit, period, atLimit, used, state, offer, periodStart, periodEnd) => ({
		path, name, installed: true, limitBytes: limit, period, atLimit, used, up: Math.round(used * 0.2), down: Math.round(used * 0.8),
		percent: Math.min(100, (used / limit) * 100), level: state === 'ok' ? 0 : state === 'warn80' ? 80 : 100, state, offer, periodStart, periodEnd
	});
	const today = now - (now % day);
	const caps = [
		cap('Slack', '/Applications/Slack.app', 5 * GB, 'week', 'notify', 4.3 * GB, 'warn80', 'none', week, week + 7 * day),
		cap('Telegram', '/Applications/Telegram.app', 1 * GB, 'day', 'offerBlock', 1.02 * GB, 'blocked', 'taken', today, today + day),
		opts.capOffer
			? cap('Discord', '/Applications/Discord.app', 2 * GB, 'month', 'offerBlock', 2.05 * GB, 'over', 'pending', start, start + 30 * day)
			: cap('Discord', '/Applications/Discord.app', 2 * GB, 'month', 'offerBlock', 0.8 * GB, 'ok', 'none', start, start + 30 * day)
	];

	const cbs = {}; const listeners = {}; let nextId = 1;
	const emit = (ev, payload) => (listeners[ev] || []).forEach((h) => cbs[h] && cbs[h]({ event: ev, id: 0, payload }));
	window.__demoEmit = emit;

	const handlers = {
		app_info: () => ({ name: opts.name, brand: 'Apzur', version: opts.version, updatesConfigured: false, feedbackConfigured: false, protocols: opts.protocols }),
		new_profile: (a) => {
			const base = { id: '', protocol: a.protocol, name: '', server: '', port: 443, group: null, favorite: false, subscriptionId: null };
			const settings = {
				vless: { uuid: '', security: 'tls', flow: null, packetEncoding: null, transport: tcp, tls: tls() },
				trojan: { password: '', transport: tcp, tls: tls() },
				shadowsocks: { method: 'aes-256-gcm', password: '' },
				hysteria2: { password: '', obfs: null, tls: tls({ fingerprint: null }) },
				vmess: { uuid: '', security: 'auto', tlsEnabled: false, transport: tcp, tls: tls() },
				tuic: { uuid: '', password: '', congestionControl: 'bbr', udpRelayMode: 'native', zeroRtt: false, tls: tls({ fingerprint: null, alpn: ['h3'] }) },
				hysteria: { auth: '', upMbps: 10, downMbps: 50, obfs: null, tls: tls({ fingerprint: null, alpn: ['hysteria'] }) },
				ssh: { user: '', password: '', privateKey: '', passphrase: '', hostKeys: [] }
			}[a.protocol];
			const port = { shadowsocks: 8388, ssh: 22 }[a.protocol] ?? 443;
			return { ...base, port, settings };
		},
		snapshot: () => snapshot,
		app_update_settings: () => ({ policy: 'ask', lastCheck: 0 }),
		test_all: () => profiles.map((p) => ({ id: p.id, ms: pings[p.id] })),
		ping_profile: (a) => pings[a.id] ?? 80,
		latency_history: () => Object.fromEntries(profiles.map((p) => [p.id, Array.from({ length: 12 }, () => Math.round(pings[p.id] * (0.85 + 0.3 * rnd())))])),
		test_connection: () => 38, exit_ip: () => '203.0.113.42',
		core_status: () => coreStatus, core_check: () => releases,
		usage_get_settings: () => ({ paused: false, recordApps: true, recordDomains: true, detailRetentionDays: 30, quotaBytes: 500 * 1024 * MB, cycleDay: 1, quotaAction: 'notify', alertCycle: 0, alertLevel: 0, appCaps: caps.map(({ path, limitBytes, period, atLimit }) => ({ path, limitBytes, period, atLimit })) }),
		caps_list: () => caps,
		// Made-up icons (a coloured tile with the initial), so no real app's artwork is shown.
		app_icon: (a) => {
			const name = a.path.split('/').pop().replace(/\.app$/, '');
			const hue = [...name].reduce((h, c) => (h * 31 + c.charCodeAt(0)) % 360, 7);
			const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><defs><linearGradient id="g" x2="0" y2="1"><stop offset="0" stop-color="hsl(${hue} 70% 62%)"/><stop offset="1" stop-color="hsl(${hue} 65% 46%)"/></linearGradient></defs><rect x="3" y="3" width="42" height="42" rx="10" fill="url(#g)"/><text x="24" y="31" font-family="-apple-system,Helvetica" font-size="20" font-weight="600" fill="#fff" text-anchor="middle">${name.charAt(0).toUpperCase()}</text></svg>`;
			return 'data:image/svg+xml;base64,' + btoa(svg);
		},
		current_network: () => ({ id: 'gw-home', name: 'Home Wi-Fi' }),
		root_service_installed: () => true,
		set_routing: (a) => a.settings,
		set_reliability: () => false,
		usage_quota: () => ({ limit: 500 * 1024 * MB, used: sum.down + sum.up, up: sum.up, down: sum.down, cycleStart: start, cycleEnd: start + 30 * day, percent: ((sum.down + sum.up) / (500 * 1024 * MB)) * 100 }),
		usage_report: () => ({ series, total: sum, previous: null, peak: series.reduce((a, b) => (b.down + b.up > a.down + a.up ? b : a)), connectedSecs: 86400 * 11 }),
		usage_top: (a) => (a.args.kind === 'app' ? topApps : a.args.kind === 'domain' ? topDomains : topProfiles),
		// Byte counters grow between polls, so the page measures live speeds.
		connections_list: () => {
			conns.forEach((c, i) => {
				const rate = [520, 1900, 260, 12, 90, 340, 60, 1400][i] * 1024;
				c.down += Math.round(rate * 2 * (0.7 + 0.6 * rnd()));
				c.up += Math.round(rate * 0.25 * (0.6 + 0.8 * rnd()));
			});
			return conns.map((c) => ({ ...c }));
		},
		installed_apps: () => apps,
		export_backup: () => '~/Downloads/kage-vpn-backup-demo.json',
		run_diagnostics: () => ({
			// In full-system (TUN) mode the system route exits through the tunnel too, so both match.
			system: { ip: '203.0.113.42', country: 'Germany', city: 'Frankfurt', org: 'Example Hosting GmbH' },
			tunnel: { ip: '203.0.113.42', country: 'Germany', city: 'Frankfurt', org: 'Example Hosting GmbH' },
			dns: { ip: '203.0.113.53', geo: 'Germany' }
		}),
		rule_providers: () => [
			{ id: 'sagernet', name: 'SagerNet', geoip: 'x{name}', geosite: 'x{name}', note: "The sing-box authors' own lists: countries and V2Ray's domain categories." },
			{ id: 'metacubex', name: 'MetaCubeX', geoip: 'x{name}', geosite: 'x{name}', note: 'meta-rules-dat, the lists Clash Meta uses. Many more categories than SagerNet.' },
			{ id: 'loyalsoldier', name: 'Loyalsoldier (IP lists)', geoip: 'x{name}', geosite: '', note: 'Enhanced country IP lists.' }
		],
		rule_sets_status: () => ({
			downloadedFrom: 'sagernet', throughTunnel: !!opts.connected,
			lists: [
				{ tag: 'geoip-lk', kind: 'geoip', name: 'lk', present: true, bytes: 9400, modified: now - 86400 * 2 },
				{ tag: 'geosite-category-ads-all', kind: 'geosite', name: 'category-ads-all', present: true, bytes: 61000, modified: now - 86400 * 2 },
				{ tag: 'geosite-netflix', kind: 'geosite', name: 'netflix', present: false, bytes: 0, modified: null }
			]
		}),
		run_speed_test: async () => {
			const steps = [['ping', 0, 0.5], ['download', 120, 0.3], ['download', 340, 0.7], ['download', 412, 1], ['upload', 60, 0.5], ['upload', 96, 1]];
			for (const [phase, mbps, progress] of steps) {
				emit('speed-progress', { phase, mbps, progress, pingMs: 41, jitterMs: 3 });
				await new Promise((r) => setTimeout(r, 120));
			}
			emit('speed-progress', { phase: 'done', mbps: 412, progress: 1, pingMs: 41, jitterMs: 3 });
			return { downMbps: 412, upMbps: 96, pingMs: 41, jitterMs: 3 };
		}
	};

	window.__TAURI_INTERNALS__ = {
		metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main', windowLabel: 'main' } },
		transformCallback: (cb) => { const i = nextId++; cbs[i] = cb; return i; },
		unregisterCallback: (i) => { delete cbs[i]; },
		convertFileSrc: (x) => x,
		invoke: async (cmd, args) => {
			if (cmd === 'plugin:event|listen') { (listeners[args.event] = listeners[args.event] || []).push(args.handler); return args.handler; }
			if (cmd.startsWith('plugin:')) return null;
			const handler = handlers[cmd];
			return handler ? handler(args || {}) : null;
		}
	};
}
