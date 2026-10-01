import { listen } from '@tauri-apps/api/event';
import { app } from './app.svelte';
import { coreApi, type Channel, type CoreProgress, type CoreStatus, type Release } from './api';
import { compareVersions } from './version';

class CoreStore {
	status = $state<CoreStatus | null>(null);
	releases = $state<Release[]>([]);
	progress = $state<CoreProgress | null>(null);
	checking = $state(false);
	installing = $state<string | null>(null);
	checked = $state(false);

	activeVersion = $derived(this.status?.active?.version ?? null);
	missing = $derived(this.status !== null && this.status.active === null);
	newest = $derived(
		this.releases.reduce<Release | null>(
			(best, r) => (best === null || compareVersions(r.version, best.version) > 0 ? r : best),
			null
		)
	);
	updateAvailable = $derived(
		this.newest !== null && this.activeVersion !== null && compareVersions(this.newest.version, this.activeVersion) > 0
	);

	async init() {
		await this.refresh();
		await listen<CoreProgress>('core-progress', (e) => (this.progress = e.payload));
	}

	async refresh() {
		this.status = await coreApi.status();
	}

	private async run<T>(action: () => Promise<T>): Promise<T | undefined> {
		try {
			return await action();
		} catch (e) {
			app.notify(String(e));
		}
	}

	async check() {
		this.checking = true;
		const releases = await this.run(coreApi.check);
		this.checking = false;
		if (releases) {
			this.releases = releases;
			this.checked = true;
			await this.refresh();
		}
	}

	async install(version: string, allowUnverified = false) {
		this.installing = version;
		this.progress = null;
		const ok = await this.run(async () => {
			await coreApi.install(version, allowUnverified);
			return true;
		});
		this.installing = null;
		this.progress = null;
		await this.refresh();
		if (ok) app.notify(`sing-box ${version} installed and selected. It applies on the next connect.`, 'info');
	}

	async use(version: string | null) {
		await this.run(() => coreApi.use(version));
		await this.refresh();
		if (app.connected) app.notify('Reconnect to use the selected core.', 'info');
	}

	async remove(version: string) {
		await this.run(() => coreApi.remove(version));
		await this.refresh();
	}

	async setChannel(channel: Channel) {
		if (!this.status) return;
		await this.run(() => coreApi.updateSettings(channel, this.status!.settings.autoCheck));
		this.releases = [];
		this.checked = false;
		await this.refresh();
	}

	async setAutoCheck(autoCheck: boolean) {
		if (!this.status) return;
		await this.run(() => coreApi.updateSettings(this.status!.settings.channel, autoCheck));
		await this.refresh();
	}

	async brewUpgrade() {
		this.installing = 'brew';
		const output = await this.run(coreApi.brewUpgrade);
		this.installing = null;
		await this.refresh();
		if (output !== undefined) app.notify(output || 'Homebrew finished.', 'info');
	}
}

export const core = new CoreStore();
