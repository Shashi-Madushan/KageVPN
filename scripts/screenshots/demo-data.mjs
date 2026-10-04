// Fake backend for the screenshots in the README. Everything here is made up: documentation-range
// IP addresses (RFC 5737), example.com hosts and a placeholder UUID. No real server or account.

/** Returns JavaScript that installs a fake Tauri backend before the app loads. */
export function demoScript({ connected = true, withServers = true, name = 'Apzur VPN', version = '0.1.0' } = {}) {
	return `(${install.toString()})(${JSON.stringify({ connected, withServers, name, version })});`;
}

function install(opts) {
	const MB = 1048576;
	const now = Math.floor(Date.now() / 1000);
	let seed = 7;
	const rnd = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
	const UUID = '11111111-2222-4333-8444-555555555555';

	const city = (id, name, host, group, extra = {}) => ({
		id, name, server: host, port: 443, uuid: UUID, security: 'reality', sni: 'www.example.org', fingerprint: 'chrome',
		alpn: [], flow: 'xtls-rprx-vision', pinnedKey: null, transport: { kind: 'tcp', path: null, host: null, serviceName: null },
		reality: { publicKey: 'DEMOPUBLICKEYDEMOPUBLICKEYDEMOPUBLICKEY00', shortId: 'a1b2c3d4' }, packetEncoding: null,
		group, favorite: false, subscriptionId: null, ...extra
	});
	const profiles = opts.withServers
		? [
				city('p1', 'Frankfurt, Germany', 'de1.example.com', 'Europe', { favorite: true }),
				city('p2', 'Amsterdam, Netherlands', 'nl1.example.com', 'Europe'),
				city('p3', 'London, United Kingdom', 'uk1.example.com', 'Europe'),
				city('p4', 'New York, United States', 'us1.example.com', 'Americas', { favorite: true }),
				city('p5', 'Singapore', 'sg1.example.com', 'Asia'),
				city('p6', 'Tokyo, Japan', 'jp1.example.com', 'Asia'),
				city('p7', 'Sydney, Australia', 'au1.example.com', 'Oceania')
			]
		: [];
	const pings = { p1: 38, p2: 41, p3: 52, p4: 118, p5: 164, p6: 187, p7: 242 };

	const history = Array.from({ length: 60 }, (_, i) => ({
		down: Math.round((900000 + 700000 * Math.sin(i / 6) + 500000 * rnd()) * (i > 40 ? 1.6 : 1)),
		up: Math.round(90000 + 60000 * rnd())
	}));

	const status = opts.connected
		? { state: 'connected', mode: 'tun', since: now - 5400, error: null }
		: { state: 'disconnected', mode: 'tun', since: null, error: null };
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
		settings: { showSpeed: true, autoConnect: false }, launchAtLogin: true, auto: { enabled: false, group: null },
		subscriptions: [{ id: 's1', name: 'Example provider', url: 'https://example.com/sub', intervalHours: 24, lastUpdated: now - 7200, lastError: null, userInfo: { upload: 8e9, download: 41e9, total: 200e9, expire: now + 86400 * 18 } }],
		routing: {
			mode: 'rule', bypassLan: true, finalAction: 'proxy',
			rules: [
				{ id: 'r1', enabled: true, kind: 'domainSuffix', value: 'netflix.com', action: 'proxy' },
				{ id: 'r2', enabled: true, kind: 'processName', value: 'Spotify', action: 'direct' },
				{ id: 'r3', enabled: true, kind: 'domainKeyword', value: 'doubleclick', action: 'block' },
				{ id: 'r4', enabled: true, kind: 'ipCidr', value: '192.168.50.0/24', action: 'direct' }
			],
			ruleSets: [{ id: 'g1', enabled: true, kind: 'geoip', country: 'lk', action: 'direct' }],
			dns: { provider: 'cloudflare', custom: '' }
		},
		reliability: { autoReconnect: true, watchdog: true, killSwitch: false, hotkey: true }, hotkey: '⌥⌘V'
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
		active: { path: '~/Library/Application Support/com.apzur.vpn/core/sing-box/1.12.9/sing-box', version: '1.12.9', tags: ['with_clash_api', 'with_utls'], source: 'managed', missingTags: [], outdated: false },
		error: null, system: null,
		installed: [{ version: '1.12.9', path: '/x', installedAt: now - 5 * day }, { version: '1.12.8', path: '/y', installedAt: now - 19 * day }],
		settings: { managedVersion: '1.12.9', channel: 'stable', autoCheck: true, lastCheck: now - 3600, updatePolicy: 'ask', pendingVersion: null }, busy: false
	};

	const conns = [
		['www.youtube.com', '142.250.0.1', '443', 'Safari', 'tcp', 18 * MB, 0.6 * MB, 'proxy', 'domain_suffix: youtube.com'],
		['netflix.com', '198.51.100.20', '443', 'Safari', 'tcp', 42 * MB, 1 * MB, 'proxy', 'domain_suffix: netflix.com'],
		['api.spotify.com', '198.51.100.31', '443', 'Spotify', 'tcp', 6 * MB, 0.2 * MB, 'direct', 'process_name: Spotify'],
		['github.com', '140.82.0.4', '22', 'ssh', 'tcp', 0.4 * MB, 0.3 * MB, 'proxy', 'final'],
		['slack-edge.com', '198.51.100.44', '443', 'Slack Helper', 'tcp', 2.2 * MB, 0.9 * MB, 'proxy', 'final']
	].map((c, i) => ({ id: 'c' + i, host: c[0], destinationIp: c[1], destinationPort: c[2], process: c[3], network: c[4], down: Math.round(c[5]), up: Math.round(c[6]), start: new Date((now - 120 - i * 300) * 1000).toISOString(), chain: c[7], rule: c[8] }));

	const cbs = {}; const listeners = {}; let nextId = 1;
	const emit = (ev, payload) => (listeners[ev] || []).forEach((h) => cbs[h] && cbs[h]({ event: ev, id: 0, payload }));
	window.__demoEmit = emit;

	const handlers = {
		app_info: () => ({ name: opts.name, brand: 'Apzur', version: opts.version, updatesConfigured: false, feedbackConfigured: false }),
		snapshot: () => snapshot,
		app_update_settings: () => ({ policy: 'ask', lastCheck: 0 }),
		test_all: () => profiles.map((p) => ({ id: p.id, ms: pings[p.id] })),
		ping_profile: (a) => pings[a.id] ?? 80,
		latency_history: () => Object.fromEntries(profiles.map((p) => [p.id, Array.from({ length: 12 }, () => Math.round(pings[p.id] * (0.85 + 0.3 * rnd())))])),
		test_connection: () => 38, exit_ip: () => '203.0.113.42',
		core_status: () => coreStatus, core_check: () => releases,
		usage_get_settings: () => ({ paused: false, recordApps: true, recordDomains: true, detailRetentionDays: 30, quotaBytes: 500 * 1024 * MB, cycleDay: 1, quotaAction: 'notify', alertCycle: 0 }),
		usage_quota: () => ({ limit: 500 * 1024 * MB, used: sum.down + sum.up, up: sum.up, down: sum.down, cycleStart: start, cycleEnd: start + 30 * day, percent: ((sum.down + sum.up) / (500 * 1024 * MB)) * 100 }),
		usage_report: () => ({ series, total: sum, previous: null, peak: series.reduce((a, b) => (b.down + b.up > a.down + a.up ? b : a)), connectedSecs: 86400 * 11 }),
		usage_top: (a) => (a.args.kind === 'app' ? topApps : a.args.kind === 'domain' ? topDomains : topProfiles),
		connections_list: () => conns,
		run_diagnostics: () => ({
			// In full-system (TUN) mode the system route exits through the tunnel too, so both match.
			system: { ip: '203.0.113.42', country: 'Germany', city: 'Frankfurt', org: 'Example Hosting GmbH' },
			tunnel: { ip: '203.0.113.42', country: 'Germany', city: 'Frankfurt', org: 'Example Hosting GmbH' },
			dns: { ip: '203.0.113.53', geo: 'Germany' }
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
