import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { capsApi, usageApi, type AppCap, type CapStatus } from './api';
import { app } from './app.svelte';
import { canBlock } from './capView';

/** Per-app data caps as the backend reports them. Refreshes only while something shows them. */
class CapsStore {
	list = $state<CapStatus[]>([]);
	/** The offers waiting for a decision (Block now or Not now). */
	pending = $derived(this.list.filter(canBlock));

	private users = 0;
	private timer: ReturnType<typeof setInterval> | undefined;
	private unlisten: UnlistenFn | undefined;

	byPath(path: string): CapStatus | undefined {
		return this.list.find((c) => c.path === path);
	}

	/** Starts refreshing; call the returned function when the component goes away. */
	use(): () => void {
		this.users++;
		if (this.users === 1) {
			this.load();
			this.timer = setInterval(() => this.load(), 5000);
			listen('caps-changed', () => this.refresh()).then((un) => (this.users > 0 ? (this.unlisten = un) : un()));
		}
		return () => {
			this.users = Math.max(0, this.users - 1);
			if (this.users > 0) return;
			clearInterval(this.timer);
			this.unlisten?.();
			this.unlisten = undefined;
		};
	}

	async load() {
		try {
			this.list = await capsApi.list();
		} catch {
			// Keep what is shown; the next refresh tries again.
		}
	}

	/** After a change: the caps, and the routing a cap block may have edited. */
	async refresh() {
		await Promise.all([this.load(), app.reloadRouting()]);
	}

	private async act(action: () => Promise<void>): Promise<boolean> {
		try {
			await action();
		} catch (e) {
			app.notify(String(e));
			return false;
		}
		await this.refresh();
		return true;
	}

	save = (cap: AppCap) => this.act(() => capsApi.set(cap));
	remove = (path: string) => this.act(() => capsApi.remove(path));
	block = (path: string) => this.act(() => capsApi.block(path));
	notNow = (path: string) => this.act(() => capsApi.notNow(path));
	unblock = (path: string) => this.act(() => capsApi.unblock(path));

	/** Caps count only while app usage is recorded. */
	turnOnRecording = () =>
		this.act(async () => {
			const settings = await usageApi.settings();
			await usageApi.setSettings({ ...settings, recordApps: true, paused: false });
		});
}

export const caps = new CapsStore();
