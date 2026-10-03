import { listen } from '@tauri-apps/api/event';
import { app } from './app.svelte';
import { coreApi, downloadsApi, type Channel, type CoreStatus, type DownloadFinished, type DownloadProgress, type Release, type UpdatePolicy, type UpdateAvailable } from './api';
import { downloads } from './downloads.svelte';
import { compareVersions } from './version';


class CoreStore {
	status = $state<CoreStatus | null>(null);
	releases = $state<Release[]>([]);
	progress = $state<DownloadProgress | null>(null);
	checking = $state(false);
	installing = $state<string | null>(null);
	checked = $state(false);
	/** Newer version reported by the daily background check. */
	announced = $state<string | null>(null);
	failure = $state<string | null>(null);
	promptInstall = $state(false);

	coreName = $derived(this.status?.core.name ?? 'Engine');
	activeVersion = $derived(this.status?.active?.version ?? null);
	missing = $derived(this.status !== null && this.status.active === null);
	newest = $derived(
		this.releases.reduce<Release | null>(
			(best, r) => (best === null || compareVersions(r.version, best.version) > 0 ? r : best),
			null
		)
	);
	latestVersion = $derived(this.newest?.version ?? this.announced);
	updateAvailable = $derived(
		this.activeVersion !== null && this.latestVersion !== null && compareVersions(this.latestVersion, this.activeVersion) > 0
	);
	downloading = $derived(this.installing !== null && this.installing !== 'brew');
	percent = $derived(
		this.progress && this.progress.total > 0 ? Math.min(100, (this.progress.received / this.progress.total) * 100) : null
	);

	async init() {
		await this.refresh();
		downloads.onRetry('core', (version) => this.installBackground(version));
		await listen<DownloadProgress>('download-progress', (e) => {
			if (e.payload.kind !== 'core') return;
			this.progress = e.payload;
			if (this.installing === null && e.payload.stage !== 'done') this.installing = e.payload.version;
		});
		await listen<DownloadFinished>('download-finished', async (e) => {
			if (e.payload.kind !== 'core') return;
			this.installing = null;
			this.progress = null;
			this.failure = e.payload.error;
			if (e.payload.version) this.announced = null;
			await this.refresh();
		});
		await listen<UpdateAvailable>('update-available', (e) => {
			if (e.payload.kind === 'core') this.announced = e.payload.version;
		});
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

	/** Downloads in the backend, so it keeps going if the user leaves this page. */
	async installBackground(version: string | null = null, allowUnverified = false) {
		this.failure = null;
		const id = `core:${this.status?.core.id ?? 'engine'}`;
		downloads.start({
			id,
			kind: 'core',
			version,
			title: version ? `${this.coreName} ${version}` : `${this.coreName} engine`,
			detail: 'Starting'
		});
		this.installing = version ?? 'latest';
		this.progress = null;
		this.promptInstall = false;
		try {
			await coreApi.installBackground(version, allowUnverified);
		} catch (e) {
			this.installing = null;
			downloads.finish(`core:${this.status?.core.id ?? 'engine'}`, String(e));
			app.notify(String(e));
		}
	}

	cancel() {
		downloadsApi.cancel().catch(() => null);
	}

	async setActive(id: string) {
		await this.run(() => coreApi.setActive(id));
		this.releases = [];
		this.checked = false;
		this.announced = null;
		await this.refresh();
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
		await this.run(() => coreApi.setChannel(channel));
		this.releases = [];
		this.checked = false;
		await this.refresh();
	}

	async setPolicy(policy: UpdatePolicy) {
		await this.run(() => coreApi.setPolicy(policy));
		await this.refresh();
	}

	async brewUpgrade() {
		this.installing = 'brew';
		downloads.start({ id: 'brew', title: 'sing-box via Homebrew', detail: 'Upgrading' });
		const output = await this.run(coreApi.brewUpgrade);
		downloads.finish('brew', output === undefined ? 'Upgrade failed' : null);
		this.installing = null;
		await this.refresh();
		if (output !== undefined) app.notify(output || 'Homebrew finished.', 'info');
	}
}

export const core = new CoreStore();
