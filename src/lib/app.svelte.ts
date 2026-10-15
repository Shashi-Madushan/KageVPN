import { listen } from '@tauri-apps/api/event';
import { setPath } from './path';
import { skippedReason } from './format';
import { protocols } from './protocols.svelte';
import {
	api,
	type AppInfo,
	type AutoSettings,
	type Hint,
	type ImportSummary,
	type LogLine,
	type Mode,
	type Profile,
	type ReliabilitySettings,
	type RouteAction,
	type RoutingSettings,
	type Rule,
	type Sample,
	type Settings,
	type Status,
	type Subscription,
	type SubscriptionOptions,
	type Traffic
} from './api';

const MAX_LOG_LINES = 2000;
/** A second press this soon after Connect is a double-click, not a request to cancel. */
const CANCEL_GUARD_MS = 1200;
const HISTORY_LENGTH = 60;

export type Latency = { ms: number } | { failed: true } | { pending: true };
type NoticeKind = 'error' | 'info';

const NO_TRAFFIC: Traffic = { up: 0, down: 0, totalUp: 0, totalDown: 0 };

class AppStore {
	profiles = $state<Profile[]>([]);
	activeId = $state<string | null>(null);
	mode = $state<Mode>('tun');
	status = $state<Status>({ state: 'disconnected', mode: 'tun', since: null, error: null, health: 'idle', reason: null, latencyMs: null, failure: null, proxyPort: null, blocked: false });
	logs = $state<LogLine[]>([]);
	traffic = $state<Traffic>(NO_TRAFFIC);
	history = $state<Sample[]>([]);
	settings = $state<Settings>({ showSpeed: true, autoConnect: false, logLevel: 'normal', clearLogs: false });
	launchAtLogin = $state(false);
	auto = $state<AutoSettings>({ enabled: false, group: null });
	subscriptions = $state<Subscription[]>([]);
	routing = $state<RoutingSettings | null>(null);
	reliability = $state<ReliabilitySettings>({ autoReconnect: true, watchdog: true, killSwitch: false, hotkey: false, proxyAuth: false, proxyPassword: '' });
	hotkey = $state('');
	latency = $state<Record<string, Latency>>({});
	latencyHistory = $state<Record<string, (number | null)[]>>({});
	exitIp = $state<string | null>(null);
	delayMs = $state<number | null>(null);
	/** Result of the Ping button: through the tunnel when connected, straight to the server otherwise. */
	check = $state<Latency | undefined>(undefined);
	/** What `check` measured, so a result is not shown after the connection state changed. */
	checkVia = $state<'tunnel' | 'server'>('server');
	pendingImport = $state<string | null>(null);
	private connectingSince = 0;
	/** True once the first snapshot from the backend has been applied. */
	loaded = $state(false);
	/** Name, version and feature switches; the source of truth is `app_config.rs`. */
	info = $state<AppInfo>({ name: '', brand: '', version: '', updatesConfigured: false, feedbackConfigured: false, protocols: [] });

	activeProfile = $derived(this.profiles.find((p) => p.id === this.activeId) ?? null);
	busy = $derived(this.status.state === 'connecting');
	connected = $derived(this.status.state === 'connected');
	/** Connected and proven to carry traffic. */
	protectedNow = $derived(this.status.state === 'connected' && this.status.health === 'healthy');
	/** Connected, but the tunnel does not pass traffic. */
	tunnelDown = $derived(this.status.state === 'connected' && this.status.health === 'down');
	verifying = $derived(this.status.state === 'connected' && this.status.health === 'verifying');
	locked = $derived(this.busy || this.connected);
	groups = $derived([...new Set(this.profiles.map((p) => p.group).filter((g): g is string => !!g))].sort());
	hint = $derived<Hint | null>(this.logs.findLast((l) => l.hint)?.hint ?? null);

