import { defineConfig } from 'vite';
import base from './vite.config.js';

const mock = `
(() => {
  const q = new URLSearchParams(location.search).get('s') || 'connected';
  const now = Math.floor(Date.now() / 1000);
  const on = q === 'connected' || q === 'down';
  const st = {
    state: q === 'connecting' ? 'connecting' : on ? 'connected' : 'disconnected',
    mode: 'tun', since: on ? now - 5402 : null, error: null,
    health: q === 'down' ? 'down' : on ? 'healthy' : 'idle',
    reason: q === 'down' ? { cause: 'noTraffic', message: 'Traffic is not getting through the tunnel.' } : null,
    latencyMs: on ? 38 : null, failure: null, proxyPort: 2080, blocked: false, disconnecting: false
  };
  const hist = Array.from({ length: 60 }, (_, i) => ({ down: on ? 900000 + 800000 * Math.sin(i / 6) ** 2 + (i > 40 ? 900000 : 0) : 0, up: on ? 60000 + 30000 * Math.sin(i / 3) ** 2 : 0 }));
  const profile = { id: 'p1', protocol: 'vless', name: '\\u{1F1E9}\\u{1F1EA} Frankfurt, Germany', server: 'de1.example.net', port: 443, settings: {}, group: null, favorite: false, subscriptionId: 's1', summary: 'Reality' };
  const snap = {
    profiles: [profile, { ...profile, id: 'p2', name: '\\u{1F1F8}\\u{1F1EC} Singapore', server: 'sg1.example.net' }], activeId: 'p1', mode: 'tun', status: st, logs: [],
    traffic: { up: on ? 94000 : 0, down: on ? 1700000 : 0, totalUp: 74e6, totalDown: 612e6 }, history: hist,
    settings: { showSpeed: true, autoConnect: false, logLevel: 'normal', clearLogs: false }, launchAtLogin: false,
    auto: { enabled: false, group: null },
    subscriptions: [{ id: 's1', name: 'Demo', url: 'https://example.net/sub', intervalHours: 12, lastUpdated: now, lastError: null, userInfo: { upload: 5e9, download: 40e9, total: 186e9, expire: now + 18 * 86400 }, userAgent: null, sendDeviceId: false }],
    routing: { mode: 'global', bypassLan: true, ipv6: false, finalAction: 'proxy', rules: [], ruleSets: [], ruleProvider: 'sagernet', customProviders: [], dns: { provider: 'cloudflare', custom: '', fakeIp: false } },
    reliability: { autoReconnect: true, watchdog: true, killSwitch: true, hotkey: false, proxyAuth: false, proxyPassword: '', systemProxy: false, bypass: [] },
    networks: { enabled: false, networks: [] }, hotkey: ''
  };
  const info = { name: 'Kage', brand: 'Apzur', version: '0.3.0', updatesConfigured: true, feedbackConfigured: false, protocols: [{ id: 'vless', label: 'VLESS', schemes: ['vless'], fields: [] }] };
  const win = new URLSearchParams(location.search).get('os') === 'windows';
  const platform = win
    ? { os: 'windows', tun: false, killSwitch: false, rootService: false, systemProxy: true, appNap: false, speedInTray: false }
    : { os: 'macos', tun: true, killSwitch: true, rootService: true, systemProxy: true, appNap: true, speedInTray: true };
  const rows = { app_info: info, platform, snapshot: snap, exit_ip: '203.0.113.42' };
  let n = 0;
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main', windowLabel: 'main' } },
    transformCallback(cb) { const id = ++n; window['_' + id] = cb; return id; },
    unregisterCallback() {},
    convertFileSrc: (p) => p,
    invoke(cmd) {
      if (cmd === 'plugin:event|listen') return Promise.resolve(++n);
      if (cmd in rows) return Promise.resolve(rows[cmd]);
      return Promise.reject('mock: ' + cmd);
    }
  };
})();
`;

export default defineConfig(() => {
	const cfg = base();
	return {
		...cfg,
		server: { ...cfg.server, port: 1431 },
		plugins: [...cfg.plugins, { name: 'tauri-mock', transform(code, id) { if (id.endsWith('/src/lib/api.ts')) return mock + code; } }
		]
	};
});
