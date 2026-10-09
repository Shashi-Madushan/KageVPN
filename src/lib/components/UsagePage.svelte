<script lang="ts">
	import Select from './Select.svelte';
	import { onMount } from 'svelte';
	import type { Bucket, Point, TopRow } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { bucketLabel, rangeDays, type Preset } from '$lib/dates';
	import { formatBytes, profileFlag } from '$lib/format';
	import { usage } from '$lib/usage.svelte';
	import BarChart from './BarChart.svelte';
	import HeaderActions from './HeaderActions.svelte';
	import Segmented from './Segmented.svelte';
	import Skeleton from './Skeleton.svelte';

	const GIB = 1024 ** 3;
	const quick: { value: Preset; label: string }[] = [
		{ value: 'today', label: 'Today' },
		{ value: 'week', label: '7 days' },
		{ value: 'thisMonth', label: 'This month' },
		{ value: 'lastMonth', label: 'Last month' }
	];
	const more: { value: string; label: string }[] = [
		{ value: '', label: 'More ranges…' },
		{ value: 'yesterday', label: 'Yesterday' },
		{ value: 'month30', label: 'Last 30 days' },
		{ value: 'cycle', label: 'Billing cycle' },
		{ value: 'custom', label: 'Pick dates' }
	];
	const buckets: { value: Bucket; label: string }[] = [
		{ value: 'hour', label: 'Hour' },
		{ value: 'day', label: 'Day' },
		{ value: 'week', label: 'Week' },
		{ value: 'month', label: 'Month' }
	];
	const periodName: Partial<Record<Preset, string>> = {
		today: 'today',
		yesterday: 'yesterday',
		week: 'in the last 7 days',
		month30: 'in the last 30 days',
		thisMonth: 'this month',
		lastMonth: 'last month',
		cycle: 'this billing cycle',
		custom: 'in this period'
	};

	let confirmReset = $state(false);
	let tab = $state<'app' | 'domain' | 'profile'>('app');
	let settingsBox: HTMLDetailsElement | undefined = $state();

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
	const quota = $derived(usage.quota);
	const when = $derived(periodName[usage.preset] ?? 'in this period');
	const isMoreRange = $derived(['yesterday', 'month30', 'cycle', 'custom'].includes(usage.preset));
	const change = $derived.by(() => {
		if (!report?.previous) return null;
		const before = report.previous.up + report.previous.down;
		if (before === 0) return null;
		return ((total - before) / before) * 100;
	});
	const changeText = $derived.by(() => {
		if (change === null) return null;
		const against = usage.preset === 'thisMonth' ? 'last month' : usage.preset === 'today' ? 'yesterday' : 'the period before';
		if (Math.abs(change) < 1) return `About the same as ${against}`;
		return `${Math.abs(change).toFixed(0)}% ${change > 0 ? 'more' : 'less'} than ${against}`;
	});
	const filtered = $derived(!!(usage.profileId || usage.mode || usage.app || usage.domain));
	const downShare = $derived(total ? ((report?.total.down ?? 0) / total) * 100 : 50);
	const perDay = $derived(formatBytes(total / days));
	const busiest = $derived(report?.peak ? `${bucketLabel(report.peak.start, usage.bucket)} · ${formatBytes(report.peak.up + report.peak.down)}` : null);

	function duration(seconds: number): string {
		const h = Math.floor(seconds / 3600);
		const m = Math.floor((seconds % 3600) / 60);
		return h ? `${h}h ${String(m).padStart(2, '0')}m` : `${m}m`;
	}

	function profileName(id: string): string {
		if (id === 'auto') return 'Auto (fastest)';
		return app.profiles.find((p) => p.id === id)?.name ?? 'Removed server';
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

	function clearFilters() {
		usage.clearDimension();
		usage.profileId = null;
		usage.mode = null;
	}

	function openSettings() {
		if (settingsBox) {
			settingsBox.open = true;
			settingsBox.scrollIntoView({ behavior: 'smooth', block: 'center' });
		}
	}

	const quotaGb = $derived(usage.settings?.quotaBytes ? +(usage.settings.quotaBytes / GIB).toFixed(2) : '');
	const rowTotal = (r: TopRow) => r.up + r.down;
	const maxOf = (rows: TopRow[]) => Math.max(1, ...rows.map(rowTotal));
	const sumOf = (rows: TopRow[]) => rows.reduce((sum, r) => sum + rowTotal(r), 0);
	const percentOf = (row: TopRow, rows: TopRow[]) => {
		const sum = sumOf(rows);
		if (!sum) return '0%';
		const value = (rowTotal(row) / sum) * 100;
		return value > 0 && value < 1 ? '<1%' : `${Math.round(value)}%`;
	};

	/** Plain display name for one breakdown row. */
	function describe(kind: 'app' | 'domain' | 'profile', row: TopRow) {
		if (kind === 'profile') {
			const name = profileName(row.key);
			const server = app.profiles.find((p) => p.id === row.key)?.server ?? '';
			return { title: name, note: null as string | null, flag: profileFlag(name, server) };
		}
		if (kind === 'app') {
			// macOS reports apps as "Name (owner)"; the owner only matters for system processes.
			const match = row.key.match(/^(.*?)\s*\(([^)]+)\)$/);
			return { title: match ? match[1] : row.key, note: match?.[2] === 'root' ? 'system' : null, flag: null };
		}
		return { title: row.key, note: null, flag: null };
	}

	const tabs = $derived([
		{ value: 'app' as const, label: 'Apps', rows: usage.topApps, empty: usage.settings?.recordApps ? 'No app data yet.' : 'App tracking is off. Turn it on in the settings below to see which apps use your data.' },
		{ value: 'domain' as const, label: 'Websites', rows: usage.topDomains, empty: usage.settings?.recordDomains ? 'No website data yet.' : 'Website tracking is off to protect your privacy. You can turn it on in the settings below.' },
		{ value: 'profile' as const, label: 'Servers', rows: usage.topProfiles, empty: 'No data yet.' }
	]);
	const current = $derived(tabs.find((t) => t.value === tab)!);