	async init() {
		const [info, snap] = await Promise.all([api.appInfo(), api.snapshot()]);
		this.info = info;
		protocols.set(this.info.protocols);
		this.profiles = snap.profiles;
		this.activeId = snap.activeId;
		this.mode = snap.mode;
		this.status = snap.status;
		this.logs = snap.logs;
		this.traffic = snap.traffic;
		this.history = snap.history;
		this.settings = snap.settings;
		this.launchAtLogin = snap.launchAtLogin;
		this.auto = snap.auto;
		this.subscriptions = snap.subscriptions;
		this.routing = snap.routing;
		this.reliability = snap.reliability;
		this.hotkey = snap.hotkey;
		this.loaded = true;
		if (snap.status.state === 'connected') this.verify();
		this.loadLatencyHistory();

		await listen<Status>('status', (e) => this.applyStatus(e.payload));
		await listen<LogLine[]>('logs', (e) => {
			this.logs = [...this.logs, ...e.payload].slice(-MAX_LOG_LINES);
		});
		await listen<Traffic>('traffic', (e) => this.applyTraffic(e.payload));
		await listen<string>('import-request', (e) => (this.pendingImport = e.payload));
	}

	async reloadSnapshot() {
		const snap = await api.snapshot();
		this.profiles = snap.profiles;
		this.activeId = snap.activeId;
		this.subscriptions = snap.subscriptions;
		this.auto = snap.auto;
	}

	private applyStatus(next: Status) {
		const previous = this.status.state;
		this.status = next;
		if (previous !== 'connecting' && next.state === 'connecting') this.connectingSince = Date.now();
		if (previous !== 'connecting' && next.state === 'connecting') this.logs = [];
		if (previous !== 'connected' && next.state === 'connected') this.verify();
		if (next.state !== 'connected') {
			this.exitIp = null;
			this.delayMs = null;
			this.history = [];
			this.traffic = NO_TRAFFIC;
		}
		if (next.error) this.notify(next.error);
	}

	private applyTraffic(next: Traffic) {
		this.traffic = next;
		if (!this.connected) return;
		this.history = [...this.history, { up: next.up, down: next.down }].slice(-HISTORY_LENGTH);
	}

	notify(message: string, kind: NoticeKind = 'error') {
		const title = kind === 'error' ? `${this.info.name} Error` : this.info.name;
		api.notifyOs(title, message).catch(() => null);
	}

	private async run<T>(action: () => Promise<T>): Promise<T | undefined> {
		try {
			return await action();
		} catch (e) {
			this.notify(String(e));
		}
	}

	/**
	 * Runs an action that changes saved state. Returns false, after telling the user, when it failed,
	 * so the screen never shows a change the app did not keep.
	 */
	private async attempt(action: () => Promise<unknown>): Promise<boolean> {
		try {
			await action();
			return true;
		} catch (e) {
			this.notify(String(e));
			return false;
		}
	}

	async toggle() {
		if (this.busy) {
			// Still connecting: pressing again cancels, unless it is the tail of a double-click.
			if (Date.now() - this.connectingSince > CANCEL_GUARD_MS) await this.run(api.disconnect);
			return;
		}
		if (!this.connected) {
			// Without an engine connecting can only fail, so offer the download instead.
			const { core } = await import('./core.svelte');
			if (core.missing) {
				core.promptInstall = true;
				return;
			}
		}
		await this.run(this.connected ? api.disconnect : api.connect);
	}

	/** Drops the tunnel and brings it back up. */
	async restartTunnel() {
		if (!(await this.attempt(api.disconnect))) return;
		await this.run(api.connect);
	}

	/** Moves to the next server and reconnects; for when the current one is not passing traffic. */
	async switchServer() {
		if (this.profiles.length < 2) return;
		const index = this.profiles.findIndex((p) => p.id === this.activeId);
		const next = this.profiles[(index + 1) % this.profiles.length];
		if (!(await this.attempt(api.disconnect))) return;
		if (this.auto.enabled) await this.setAuto({ ...this.auto, enabled: false });
		await this.select(next.id);
		await this.run(api.connect);
	}

	async verify() {
		try {
			this.delayMs = await api.testConnection();
			this.exitIp = await api.exitIp();
		} catch {
			this.delayMs = null;
		}
	}

	/** Checks the active server: end to end through the tunnel when connected, TCP otherwise. */
	async checkServer() {
		if (this.check && 'pending' in this.check) return;
		this.check = { pending: true };
		this.checkVia = this.connected ? 'tunnel' : 'server';
		try {
			if (this.connected) {
				this.delayMs = await api.testConnection();
				this.check = { ms: this.delayMs };
			} else if (this.activeId) {
				this.check = { ms: await api.pingProfile(this.activeId) };
			} else {
				this.check = { failed: true };
			}
		} catch {
			this.check = { failed: true };
		}
	}

