import { listen } from '@tauri-apps/api/event';
import { app } from './app.svelte';
import {
	appUpdateApi,
	type AppUpdateInfo,
	type DownloadFinished,
	type DownloadProgress,
	type UpdateAvailable,
	type UpdatePolicy
} from './api';
import { downloads } from './downloads.svelte';

/** Updates for the app itself. Everything stays idle unless `app_config.rs` sets a feed and key. */
class AppUpdate {
	policy = $state<UpdatePolicy>('ask');
	available = $state<AppUpdateInfo | null>(null);
	checking = $state(false);
	/** Result of the last manual check, for the Settings row. */
	checkedMessage = $state<string | null>(null);
	installing = $state(false);
	/** Version that is installed and waiting for a restart. */
	restartPending = $state<string | null>(null);
	failure = $state<string | null>(null);
	confirmRestart = $state(false);

	enabled = $derived(app.info.updatesConfigured);

	async init() {
		try {
			this.policy = (await appUpdateApi.settings()).policy;
		} catch {
			// keep the default
		}
		if (!app.info.updatesConfigured) return;
		downloads.onRetry('app', () => this.install());
		await listen<DownloadProgress>('download-progress', (e) => {
			if (e.payload.kind === 'app') this.installing = true;
		});
		await listen<DownloadFinished>('download-finished', (e) => {
			if (e.payload.kind !== 'app') return;
			this.installing = false;
			this.failure = e.payload.error && !/cancel/i.test(e.payload.error) ? e.payload.error : null;
			if (e.payload.version) {
				this.restartPending = e.payload.version;
				this.available = null;
			}
		});
		await listen<UpdateAvailable>('update-available', (e) => {
			if (e.payload.kind === 'app') this.available = { version: e.payload.version, notes: null };
		});
		// The backend checks too, but it may announce before this window is listening.
		// "Automatic" installs from the backend, so only look here when the user decides.
		if (this.policy === 'ask') await this.check(true);
	}

	async check(silent = false) {
		if (this.checking) return;
		this.checking = true;
		if (!silent) this.checkedMessage = null;
		try {
			const found = await appUpdateApi.check();
			this.available = found;
			if (!silent) this.checkedMessage = found ? `Version ${found.version} is available.` : `You have the latest version (${app.info.version}).`;
		} catch (e) {
			if (!silent) this.checkedMessage = String(e);
		} finally {
			this.checking = false;
		}
	}

	async install() {
		this.failure = null;
		this.installing = true;
		downloads.start({ id: 'app', kind: 'app', version: this.available?.version ?? null, title: `${app.info.name} ${this.available?.version ?? ''}`.trim(), detail: 'Starting' });
		try {
			await appUpdateApi.install();
		} catch (e) {
			this.installing = false;
			downloads.finish('app', String(e));
			this.failure = String(e);
		}
	}

	async setPolicy(policy: UpdatePolicy) {
		try {
			await appUpdateApi.setPolicy(policy);
			this.policy = policy;
		} catch (e) {
			app.notify(String(e));
		}
	}

	/** Restarts into the new version; a live connection is only ended after the user agrees. */
	async restart(force = false) {
		if (app.locked && !force) {
			this.confirmRestart = true;
			return;
		}
		this.confirmRestart = false;
		try {
			await appUpdateApi.restart(force || app.locked);
		} catch (e) {
			app.notify(String(e));
		}
	}
}

export const appUpdate = new AppUpdate();
