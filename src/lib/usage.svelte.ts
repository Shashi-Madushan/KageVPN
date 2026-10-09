import { app } from './app.svelte';
import {
	usageApi,
	type Bucket,
	type Filters,
	type QuotaStatus,
	type TopRow,
	type UsageReport,
	type UsageSettings
} from './api';
import { autoBucket, rangeFor, toDateInput, type Preset } from './dates';

class UsageStore {
	preset = $state<Preset>('thisMonth');
	customFrom = $state(toDateInput(new Date(Date.now() - 6 * 86400000)));
	customTo = $state(toDateInput(new Date()));
	bucketOverride = $state<Bucket | null>(null);
	compare = $state(true);
	profileId = $state<string | null>(null);
	mode = $state<string | null>(null);
	app = $state<string | null>(null);
	domain = $state<string | null>(null);

	report = $state<UsageReport | null>(null);
	topApps = $state<TopRow[]>([]);
	topDomains = $state<TopRow[]>([]);
	topProfiles = $state<TopRow[]>([]);
	quota = $state<QuotaStatus | null>(null);
	settings = $state<UsageSettings | null>(null);
	today = $state<{ up: number; down: number } | null>(null);
	/** True until the first load of `today` finishes, so the UI can show a skeleton. */
	todayLoading = $state(true);
	loading = $state(false);

	range = $derived(
		rangeFor(this.preset, { from: this.customFrom, to: this.customTo }, this.quota ? { start: this.quota.cycleStart, end: this.quota.cycleEnd } : null)
	);
	bucket = $derived<Bucket>(this.bucketOverride ?? autoBucket(this.range[0], this.range[1]));
	filters = $derived<Filters>({ profileId: this.profileId, mode: this.mode, app: this.app, domain: this.domain });
	hasDimensionFilter = $derived(this.app !== null || this.domain !== null);

	async loadMeta() {
		try {
			[this.settings, this.quota] = await Promise.all([usageApi.settings(), usageApi.quota()]);
		} catch (e) {
			app.notify(String(e));
		}
	}

	async refresh() {
		this.loading = true;
		const [from, to] = this.range;
		const filters = this.filters;
		const dimension = { ...filters, app: null, domain: null };
		try {
			const [report, apps, domains, profiles] = await Promise.all([
				usageApi.report({ from, to, bucket: this.bucket, filters, compare: this.compare }),
				usageApi.top({ kind: 'app', from, to, limit: 8, filters: dimension }),
				usageApi.top({ kind: 'domain', from, to, limit: 8, filters: dimension }),
				usageApi.top({ kind: 'profile', from, to, limit: 8, filters: { mode: this.mode } })
			]);
			this.report = report;
			this.topApps = apps;
			this.topDomains = domains;
			this.topProfiles = profiles;
		} catch (e) {
			app.notify(String(e));
		} finally {
			this.loading = false;
		}
	}

	async loadToday() {
		const start = new Date();
		start.setHours(0, 0, 0, 0);
		const from = Math.floor(start.getTime() / 1000);
		try {
			const report = await usageApi.report({ from, to: from + 86400, bucket: 'day', filters: {}, compare: false });
			this.today = report.total;
			this.quota = await usageApi.quota();
		} catch {
			this.today = null;
		} finally {
			this.todayLoading = false;
		}
	}

	async saveSettings(patch: Partial<UsageSettings>) {
		if (!this.settings) return;
		const next = { ...this.settings, ...patch };
		try {
			await usageApi.setSettings(next);
			this.settings = next;
			this.quota = await usageApi.quota();
		} catch (e) {
			app.notify(String(e));
		}
	}

	async exportCsv() {
		const [from, to] = this.range;
		try {
			const path = await usageApi.exportCsv({ from, to, bucket: this.bucket, filters: this.filters, compare: false });
			app.notify(`Saved to ${path}`, 'info');
		} catch (e) {
			app.notify(String(e));
		}
	}

	async resetAll() {
		await usageApi.reset().catch((e) => app.notify(String(e)));
		await this.refresh();
		await this.loadMeta();
	}

	async clearDetails() {
		await usageApi.clearDetails().catch((e) => app.notify(String(e)));
		await this.refresh();
	}

	clearDimension() {
		this.app = null;
		this.domain = null;
	}
}

export const usage = new UsageStore();
