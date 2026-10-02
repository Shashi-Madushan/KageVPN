<script lang="ts">
	import Select from './Select.svelte';
	import { onMount } from 'svelte';
	import type { Bucket, Point, TopRow } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { bucketLabel, PRESETS, rangeDays, type Preset } from '$lib/dates';
	import { formatBytes } from '$lib/format';
	import { usage } from '$lib/usage.svelte';
	import BarChart from './BarChart.svelte';
	import HeaderActions from './HeaderActions.svelte';
	import Segmented from './Segmented.svelte';

	const GIB = 1024 ** 3;
	const buckets: { value: Bucket; label: string }[] = [
		{ value: 'hour', label: 'Hour' },
		{ value: 'day', label: 'Day' },
		{ value: 'week', label: 'Week' },
		{ value: 'month', label: 'Month' }
	];

	let confirmReset = $state(false);
	let tab = $state<'app' | 'domain' | 'profile'>('app');

	onMount(() => {
		usage.loadMeta();
		const timer = setInterval(() => app.connected && usage.refresh(), 10000);
		return () => clearInterval(timer);
	});

	$effect(() => {
		usage.range;
		usage.bucket;
		usage.filters;
		usage.compare;
		const timer = setTimeout(() => usage.refresh(), 120);
		return () => clearTimeout(timer);
	});

	const report = $derived(usage.report);
	const total = $derived(report ? report.total.up + report.total.down : 0);
	const days = $derived(rangeDays(usage.range[0], usage.range[1]));
	const change = $derived.by(() => {
		if (!report?.previous) return null;
		const before = report.previous.up + report.previous.down;
		if (before === 0) return total === 0 ? 0 : null;
		return ((total - before) / before) * 100;
	});
	const quota = $derived(usage.quota);

	function duration(seconds: number): string {
		const h = Math.floor(seconds / 3600);
		const m = Math.floor((seconds % 3600) / 60);
		return h ? `${h}h ${String(m).padStart(2, '0')}m` : `${m}m`;
	}

	function profileName(id: string): string {
		if (id === 'auto') return 'Auto (fastest)';
		return app.profiles.find((p) => p.id === id)?.name ?? 'Removed profile';
	}

	function drill(point: Point) {
		const next: Record<Bucket, Bucket | null> = { month: 'week', week: 'day', day: 'hour', hour: null };
		const target = next[usage.bucket];
		if (!target) return;
		const start = new Date(point.start * 1000);
		const from = `${start.getFullYear()}-${String(start.getMonth() + 1).padStart(2, '0')}-${String(start.getDate()).padStart(2, '0')}`;
		const spanDays = usage.bucket === 'month' ? 31 : usage.bucket === 'week' ? 6 : 0;
		const end = new Date(start.getFullYear(), start.getMonth(), start.getDate() + spanDays);
		usage.customFrom = from;
		usage.customTo = `${end.getFullYear()}-${String(end.getMonth() + 1).padStart(2, '0')}-${String(end.getDate()).padStart(2, '0')}`;
		usage.preset = 'custom';
		usage.bucketOverride = target;
	}

	function pick(kind: 'app' | 'domain' | 'profile', row: TopRow) {
		if (kind === 'app') {
			usage.domain = null;
			usage.app = row.key;
		} else if (kind === 'domain') {
			usage.app = null;
			usage.domain = row.key;
		} else {
			usage.profileId = row.key;
		}
	}

	function setPreset(value: Preset) {
		usage.preset = value;
		usage.bucketOverride = null;
	}

	const quotaGb = $derived(usage.settings?.quotaBytes ? +(usage.settings.quotaBytes / GIB).toFixed(2) : '');
	const share = (row: TopRow, rows: TopRow[]) => {
		const max = Math.max(1, ...rows.map((r) => r.up + r.down));
		return ((row.up + row.down) / max) * 100;
	};

	const tabs = $derived([
		{ value: 'app' as const, label: 'Apps', rows: usage.topApps, empty: usage.settings?.recordApps ? 'No app data yet.' : 'App recording is off. Turn it on under Limits and privacy.' },
		{ value: 'domain' as const, label: 'Sites', rows: usage.topDomains, empty: usage.settings?.recordDomains ? 'No site data yet.' : 'Site recording is off. Turn it on under Limits and privacy.' },
		{ value: 'profile' as const, label: 'Servers', rows: usage.topProfiles, empty: 'No data yet.' }
	]);
	const current = $derived(tabs.find((t) => t.value === tab)!);
	const downShare = $derived(total ? ((report?.total.down ?? 0) / total) * 100 : 50);
	const filtered = $derived(!!(usage.profileId || usage.mode || usage.app || usage.domain));