	async importText(text: string): Promise<{ summary: ImportSummary | null; error: string | null }> {
		try {
			const summary = await api.importText(text);
			await this.reloadSnapshot();
			return { summary, error: null };
		} catch (e) {
			return { summary: null, error: String(e) };
		}
	}

	async saveProfile(profile: Profile): Promise<string | null> {
		try {
			await api.saveProfile(profile);
			await this.reloadSnapshot();
			return null;
		} catch (e) {
			return String(e);
		}
	}

	async select(id: string) {
		if (!(await this.attempt(() => api.selectProfile(id)))) return;
		this.activeId = id;
		if (this.connected && !this.auto.enabled) this.verify();
	}

	async remove(id: string) {
		await this.run(() => api.deleteProfile(id));
		await this.reloadSnapshot();
	}

	async duplicate(id: string) {
		await this.run(() => api.duplicateProfile(id));
		await this.reloadSnapshot();
	}

	async rename(id: string, name: string) {
		const profile = this.profiles.find((p) => p.id === id);
		if (!profile || !name.trim() || name === profile.name) return;
		if (!(await this.attempt(() => api.renameProfile(id, name)))) return;
		profile.name = name.trim();
	}

	async toggleFavorite(id: string) {
		const profile = this.profiles.find((p) => p.id === id);
		if (!profile) return;
		if (!(await this.attempt(() => api.setFavorite(id, !profile.favorite)))) return;
		profile.favorite = !profile.favorite;
	}

	async setGroup(id: string, group: string | null) {
		const profile = this.profiles.find((p) => p.id === id);
		if (!profile) return;
		if (!(await this.attempt(() => api.setGroup(id, group)))) return;
		profile.group = group?.trim() || null;
	}

	async setMode(mode: Mode) {
		if (this.locked) return;
		if (!(await this.attempt(() => api.setMode(mode)))) return;
		this.mode = mode;
	}

	async setAuto(auto: AutoSettings, quiet = false) {
		if (!(await this.attempt(() => api.setAuto(auto)))) return false;
		this.auto = auto;
		if (this.connected && !quiet) this.notify('Reconnect to apply the auto setting.', 'info');
		return true;
	}

	/**
	 * Picks a server, or `auto` for the fastest, from the menu bar. While connected a server change
	 * happens in place; switching to or from Auto needs a new tunnel, so it reconnects.
	 */
	async chooseServer(choice: string) {
		const wantAuto = choice === 'auto';
		if (wantAuto === this.auto.enabled) {
			if (!wantAuto) await this.select(choice);
			return;
		}
		const reconnect = this.connected;
		if (reconnect && !(await this.attempt(api.disconnect))) return;
		if (!(await this.setAuto({ ...this.auto, enabled: wantAuto }, true))) return;
		if (!wantAuto) await this.select(choice);
		if (reconnect) await this.run(api.connect);
	}

	async updateSettings(patch: Partial<Settings>) {
		const next = { ...this.settings, ...patch };
		if (!(await this.attempt(() => api.updateSettings(next)))) return;
		this.settings = next;
	}

	async setLaunchAtLogin(enabled: boolean) {
		if (!(await this.attempt(() => api.setLaunchAtLogin(enabled)))) return;
		this.launchAtLogin = enabled;
	}

	async saveReliability(patch: Partial<ReliabilitySettings>) {
		const next = { ...this.reliability, ...patch };
		if (!(await this.attempt(() => api.setReliability(next)))) return;
		this.reliability = next;
	}

	async saveRouting(next: RoutingSettings) {
		if (!(await this.attempt(() => api.setRouting(next)))) return;
		this.routing = next;
		this.notify(this.connected ? 'Routing saved. Reconnect to apply it.' : 'Routing saved.', 'info');
	}

	async ping(id: string) {
		this.latency[id] = { pending: true };
		try {
			this.latency[id] = { ms: await api.pingProfile(id) };
		} catch {
			this.latency[id] = { failed: true };
		}
	}

	async testAll() {
		for (const profile of this.profiles) this.latency[profile.id] = { pending: true };
		const results = await this.run(api.testAll);
		for (const profile of this.profiles) {
			const found = results?.find((r) => r.id === profile.id);
			this.latency[profile.id] = found?.ms != null ? { ms: found.ms } : { failed: true };
		}
		await this.loadLatencyHistory();
	}

