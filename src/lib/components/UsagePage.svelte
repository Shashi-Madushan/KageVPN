<script lang="ts">
	import { onMount } from 'svelte';
	import type { Bucket, Point, TopRow } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { bucketLabel, PRESETS, rangeDays, type Preset } from '$lib/dates';
	import { formatBytes } from '$lib/format';
	import { usage } from '$lib/usage.svelte';
	import BarChart from './BarChart.svelte';
	import Segmented from './Segmented.svelte';

	const GIB = 1024 ** 3;
	const buckets: { value: Bucket; label: string }[] = [
		{ value: 'hour', label: 'Hour' },
		{ value: 'day', label: 'Day' },
		{ value: 'week', label: 'Week' },
		{ value: 'month', label: 'Month' }
	];

	let confirmReset = $state(false);

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
</script>

<div class="page">
	<section class="card controls">
		<div class="presets" role="tablist" aria-label="Date range">
			{#each PRESETS as preset (preset.value)}
				<button class:active={usage.preset === preset.value} role="tab" aria-selected={usage.preset === preset.value} onclick={() => setPreset(preset.value)}>
					{preset.label}
				</button>
			{/each}
		</div>
		{#if usage.preset === 'custom'}
			<div class="custom">
				<label>From <input type="date" bind:value={usage.customFrom} max={usage.customTo} /></label>
				<label>To <input type="date" bind:value={usage.customTo} min={usage.customFrom} /></label>
			</div>
		{/if}
		<div class="row">
			<Segmented options={buckets} value={usage.bucket} label="Group by" onchange={(b) => (usage.bucketOverride = b)} />
			<label class="check"><input type="checkbox" bind:checked={usage.compare} /> Compare with previous period</label>
		</div>
		<div class="row filters">
			<select bind:value={usage.profileId} aria-label="Profile">
				<option value={null}>All profiles</option>
				{#each app.profiles as profile (profile.id)}<option value={profile.id}>{profile.name}</option>{/each}
				<option value="auto">Auto (fastest)</option>
			</select>
			<select bind:value={usage.mode} aria-label="Mode">
				<option value={null}>All modes</option>
				<option value="tun">TUN</option>
				<option value="proxy">Proxy</option>
			</select>
			{#if usage.app || usage.domain}
				<button class="chip" onclick={() => usage.clearDimension()}>{usage.app ?? usage.domain} ✕</button>
			{/if}
			{#if usage.profileId}
				<button class="chip" onclick={() => (usage.profileId = null)}>{profileName(usage.profileId)} ✕</button>
			{/if}
			<span class="spacer"></span>
			<button onclick={() => usage.exportCsv()}>Export CSV</button>
		</div>
	</section>

	{#if quota?.limit}
		<section class="card quota" class:warn={quota.percent >= 80} class:over={quota.percent >= 100}>
			<div class="quota-head">
				<strong>Data cap</strong>
				<span class="tabular">{formatBytes(quota.used)} of {formatBytes(quota.limit)} · {quota.percent.toFixed(0)}%</span>
			</div>
			<div class="meter"><div style:width="{Math.min(100, quota.percent)}%"></div></div>
			<small>Cycle {new Date(quota.cycleStart * 1000).toLocaleDateString()} to {new Date((quota.cycleEnd - 1) * 1000).toLocaleDateString()}</small>
		</section>
	{/if}

	<section class="tiles">
		<div class="card tile"><span class="caption">Total</span><strong class="tabular">{formatBytes(total)}</strong>
			{#if change !== null}<small class:up-bad={change > 0}>{change >= 0 ? '▲' : '▼'} {Math.abs(change).toFixed(0)}% vs previous</small>{/if}
		</div>
		<div class="card tile"><span class="caption">Download</span><strong class="tabular down">{formatBytes(report?.total.down ?? 0)}</strong></div>
		<div class="card tile"><span class="caption">Upload</span><strong class="tabular up">{formatBytes(report?.total.up ?? 0)}</strong></div>
		<div class="card tile"><span class="caption">Daily average</span><strong class="tabular">{formatBytes(total / days)}</strong></div>
		<div class="card tile"><span class="caption">Peak</span><strong class="tabular">{report?.peak ? formatBytes(report.peak.up + report.peak.down) : '—'}</strong>
			{#if report?.peak}<small>{bucketLabel(report.peak.start, usage.bucket)}</small>{/if}
		</div>
		<div class="card tile"><span class="caption">Connected</span><strong class="tabular">{usage.hasDimensionFilter ? '—' : duration(report?.connectedSecs ?? 0)}</strong></div>
	</section>

	<section class="card chart-card">
		<div class="chart-head">
			<span class="caption">Usage over time</span>
			<span class="legend"><i class="dot down-dot"></i>Download <i class="dot up-dot"></i>Upload</span>
		</div>
		{#if report && report.series.length}
			<BarChart points={report.series} bucket={usage.bucket} onselect={drill} />
		{:else}
			<p class="empty">{usage.loading ? 'Loading…' : 'No data for this range yet. Usage is recorded while you are connected.'}</p>
		{/if}
		{#if usage.hasDimensionFilter}<small>Per-app and per-domain figures are estimates and are recorded hourly.</small>{/if}
	</section>

	<section class="tables">
		{#each [{ title: 'Top apps', kind: 'app' as const, rows: usage.topApps, empty: usage.settings?.recordApps ? 'No app data yet.' : 'App recording is off.' }, { title: 'Top domains', kind: 'domain' as const, rows: usage.topDomains, empty: usage.settings?.recordDomains ? 'No domain data yet.' : 'Domain recording is off.' }, { title: 'By profile', kind: 'profile' as const, rows: usage.topProfiles, empty: 'No data yet.' }] as table (table.kind)}
			<div class="card table">
				<span class="caption">{table.title}</span>
				{#each table.rows as row (row.key)}
					<button class="trow" onclick={() => pick(table.kind, row)} title="Filter by this">
						<span class="name">{table.kind === 'profile' ? profileName(row.key) : row.key}</span>
						<span class="val tabular">{formatBytes(row.up + row.down)}</span>
						<span class="share"><i style:width="{share(row, table.rows)}%"></i></span>
					</button>
				{:else}
					<small>{table.empty}</small>
				{/each}
			</div>
		{/each}
	</section>

	{#if usage.settings}
		{@const s = usage.settings}
		<section class="card settings">
			<span class="caption">Data and limits</span>
			<div class="form">
				<label>Monthly cap (GB)
					<input type="number" min="0" step="1" value={quotaGb} placeholder="No cap" onchange={(e) => usage.saveSettings({ quotaBytes: e.currentTarget.value ? Math.round(Number(e.currentTarget.value) * GIB) : null })} />
				</label>
				<label>Cycle starts on day
					<input type="number" min="1" max="31" value={s.cycleDay} onchange={(e) => usage.saveSettings({ cycleDay: Math.min(31, Math.max(1, Number(e.currentTarget.value) || 1)) })} />
				</label>
				<label>At 100%
					<select value={s.quotaAction} onchange={(e) => usage.saveSettings({ quotaAction: e.currentTarget.value as 'notify' | 'disconnect' })}>
						<option value="notify">Notify me</option>
						<option value="disconnect">Disconnect</option>
					</select>
				</label>
				<label>Keep app/domain detail
					<select value={s.detailRetentionDays} onchange={(e) => usage.saveSettings({ detailRetentionDays: Number(e.currentTarget.value) })}>
						<option value={30}>30 days</option>
						<option value={90}>90 days</option>
						<option value={180}>180 days</option>
						<option value={365}>1 year</option>
					</select>
				</label>
			</div>
			<label class="toggle"><span><strong>Record apps</strong><small>Which apps used the tunnel (estimated).</small></span>
				<input type="checkbox" checked={s.recordApps} onchange={(e) => usage.saveSettings({ recordApps: e.currentTarget.checked })} /></label>
			<label class="toggle"><span><strong>Record domains</strong><small>Sites you visit. Stored only on this Mac. Off by default.</small></span>
				<input type="checkbox" checked={s.recordDomains} onchange={(e) => usage.saveSettings({ recordDomains: e.currentTarget.checked })} /></label>
			<label class="toggle"><span><strong>Pause recording</strong><small>Stop counting new usage.</small></span>
				<input type="checkbox" checked={s.paused} onchange={(e) => usage.saveSettings({ paused: e.currentTarget.checked })} /></label>
			<div class="row">
				<button onclick={() => usage.clearDetails()}>Clear app and domain history</button>
				{#if confirmReset}
					<button class="danger" onclick={() => (usage.resetAll(), (confirmReset = false))}>Yes, delete all usage</button>
					<button onclick={() => (confirmReset = false)}>Cancel</button>
				{:else}
					<button class="danger" onclick={() => (confirmReset = true)}>Reset all usage…</button>
				{/if}
			</div>
		</section>
	{/if}
</div>

<style>
	.page {
		display: grid;
		gap: 14px;
		align-content: start;
	}
	.controls {
		padding: 12px 14px;
		display: grid;
		gap: 10px;
	}
	.presets {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.presets button,
	.chip {
		border: 1px solid var(--hairline);
		background: var(--surface);
		border-radius: 999px;
		padding: 3px 11px;
		font-size: 12px;
	}
	.presets button.active {
		background: var(--accent);
		border-color: var(--accent);
		color: white;
		font-weight: 600;
	}
	.custom,
	.row {
		display: flex;
		gap: 10px;
		align-items: center;
		flex-wrap: wrap;
	}
	.filters .spacer {
		flex: 1;
	}
	.check {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		color: var(--fg-muted);
	}
	input,
	select {
		padding: 5px 8px;
		border-radius: 8px;
		border: 1px solid var(--hairline);
		background: var(--surface);
		user-select: text;
	}
	label {
		display: grid;
		gap: 3px;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.check input,
	.toggle input {
		padding: 0;
	}
	button {
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		border-radius: 8px;
		padding: 5px 12px;
	}
	button.danger {
		color: var(--danger);
	}
	.chip {
		background: color-mix(in srgb, var(--accent) 18%, transparent);
		border-color: var(--accent);
		color: var(--accent);
	}
	.quota {
		padding: 12px 14px;
		display: grid;
		gap: 6px;
	}
	.quota.warn {
		border-color: var(--warn);
	}
	.quota.over {
		border-color: var(--danger);
	}
	.quota-head {
		display: flex;
		justify-content: space-between;
	}
	.meter {
		height: 7px;
		border-radius: 4px;
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
	small {
		color: var(--fg-muted);
	}
	.tiles {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 10px;
	}
	.tile {
		padding: 12px 14px;
		display: grid;
		gap: 2px;
		align-content: start;
	}
	.tile strong {
		font-size: 18px;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	.up-bad {
		color: var(--warn);
	}
	.chart-card {
		padding: 14px 16px;
		display: grid;
		gap: 8px;
	}
	.chart-head {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.legend {
		font-size: 11px;
		color: var(--fg-muted);
	}
	.dot {
		display: inline-block;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		margin: 0 4px 0 10px;
	}
	.down-dot {
		background: var(--down);
	}
	.up-dot {
		background: var(--up);
	}
	.empty {
		margin: 24px 0;
		text-align: center;
		color: var(--fg-muted);
	}
	.tables {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 10px;
	}
	.table {
		padding: 12px 14px;
		display: grid;
		gap: 4px;
		align-content: start;
	}
	.trow {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 2px 8px;
		border: 0;
		background: none;
		text-align: left;
		padding: 5px 0;
		border-radius: 6px;
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
		padding: 14px 16px;
		display: grid;
		gap: 12px;
	}
	.form {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 10px;
	}
	.toggle {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 16px;
		color: var(--fg);
		font-size: 13px;
	}
	.toggle span {
		display: grid;
	}
	.toggle input {
		width: 34px;
		height: 20px;
		appearance: none;
		border-radius: 10px;
		background: var(--hairline);
		position: relative;
		flex: none;
		transition: background 0.2s;
	}
	.toggle input::after {
		content: '';
		position: absolute;
		top: 2px;
		left: 2px;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: white;
		box-shadow: 0 1px 2px rgb(0 0 0 / 0.3);
		transition: transform 0.2s;
	}
	.toggle input:checked {
		background: var(--ok);
	}
	.toggle input:checked::after {
		transform: translateX(14px);
	}
</style>