</script>

<HeaderActions>
	<button class="btn" onclick={() => usage.exportCsv()}>Export CSV</button>
</HeaderActions>

<div class="page">
	<div class="range">
		<Select
			value={usage.preset}
			label="Date range"
			options={PRESETS.map((p) => ({ value: p.value, label: p.label }))}
			onchange={(v) => setPreset(v as Preset)}
		/>
		{#if usage.preset === 'custom'}
			<input class="input" type="date" aria-label="From" bind:value={usage.customFrom} max={usage.customTo} />
			<span class="to">to</span>
			<input class="input" type="date" aria-label="To" bind:value={usage.customTo} min={usage.customFrom} />
		{/if}
		<details class="filter-menu" class:active={filtered}>
			<summary class="btn">Filters{filtered ? ' •' : ''}</summary>
			<div class="filter-pop card">
				<label>Server
					<Select
						value={usage.profileId ?? ''}
						label="Server"
						options={[
							{ value: '', label: 'All servers' },
							...app.profiles.map((p) => ({ value: p.id, label: p.name })),
							{ value: 'auto', label: 'Auto (fastest)' }
						]}
						onchange={(v) => (usage.profileId = v || null)}
					/>
				</label>
				<label>Mode
					<Select
						value={usage.mode ?? ''}
						label="Mode"
						options={[
							{ value: '', label: 'All modes' },
							{ value: 'tun', label: 'Full system (TUN)' },
							{ value: 'proxy', label: 'Proxy only' }
						]}
						onchange={(v) => (usage.mode = (v || null) as typeof usage.mode)}
					/>
				</label>
			</div>
		</details>
		{#if usage.app || usage.domain}
			<button class="chip" onclick={() => usage.clearDimension()}>{usage.app ?? usage.domain} ✕</button>
		{/if}
		{#if usage.profileId}
			<button class="chip" onclick={() => (usage.profileId = null)}>{profileName(usage.profileId)} ✕</button>
		{/if}
	</div>

	<section class="card summary">
		<div class="total">
			<span class="caption">Total used</span>
			<strong class="big tabular">{formatBytes(total)}</strong>
			{#if change !== null}
				<small class="delta" class:more={change > 0}>{change >= 0 ? '▲' : '▼'} {Math.abs(change).toFixed(0)}% vs previous period</small>
			{/if}
		</div>
		<div class="split" role="img" aria-label="Download {formatBytes(report?.total.down ?? 0)}, upload {formatBytes(report?.total.up ?? 0)}">
			<div class="bar"><i class="d" style:width="{downShare}%"></i><i class="u" style:width="{100 - downShare}%"></i></div>
			<div class="legend">
				<span><i class="dot down-dot"></i>Download <strong class="tabular">{formatBytes(report?.total.down ?? 0)}</strong></span>
				<span><i class="dot up-dot"></i>Upload <strong class="tabular">{formatBytes(report?.total.up ?? 0)}</strong></span>
			</div>
		</div>
		<p class="facts tabular">
			<span>Daily avg <strong>{formatBytes(total / days)}</strong></span>
			<span>Peak <strong>{report?.peak ? formatBytes(report.peak.up + report.peak.down) : '—'}</strong>{#if report?.peak}{' '}<small>({bucketLabel(report.peak.start, usage.bucket)})</small>{/if}</span>
			{#if !usage.hasDimensionFilter}<span>Connected <strong>{duration(report?.connectedSecs ?? 0)}</strong></span>{/if}
		</p>
		{#if quota?.limit}
			<div class="quota" class:warn={quota.percent >= 80} class:over={quota.percent >= 100}>
				<div class="quota-head">
					<span>Data cap · resets {new Date(quota.cycleEnd * 1000).toLocaleDateString()}</span>
					<span class="tabular">{formatBytes(quota.used)} of {formatBytes(quota.limit)} · {quota.percent.toFixed(0)}%</span>
				</div>
				<div class="meter"><div style:width="{Math.min(100, quota.percent)}%"></div></div>
			</div>
		{/if}
	</section>

	<section class="card chart-card">
		<div class="chart-head">
			<span class="caption">Over time</span>
			<div class="chart-tools">
				<label class="compare"><input type="checkbox" bind:checked={usage.compare} /> Compare</label>
				<Segmented options={buckets} value={usage.bucket} label="Group by" onchange={(b) => (usage.bucketOverride = b)} />
			</div>
		</div>
		{#if report && report.series.length}
			<BarChart points={report.series} bucket={usage.bucket} onselect={drill} />
		{:else}
			<p class="empty">{usage.loading ? 'Loading…' : 'No data for this range yet. Usage is recorded while you are connected.'}</p>
		{/if}
		{#if usage.hasDimensionFilter}<small>Per-app and per-site figures are estimates and are recorded hourly.</small>{/if}
	</section>

	<section class="card breakdown">
		<div class="chart-head">
			<span class="caption">Breakdown</span>
			<Segmented options={tabs.map((t) => ({ value: t.value, label: t.label }))} value={tab} label="Breakdown by" onchange={(v) => (tab = v)} />
		</div>
		{#each current.rows as row (row.key)}
			<button class="trow" onclick={() => pick(tab, row)} title="Filter by this">
				<span class="name">{tab === 'profile' ? profileName(row.key) : row.key}</span>
				<span class="val tabular">{formatBytes(row.up + row.down)}</span>
				<span class="share"><i style:width="{share(row, current.rows)}%"></i></span>
			</button>
		{:else}
			<p class="empty small">{current.empty}</p>
		{/each}
	</section>

	{#if usage.settings}
		{@const s = usage.settings}
		<details class="card settings">
			<summary>Limits and privacy</summary>
			<div class="form">
				<label>Monthly cap (GB)
					<input class="input" type="number" min="0" step="1" value={quotaGb} placeholder="No cap" onchange={(e) => usage.saveSettings({ quotaBytes: e.currentTarget.value ? Math.round(Number(e.currentTarget.value) * GIB) : null })} />
				</label>
				<label>Cycle starts on day
					<input class="input" type="number" min="1" max="31" value={s.cycleDay} onchange={(e) => usage.saveSettings({ cycleDay: Math.min(31, Math.max(1, Number(e.currentTarget.value) || 1)) })} />
				</label>
				<label>At 100%
					<Select
						value={s.quotaAction}
						label="At 100%"
						options={[{ value: 'notify', label: 'Notify me' }, { value: 'disconnect', label: 'Disconnect' }]}
						onchange={(v) => usage.saveSettings({ quotaAction: v })}
					/>
				</label>
				<label>Keep app and site detail
					<Select
						value={String(s.detailRetentionDays)}
						label="Keep app and site detail"
						options={[
							{ value: '30', label: '30 days' },
							{ value: '90', label: '90 days' },
							{ value: '180', label: '180 days' },
							{ value: '365', label: '1 year' }
						]}
						onchange={(v) => usage.saveSettings({ detailRetentionDays: Number(v) })}
					/>
				</label>
			</div>
			<div class="toggles">
				<label class="toggle"><span><strong>Record apps</strong><small>Which apps used the tunnel (estimated).</small></span>
					<input type="checkbox" checked={s.recordApps} onchange={(e) => usage.saveSettings({ recordApps: e.currentTarget.checked })} /></label>
				<label class="toggle"><span><strong>Record sites</strong><small>Domains you visit. Stored only on this Mac. Off by default.</small></span>
					<input type="checkbox" checked={s.recordDomains} onchange={(e) => usage.saveSettings({ recordDomains: e.currentTarget.checked })} /></label>
				<label class="toggle"><span><strong>Pause recording</strong><small>Stop counting new usage.</small></span>
					<input type="checkbox" checked={s.paused} onchange={(e) => usage.saveSettings({ paused: e.currentTarget.checked })} /></label>
			</div>
			<div class="danger-zone">
				<button class="btn" onclick={() => usage.clearDetails()}>Clear app and site history</button>
				{#if confirmReset}
					<button class="btn danger" onclick={() => (usage.resetAll(), (confirmReset = false))}>Yes, delete all usage</button>
					<button class="btn" onclick={() => (confirmReset = false)}>Cancel</button>
				{:else}
					<button class="btn danger" onclick={() => (confirmReset = true)}>Reset all usage…</button>
				{/if}
			</div>
		</details>
	{/if}
</div>

<style>
	.page {
		display: grid;
		gap: 12px;
		align-content: start;
	}
	.range {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 8px;
	}
	.to {
		color: var(--fg-muted);
	}
	.chip {
		border: 1px solid var(--accent);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
		color: var(--accent);
		border-radius: 999px;
		padding: 3px 11px;
		font-size: 12px;
	}
	.filter-menu {
		position: relative;
	}
	.filter-menu summary {
		list-style: none;
	}
	.filter-menu summary::-webkit-details-marker {
		display: none;
	}
	.filter-menu.active summary {
		border-color: var(--accent);
		color: var(--accent);
	}
	.filter-pop {
		position: absolute;
		z-index: 30;
		top: calc(100% + 6px);
		left: 0;
		min-width: 220px;
		padding: 12px;
		display: grid;
		gap: 10px;
		background: var(--surface-strong);
	}
	label {
		display: grid;
		gap: 4px;
		font-size: 11px;
		color: var(--fg-muted);
	}
	small {
		color: var(--fg-muted);
	}
	.summary {
		padding: 18px 20px;
		display: grid;
		gap: 16px;
	}
	.total {
		display: grid;
		gap: 2px;
	}
	.big {
		font-size: 34px;
		font-weight: 650;
		letter-spacing: -0.02em;
		line-height: 1.1;
	}
	.delta {
		font-size: 12px;
	}
	.delta.more {
		color: var(--warn);
	}
	.split {
		display: grid;
		gap: 8px;
	}
	.bar {
		display: flex;
		height: 8px;
		border-radius: 4px;
		overflow: hidden;
		background: var(--hairline);
		gap: 2px;
	}
	.bar i.d {
		background: var(--down);
	}
	.bar i.u {
		background: var(--up);
	}
	.legend {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 20px;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.legend strong {
		color: var(--fg);
		margin-left: 4px;
	}
	.dot {
		display: inline-block;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		margin-right: 6px;
	}
	.down-dot {
		background: var(--down);
	}
	.up-dot {
		background: var(--up);
	}
	.facts {
		margin: 0;
		display: flex;
		flex-wrap: wrap;
		gap: 4px 18px;
		padding-top: 12px;
		border-top: 1px solid var(--hairline);
		font-size: 12px;
		color: var(--fg-muted);
	}
	.facts strong {
		color: var(--fg);
		font-weight: 600;
	}
	.quota {
		display: grid;
		gap: 6px;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.quota-head {
		display: flex;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 4px 12px;
	}
	.meter {
		height: 6px;
		border-radius: 3px;
		background: var(--hairline);
		overflow: hidden;
	}
	.meter div {
		height: 100%;
		background: var(--accent);
		transition: width 0.3s;
	}
	.quota.warn .meter div {
		background: var(--warn);
	}
	.quota.over .meter div {
		background: var(--danger);
	}
	.chart-card,
	.breakdown {
		padding: 14px 16px;
		display: grid;
		gap: 10px;
		align-content: start;
	}
	.chart-head {
		display: flex;
		justify-content: space-between;
		align-items: center;
		flex-wrap: wrap;
		gap: 8px;
	}
	.chart-tools {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.compare {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
	}
	.empty {
		margin: 20px 0;
		text-align: center;
		color: var(--fg-muted);
	}
	.empty.small {
		margin: 10px 0;
		font-size: 12px;
	}
	.trow {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		gap: 3px 10px;
		border: 0;
		background: none;
		text-align: left;
		padding: 7px 8px;
		margin: 0 -8px;
		border-radius: 8px;
	}
	.trow:hover {
		background: var(--hairline);
	}
	.name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.val {
		color: var(--fg-muted);
		font-size: 12px;
	}
	.share {
		grid-column: 1 / -1;
		height: 3px;
		border-radius: 2px;
		background: var(--hairline);
		overflow: hidden;
	}
	.share i {
		display: block;
		height: 100%;
		background: var(--accent);
	}
	.settings {
		padding: 0;
	}
	.settings summary {
		padding: 12px 16px;
		font-weight: 600;
		cursor: pointer;
	}
	.settings[open] summary {
		border-bottom: 1px solid var(--hairline);
	}
	.form {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
		gap: 12px;
		padding: 14px 16px;
	}
	.toggles {
		border-top: 1px solid var(--hairline);
	}
	.toggle {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 16px;
		padding: 12px 16px;
		color: var(--fg);
		font-size: 13px;
	}
	.toggle + .toggle {
		border-top: 1px solid var(--hairline);
	}
	.toggle span {
		display: grid;
	}
	.danger-zone {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		padding: 12px 16px 14px;
		border-top: 1px solid var(--hairline);
	}
</style>
