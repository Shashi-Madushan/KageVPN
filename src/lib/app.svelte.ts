import { listen } from '@tauri-apps/api/event';
import {
	api,
	type AutoSettings,
	type Hint,
	type ImportSummary,
	type LogLine,
	type Mode,
	type Profile,
	type ReliabilitySettings,
	type RoutingSettings,
	type Sample,
	type Settings,
	type Status,
	type Subscription,
	type Traffic
} from './api';

const MAX_LOG_LINES = 2000;
const HISTORY_LENGTH = 60;

export type Latency = { ms: number } | { failed: true } | { pending: true };
export type NoticeKind = 'error' | 'info';

const NO_TRAFFIC: Traffic = { up: 0, down: 0, totalUp: 0, totalDown: 0 };

class AppStore {
	profiles = $state<Profile[]>([]);
	activeId = $state<string | null>(null);
	mode = $state<Mode>('tun');
	status = $state<Status>({ state: 'disconnected', mode: 'tun', since: null, error: null });
	logs = $state<LogLine[]>([]);
	traffic = $state<Traffic>(NO_TRAFFIC);
	history = $state<Sample[]>([]);
	settings = $state<Settings>({ showSpeed: true, autoConnect: false });
	launchAtLogin = $state(false);
	auto = $state<AutoSettings>({ enabled: false, group: null });
	subscriptions = $state<Subscription[]>([]);
	routing = $state<RoutingSettings | null>(null);
	reliability = $state<ReliabilitySettings>({ autoReconnect: true, watchdog: false, killSwitch: false, hotkey: false });
	hotkey = $state('');
	latency = $state<Record<string, Latency>>({});
	latencyHistory = $state<Record<string, (number | null)[]>>({});
	exitIp = $state<string | null>(null);
	delayMs = $state<number | null>(null);
	pendingImport = $state<string | null>(null);
	/** True once the first snapshot from the backend has been applied. */
	loaded = $state(false);

	activeProfile = $derived(this.profiles.find((p) => p.id === this.activeId) ?? null);
	busy = $derived(this.status.state === 'connecting');
	connected = $derived(this.status.state === 'connected');
	locked = $derived(this.busy || this.connected);
	groups = $derived([...new Set(this.profiles.map((p) => p.group).filter((g): g is string => !!g))].sort());
	hint = $derived<Hint | null>(this.logs.findLast((l) => l.hint)?.hint ?? null);

	async init() {
		const snap = await api.snapshot();
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
		const title = kind === 'error' ? 'SingBox Error' : 'SingBox GUI';
		api.notifyOs(title, message).catch(() => null);
	}

	private async run<T>(action: () => Promise<T>): Promise<T | undefined> {
		try {
			return await action();
		} catch (e) {
			this.notify(String(e));
		}
	}

	async toggle() {
		if (this.busy) return;
		await this.run(this.connected ? api.disconnect : api.connect);
	}

	async verify() {
		try {
			this.delayMs = await api.testConnection();
			this.exitIp = await api.exitIp();
		} catch {
			this.delayMs = null;
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
		await this.run(() => api.selectProfile(id));
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
		await this.run(() => api.renameProfile(id, name));
		profile.name = name.trim();
	}

	async toggleFavorite(id: string) {
		const profile = this.profiles.find((p) => p.id === id);
		if (!profile) return;
		await this.run(() => api.setFavorite(id, !profile.favorite));
		profile.favorite = !profile.favorite;
	}

	async setGroup(id: string, group: string | null) {
		const profile = this.profiles.find((p) => p.id === id);
		if (!profile) return;
		await this.run(() => api.setGroup(id, group));
		profile.group = group?.trim() || null;
	}

	async setMode(mode: Mode) {
		if (this.locked) return;
		await this.run(() => api.setMode(mode));
		this.mode = mode;
	}

	async setAuto(auto: AutoSettings) {
		await this.run(() => api.setAuto(auto));
		this.auto = auto;
		if (this.connected) this.notify('Reconnect to apply the auto setting.', 'info');
	}

	async updateSettings(patch: Partial<Settings>) {
		const next = { ...this.settings, ...patch };
		await this.run(() => api.updateSettings(next));
		this.settings = next;
	}

	async setLaunchAtLogin(enabled: boolean) {
		await this.run(() => api.setLaunchAtLogin(enabled));
		this.launchAtLogin = enabled;
	}

	async saveReliability(patch: Partial<ReliabilitySettings>) {
		const next = { ...this.reliability, ...patch };
		await this.run(() => api.setReliability(next));
		this.reliability = next;
	}

	async saveRouting(next: RoutingSettings) {
		await this.run(() => api.setRouting(next));
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

	async addSubscription(name: string, url: string): Promise<string | null> {
		try {
			const summary = await api.addSubscription(name, url);
			await this.reloadSnapshot();
			this.notify(`Added ${summary.total} profiles.`, 'info');
			return null;
		} catch (e) {
			return String(e);
		}
	}

	async refreshSubscription(id: string) {
		const summary = await this.run(() => api.refreshSubscription(id));
		await this.reloadSnapshot();
		if (summary) this.notify(`Updated: ${summary.added} new, ${summary.removed} removed.`, 'info');
	}

	async removeSubscription(id: string) {
		await this.run(() => api.removeSubscription(id));
		await this.reloadSnapshot();
	}

	async trustHintedKey(key: string) {
		if (!this.activeId) return;
		const id = this.activeId;
		await this.run(() => api.trustKey(id, key));
		const profile = this.profiles.find((p) => p.id === id);
		if (profile) profile.pinnedKey = key;
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