</script>

<HeaderActions>
	<button class="btn" onclick={() => usage.exportCsv()}>Export CSV</button>
</HeaderActions>

<div class="page">
	<div class="period">
		<Segmented options={quick} value={isMoreRange ? ('' as Preset) : usage.preset} label="Time period" onchange={setPreset} />
		<Select value={isMoreRange ? usage.preset : ''} label="More ranges" options={more} onchange={(v) => v && setPreset(v as Preset)} />
		{#if usage.preset === 'custom'}
			<span class="dates">
				<input class="input" type="date" aria-label="From" bind:value={usage.customFrom} max={usage.customTo} />
				<span class="to">to</span>
				<input class="input" type="date" aria-label="To" bind:value={usage.customTo} min={usage.customFrom} />
			</span>
		{/if}
	</div>

	{#if filtered}
		<div class="showing" role="status">
			<span>Showing only:</span>
			{#if usage.app || usage.domain}<span class="chip">{usage.app ?? usage.domain}</span>{/if}
			{#if usage.profileId}<span class="chip">{profileName(usage.profileId)}</span>{/if}
			{#if usage.mode}<span class="chip">{usage.mode === 'tun' ? 'Full system' : 'Proxy only'}</span>{/if}
			<button class="link" onclick={clearFilters}>Show everything</button>
		</div>
	{/if}

	<section class="card hero" aria-live="polite">
		{#if !report}
			<Skeleton width="60%" height="22px" />
			<Skeleton width="40%" height="44px" radius="10px" />
			<Skeleton height="10px" radius="5px" />
		{:else if total === 0 && !usage.loading}
			<div class="none">
				<strong>No data {when} yet</strong>
				<span>Usage is counted while you are connected. Connect and browse a little, then come back.</span>
			</div>
		{:else}
			<span class="lead">You used</span>
			<strong class="big tabular">{formatBytes(total)}</strong>
			<span class="lead">{when}{#if changeText} · <em class:more={change !== null && change > 0}>{changeText}</em>{/if}</span>

			<div class="split" role="img" aria-label="Downloaded {formatBytes(report.total.down)}, uploaded {formatBytes(report.total.up)}">
				<div class="bar"><i class="d" style:width="{downShare}%"></i><i class="u" style:width="{100 - downShare}%"></i></div>
				<div class="legend">
					<span><i class="dot down-dot"></i>Downloaded <strong class="tabular">{formatBytes(report.total.down)}</strong></span>
					<span><i class="dot up-dot"></i>Uploaded <strong class="tabular">{formatBytes(report.total.up)}</strong></span>
				</div>
			</div>

			<div class="facts">
				<div><small>Per day</small><strong class="tabular">{perDay}</strong></div>
				{#if busiest}<div><small>Busiest {usage.bucket === 'hour' ? 'hour' : usage.bucket}</small><strong class="tabular">{busiest}</strong></div>{/if}
				{#if !usage.hasDimensionFilter}<div><small>Time connected</small><strong class="tabular">{duration(report.connectedSecs)}</strong></div>{/if}
			</div>
		{/if}

		{#if quota?.limit}
			<div class="quota" class:warn={quota.percent >= 80} class:over={quota.percent >= 100}>
				<div class="quota-head">
					<span>Monthly limit · resets {new Date(quota.cycleEnd * 1000).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}</span>
					<strong class="tabular">{formatBytes(quota.used)} of {formatBytes(quota.limit)}</strong>
				</div>
				<div class="meter"><div style:width="{Math.min(100, quota.percent)}%"></div></div>
				<small>{quota.percent >= 100 ? 'You have reached your limit.' : `${formatBytes(Math.max(0, quota.limit - quota.used))} left (${(100 - quota.percent).toFixed(0)}%)`}</small>
			</div>
		{:else if usage.settings}
			<button class="link cap-hint" onclick={openSettings}>Set a monthly data limit</button>
		{/if}
	</section>

	<section class="card chart-card">
		<div class="section-head">
			<h2>{usage.bucket === 'hour' ? 'Hour by hour' : usage.bucket === 'day' ? 'Day by day' : usage.bucket === 'week' ? 'Week by week' : 'Month by month'}</h2>
			{#if usage.preset === 'custom' && usage.bucketOverride}<button class="link" onclick={() => setPreset('thisMonth')}>Back to this month</button>{/if}
		</div>
		{#if report && report.series.length}
			<BarChart points={report.series} bucket={usage.bucket} onselect={drill} />
			<small>Hover a bar to see its total. Click a bar to zoom in.</small>
		{:else}
			<p class="empty">{usage.loading ? 'Loading…' : 'Nothing to show for this period yet.'}</p>
		{/if}
		{#if usage.hasDimensionFilter}<small>Per-app and per-website figures are estimates.</small>{/if}
		<details class="options">
			<summary>Chart options</summary>
			<div class="options-body">
				<label>Group bars by
					<Segmented options={buckets} value={usage.bucket} label="Group by" onchange={(b) => (usage.bucketOverride = b)} />
				</label>
				<label class="check"><input type="checkbox" bind:checked={usage.compare} /> Compare with the previous period</label>
			</div>
		</details>
	</section>

	<section class="card breakdown">
		<div class="section-head">
			<h2>Where did it go?</h2>
			<Segmented options={tabs.map((t) => ({ value: t.value, label: t.label }))} value={tab} label="Break down by" onchange={(v) => (tab = v)} />
		</div>
		{#if current.rows.length}
			{@const max = maxOf(current.rows)}
			<ul class="list">
				{#each current.rows as row (row.key)}
					{@const info = describe(tab, row)}
					<li>
						<button class="item" onclick={() => pick(tab, row)} title="Show only {info.title}">
							<span class="name">
								{#if info.flag}<span class="flag" aria-hidden="true">{info.flag}</span>{/if}
								<span class="text">{info.title}</span>
								{#if info.note}<span class="tag">{info.note}</span>{/if}
							</span>
							<span class="amount tabular"><strong>{formatBytes(rowTotal(row))}</strong><small>{percentOf(row, current.rows)}</small></span>
							<span class="track" aria-hidden="true"><i style:width="{(rowTotal(row) / max) * 100}%"></i></span>
							<span class="detail tabular">↓ {formatBytes(row.down)} · ↑ {formatBytes(row.up)}</span>
						</button>
					</li>
				{/each}
			</ul>
			<small>Click one to see only its usage.</small>
		{:else}
			<p class="empty small">{current.empty}</p>
		{/if}
	</section>

	<details class="card options narrow">
		<summary>Filter by server or connection type</summary>
		<div class="options-body grid2">
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
			<label>Connection type
				<Select
					value={usage.mode ?? ''}
					label="Connection type"
					options={[
						{ value: '', label: 'All' },
						{ value: 'tun', label: 'Full system' },
						{ value: 'proxy', label: 'Proxy only' }
					]}
					onchange={(v) => (usage.mode = (v || null) as typeof usage.mode)}
				/>
			</label>
		</div>
	</details>

	{#if usage.settings}
		{@const s = usage.settings}
		<details class="card settings" bind:this={settingsBox}>
			<summary>Data cap and privacy settings</summary>
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
		gap: 14px;
		align-content: start;
	}
	h2 {
		margin: 0;
		font-size: 15px;
		font-weight: 650;
	}
	small {
		color: var(--fg-muted);
		font-size: 11.5px;
	}
	.link {
		border: 0;
		background: none;
		padding: 2px 0;
		color: var(--accent);
		font-size: 12.5px;
	}
	.link:hover {
		text-decoration: underline;
	}
	.period {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 8px;
	}
	.dates {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.to {
		color: var(--fg-muted);
	}
	.showing {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 8px;
		padding: 8px 12px;
		border-radius: 10px;
		background: color-mix(in srgb, var(--accent) 12%, transparent);
		font-size: 12.5px;
	}
	.chip {
		padding: 2px 10px;
		border-radius: 999px;
		background: var(--accent);
		color: #fff;
		font-weight: 600;
	}
	.hero {
		padding: 20px 22px;
		display: grid;
		gap: 6px;
	}
	.lead {
		color: var(--fg-muted);
		font-size: 13.5px;
	}
	.lead em {
		font-style: normal;
		font-weight: 600;
		color: var(--ok-text);
	}
	.lead em.more {
		color: var(--warn);
	}
	.big {
		font-size: 44px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.05;
	}
	.none {
		display: grid;
		gap: 4px;
		padding: 10px 0;
		color: var(--fg-muted);
	}
	.none strong {
		color: var(--fg);
		font-size: 16px;
	}
	.split {
		display: grid;
		gap: 8px;
		margin-top: 12px;
	}
	.bar {
		display: flex;
		height: 10px;
		border-radius: 5px;
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
		gap: 4px 22px;
		font-size: 12.5px;
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
		display: flex;
		flex-wrap: wrap;
		gap: 10px 32px;
		margin-top: 12px;
		padding-top: 12px;
		border-top: 1px solid var(--hairline);
	}
	.facts div {
		display: grid;
		gap: 2px;
	}
	.facts strong {
		font-size: 14px;
	}
	.quota {
		display: grid;
		gap: 6px;
		margin-top: 14px;
		padding: 12px 14px;
		border-radius: 12px;
		background: color-mix(in srgb, var(--accent) 8%, transparent);
	}
	.quota-head {
		display: flex;
		justify-content: space-between;
		gap: 8px;
		flex-wrap: wrap;
		font-size: 12.5px;
	}
	.meter {
		height: 10px;
		border-radius: 5px;
		background: var(--hairline);
		overflow: hidden;
	}
	.meter div {
		height: 100%;
		border-radius: 5px;
		background: var(--accent);
		transition: width 0.4s;
	}
	.quota.warn .meter div {
		background: var(--warn);
	}
	.quota.over .meter div {
		background: var(--danger);
	}
	.cap-hint {
		justify-self: start;
		margin-top: 8px;
	}
	.chart-card,
	.breakdown {
		padding: 16px 18px;
		display: grid;
		gap: 10px;
	}
	.section-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 8px;
	}
	.empty {
		margin: 0;
		padding: 18px 0;
		text-align: center;
		color: var(--fg-muted);
		font-size: 13px;
	}
	.empty.small {
		padding: 8px 0;
		text-align: left;
	}
	.options {
		font-size: 12.5px;
	}
	.options summary,
	.settings summary {
		cursor: pointer;
		color: var(--fg-muted);
		font-size: 12.5px;
	}
	.options.narrow {
		padding: 12px 16px;
	}
	.options-body {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 14px;
		padding-top: 10px;
	}
	.options-body.grid2 {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
	}
	label {
		display: grid;
		gap: 4px;
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	label.check {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12.5px;
		color: var(--fg);
	}
	.list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 4px;
	}
	.item {
		width: 100%;
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		grid-template-areas: 'name amount' 'track track' 'detail detail';
		gap: 4px 12px;
		padding: 9px 10px;
		border: 0;
		border-radius: 10px;
		background: none;
		color: inherit;
		font: inherit;
		text-align: left;
	}
	.item:hover {
		background: var(--hairline);
	}
	.name {
		grid-area: name;
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
		font-weight: 600;
	}
	.text {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.tag {
		padding: 1px 7px;
		border-radius: 999px;
		background: var(--hairline);
		font-size: 10.5px;
		color: var(--fg-muted);
		font-weight: 500;
	}
	.amount {
		grid-area: amount;
		display: flex;
		align-items: baseline;
		gap: 8px;
	}
	.track {
		grid-area: track;
		height: 6px;
		border-radius: 3px;
		background: var(--hairline);
		overflow: hidden;
	}
	.track i {
		display: block;
		height: 100%;
		border-radius: 3px;
		background: var(--accent);
	}
	.detail {
		grid-area: detail;
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	.settings {
		padding: 12px 16px;
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
