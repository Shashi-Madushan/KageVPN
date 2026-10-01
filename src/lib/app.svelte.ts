import { listen } from '@tauri-apps/api/event';
import {
	api,
	type Hint,
	type LogLine,
	type Mode,
	type Profile,
	type Sample,
	type Settings,
	type Status,
	type Traffic
} from './api';

const MAX_LOG_LINES = 2000;
const HISTORY_LENGTH = 60;
const TOAST_MS = 5000;

export type Latency = { ms: number } | { failed: true } | { pending: true };
export type Toast = { id: number; message: string; kind: 'error' | 'info' };

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
	latency = $state<Record<string, Latency>>({});
	exitIp = $state<string | null>(null);
	delayMs = $state<number | null>(null);
	toasts = $state<Toast[]>([]);

	activeProfile = $derived(this.profiles.find((p) => p.id === this.activeId) ?? null);
	busy = $derived(this.status.state === 'connecting');
	connected = $derived(this.status.state === 'connected');
	locked = $derived(this.busy || this.connected);
	hint = $derived<Hint | null>(this.logs.findLast((l) => l.hint)?.hint ?? null);

	private toastId = 0;

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
		if (snap.status.state === 'connected') this.verify();

		await listen<Status>('status', (e) => this.applyStatus(e.payload));
		await listen<LogLine[]>('logs', (e) => {
			this.logs = [...this.logs, ...e.payload].slice(-MAX_LOG_LINES);
		});
		await listen<Traffic>('traffic', (e) => this.applyTraffic(e.payload));
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

	notify(message: string, kind: Toast['kind'] = 'error') {
		const id = ++this.toastId;
		this.toasts = [...this.toasts, { id, message, kind }];
		setTimeout(() => this.dismiss(id), TOAST_MS);
	}

	dismiss(id: number) {
		this.toasts = this.toasts.filter((t) => t.id !== id);
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

	async importLink(link: string): Promise<string | null> {
		try {
			const profile = await api.importProfile(link);
			this.profiles = [...this.profiles, profile];
			this.activeId ??= profile.id;
			return null;
		} catch (e) {
			return String(e);
		}
	}

	async select(id: string) {
		if (this.locked) return;
		await this.run(() => api.selectProfile(id));
		this.activeId = id;
	}

	async remove(id: string) {
		await this.run(() => api.deleteProfile(id));
		this.profiles = this.profiles.filter((p) => p.id !== id);
		if (this.activeId === id) this.activeId = this.profiles[0]?.id ?? null;
	}

	async rename(id: string, name: string) {
		const profile = this.profiles.find((p) => p.id === id);
		if (!profile || !name.trim() || name === profile.name) return;
		await this.run(() => api.renameProfile(id, name));
		profile.name = name.trim();
	}

	async setMode(mode: Mode) {
		if (this.locked) return;
		await this.run(() => api.setMode(mode));
		this.mode = mode;
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

	async ping(id: string) {
		this.latency[id] = { pending: true };
		try {
			this.latency[id] = { ms: await api.pingProfile(id) };
		} catch {
			this.latency[id] = { failed: true };
		}
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