	async loadLatencyHistory() {
		try {
			this.latencyHistory = await api.latencyHistory();
		} catch {
			this.latencyHistory = {};
		}
	}

	async addSubscription(name: string, url: string, options: SubscriptionOptions): Promise<string | null> {
		try {
			const summary = await api.addSubscription(name, url, options);
			await this.reloadSnapshot();
			const skipped = summary.skipped ? `, ${summary.skipped} skipped${skippedReason(summary.skippedReason)}` : '';
			this.notify(`Added ${summary.total} profiles${skipped}.`, 'info');
			return null;
		} catch (e) {
			return String(e);
		}
	}

	async refreshSubscription(id: string) {
		const summary = await this.run(() => api.refreshSubscription(id));
		await this.reloadSnapshot();
		if (summary) {
			const skipped = summary.skipped ? `, ${summary.skipped} skipped${skippedReason(summary.skippedReason)}` : '';
			this.notify(`Updated: ${summary.added} new, ${summary.removed} removed${skipped}.`, 'info');
		}
	}

	/** Saves a subscription's name, link and options, then refreshes it so the change shows. */
	async editSubscription(id: string, name: string, url: string, options: SubscriptionOptions): Promise<string | null> {
		try {
			await api.editSubscription(id, name, url, options);
		} catch (e) {
			return String(e);
		}
		await this.reloadSnapshot();
		await this.refreshSubscription(id);
		return null;
	}

	/** Removes several servers at once and forgets their test results. */
	async removeProfiles(ids: string[]) {
		const removed = await this.run(() => api.deleteProfiles(ids));
		for (const id of ids) delete this.latency[id];
		await this.reloadSnapshot();
		if (removed) this.notify(`Removed ${removed} server${removed === 1 ? '' : 's'}.`, 'info');
	}

	/**
	 * Adds a rule for one app at the top of the list, so it wins over broader rules. Global and
	 * Direct become Rules mode with the same default, which routes everything else as before.
	 */
	async routeApp(rule: Pick<Rule, 'kind' | 'value'>, action: RouteAction) {
		if (!this.routing) return false;
		const next: RoutingSettings = structuredClone($state.snapshot(this.routing));
		if (next.mode !== 'rule') {
			next.finalAction = next.mode === 'direct' ? 'direct' : 'proxy';
			next.mode = 'rule';
		}
		next.rules = next.rules.filter((r) => !(r.kind === rule.kind && r.value === rule.value));
		next.rules.unshift({ id: crypto.randomUUID(), enabled: true, kind: rule.kind, value: rule.value, action });
		if (!(await this.attempt(() => api.setRouting(next)))) return false;
		this.routing = next;
		return true;
	}

	/** Writes a backup file to Downloads and says where it went. */
	async exportBackup() {
		const path = await this.run(api.exportBackup);
		if (path) this.notify(`Backup saved to ${path}. It contains your server passwords, so keep it private.`, 'info');
	}

	async restoreBackup(text: string): Promise<string | null> {
		try {
			const summary = await api.restoreBackup(text);
			const snap = await api.snapshot();
			this.profiles = snap.profiles;
			this.activeId = snap.activeId;
			this.mode = snap.mode;
			this.settings = snap.settings;
			this.auto = snap.auto;
			this.subscriptions = snap.subscriptions;
			this.routing = snap.routing;
			this.reliability = snap.reliability;
			this.latency = {};
			const subs = summary.subscriptions ? ` and ${summary.subscriptions} subscription${summary.subscriptions === 1 ? '' : 's'}` : '';
			this.notify(`Restored ${summary.servers} server${summary.servers === 1 ? '' : 's'}${subs}.`, 'info');
			return null;
		} catch (e) {
			return String(e);
		}
	}

	async removeSubscription(id: string) {
		await this.run(() => api.removeSubscription(id));
		await this.reloadSnapshot();
	}

	async trustHintedKey(key: string) {
		if (!this.activeId) return;
		const id = this.activeId;
		if (!(await this.attempt(() => api.trustKey(id, key)))) return;
		const profile = this.profiles.find((p) => p.id === id);
		if (profile) setPath(profile.settings, 'tls.pinnedKey', key);
		this.logs = [];
		if (this.connected) await this.run(api.disconnect);
		await this.run(api.connect);
	}

	quit() {
		return api.quitApp();
	}

	async clearLogs() {
		await api.clearLogs();
		this.logs = [];
	}
}

export const app = new AppStore();
