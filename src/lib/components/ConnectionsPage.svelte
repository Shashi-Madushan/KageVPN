<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type LiveConnection } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import HeaderActions from './HeaderActions.svelte';
	import { formatBytes, formatRate } from '$lib/format';

	let rows = $state<LiveConnection[]>([]);
	let query = $state('');
	let error = $state<string | null>(null);

	type Tally = { up: number; down: number };
	type AppRow = { name: string; open: number; rate: Tally; total: Tally; hosts: string[] };

	let view = $state<'apps' | 'connections'>('apps');
	let expanded = $state<string | null>(null);
	/** Bytes each open connection had at the last poll, so only the change is counted. */
	let seen = new Map<string, Tally>();
	let lastPoll = 0;
	let totals = $state<Record<string, Tally>>({});
	let rates = $state<Record<string, Tally>>({});

	/** "Google Chrome Helper (Renderer)" belongs to "Google Chrome". */
	const appName = (process: string) => process.replace(/\s+Helper.*$/, '').trim() || 'Unknown app';

	/** Turns the change since the last poll into per-app rates and running totals. */
	function tally(list: LiveConnection[]) {
		const now = Date.now();
		const seconds = lastPoll ? Math.max(0.5, (now - lastPoll) / 1000) : 0;
		const nextSeen = new Map<string, Tally>();
		const nextRates: Record<string, Tally> = {};
		for (const c of list) {
			const name = appName(c.process);
			const before = seen.get(c.id);
			const base = before ?? (lastPoll ? { up: 0, down: 0 } : { up: c.up, down: c.down });
			const dUp = Math.max(0, c.up - base.up);
			const dDown = Math.max(0, c.down - base.down);
			nextSeen.set(c.id, { up: c.up, down: c.down });
			const total = (totals[name] ??= { up: 0, down: 0 });
			total.up += dUp;
			total.down += dDown;
			const rate = (nextRates[name] ??= { up: 0, down: 0 });
			if (seconds) {
				rate.up += dUp / seconds;
				rate.down += dDown / seconds;
			}
		}
		seen = nextSeen;
		rates = nextRates;
		lastPoll = now;
	}

	async function load() {
		if (!app.connected) {
			rows = [];
			seen = new Map();
			lastPoll = 0;
			rates = {};
			return;
		}
		try {
			rows = await api.connections();
			tally(rows);
			error = null;
		} catch (e) {
			error = String(e);
		}
	}

	onMount(() => {
		load();
		const timer = setInterval(load, 2000);
		return () => clearInterval(timer);
	});

	const shown = $derived(
		rows
			.filter((r) => {
				const q = query.trim().toLowerCase();
				return !q || `${r.host} ${r.destinationIp} ${r.process} ${r.chain} ${r.rule}`.toLowerCase().includes(q);
			})
			.sort((a, b) => b.down + b.up - (a.down + a.up))
	);

	const apps = $derived.by((): AppRow[] => {
		const q = query.trim().toLowerCase();
		const byName = new Map<string, AppRow>();
		for (const c of rows) {
			const name = appName(c.process);
			if (q && !`${name} ${c.host} ${c.destinationIp}`.toLowerCase().includes(q)) continue;
			const row = byName.get(name) ?? { name, open: 0, rate: rates[name] ?? { up: 0, down: 0 }, total: totals[name] ?? { up: 0, down: 0 }, hosts: [] };
			row.open++;
			const host = c.host || c.destinationIp;
			if (host && !row.hosts.includes(host) && row.hosts.length < 8) row.hosts.push(host);
			byName.set(name, row);
		}
		return [...byName.values()].sort((a, b) => b.rate.up + b.rate.down - (a.rate.up + a.rate.down) || b.total.up + b.total.down - (a.total.up + a.total.down));
	});

	function age(start: string): string {
		const seconds = Math.max(0, (Date.now() - new Date(start).getTime()) / 1000);
		if (!Number.isFinite(seconds)) return '';
		if (seconds < 90) return `${Math.round(seconds)}s`;
		if (seconds < 5400) return `${Math.round(seconds / 60)}m`;
		return `${Math.round(seconds / 3600)}h`;
	}

	async function close(id: string) {
		await api.closeConnection(id).catch((e) => app.notify(String(e)));
		load();
	}

	async function closeAll() {
		await api.closeAllConnections().catch((e) => app.notify(String(e)));
		load();
	}
</script>

<div class="page">
	<div class="toolbar">
		<input class="input" type="search" placeholder="Filter by host, app or rule" bind:value={query} aria-label="Filter connections" />
		<div class="tabs" role="tablist">
			<button role="tab" aria-selected={view === 'apps'} class:on={view === 'apps'} onclick={() => (view = 'apps')}>Apps</button>
			<button role="tab" aria-selected={view === 'connections'} class:on={view === 'connections'} onclick={() => (view = 'connections')}>Connections</button>
		</div>
		<span class="count tabular">{view === 'apps' ? apps.length : shown.length} {view === 'apps' ? 'apps' : 'open'}</span>
	</div>
	<HeaderActions>
		<button class="btn danger" disabled={!rows.length} onclick={closeAll}>Close all</button>
	</HeaderActions>

	{#if !app.connected}
		<div class="card empty">Connect to see live connections.<button class="btn primary" disabled={app.busy || !app.activeProfile} onclick={() => app.toggle()}>Connect now</button></div>
	{:else if error}
		<div class="card empty error">{error}<button class="btn" onclick={load}>Retry</button></div>
	{:else if shown.length === 0}
		<div class="card empty">No open connections.</div>
	{:else if view === 'apps'}
		<div class="card list apps">
			<div class="head"><span>App</span><span class="num">Conns</span><span class="num">↑ Sending</span><span class="num">↓ Receiving</span><span class="num">↑ Sent</span><span class="num">↓ Received</span></div>
			{#each apps as a (a.name)}
				<button class="row app-row" class:busy={a.rate.up > 20_000} onclick={() => (expanded = expanded === a.name ? null : a.name)} aria-expanded={expanded === a.name}>
					<span class="app-name" title={a.name}>{a.name}</span>
					<span class="num tabular">{a.open}</span>
					<span class="num tabular up">{a.rate.up >= 1 ? formatRate(a.rate.up) : '—'}</span>
					<span class="num tabular down">{a.rate.down >= 1 ? formatRate(a.rate.down) : '—'}</span>
					<span class="num tabular">{formatBytes(a.total.up)}</span>
					<span class="num tabular">{formatBytes(a.total.down)}</span>
				</button>
				{#if expanded === a.name}
					<div class="hosts mono">{a.hosts.join(' · ')}{a.open > a.hosts.length ? ' …' : ''}</div>
				{/if}
			{/each}
		</div>
		<small class="note">Rates are measured every 2 seconds. Sent and Received count what moved since you opened this page.</small>
	{:else}
		<div class="card list">
			<div class="head"><span>Destination</span><span>App</span><span>Route</span><span class="num">Down</span><span class="num">Up</span><span class="num">Age</span><span></span></div>
			{#each shown as row (row.id)}
				<div class="row">
					<span class="mono dest" title="{row.host || row.destinationIp}:{row.destinationPort}">{row.host || row.destinationIp}<small>:{row.destinationPort} · {row.network}</small></span>
					<span class="app">{row.process || '—'}</span>
					<span class="route" title={row.rule}>{row.chain || 'direct'}</span>
					<span class="num tabular">{formatBytes(row.down)}</span>
					<span class="num tabular">{formatBytes(row.up)}</span>
					<span class="num tabular">{age(row.start)}</span>
					<button class="x" onclick={() => close(row.id)} aria-label="Close connection">✕</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page {
		display: grid;
		gap: 12px;
		align-content: start;
	}
	.toolbar {
		flex-wrap: wrap;
		display: flex;
		gap: 10px;
		align-items: center;
	}
	.toolbar input {
		flex: 1;
		min-width: 0;
	}
	.tabs {
		display: flex;
		gap: 2px;
		padding: 2px;
		border-radius: 8px;
		background: var(--hairline);
	}
	.tabs button {
		border: 0;
		background: none;
		border-radius: 6px;
		padding: 4px 12px;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.tabs button.on {
		background: var(--surface-strong);
		color: var(--fg);
		font-weight: 600;
	}
	.apps .head,
	.apps .row {
		grid-template-columns: minmax(100px, 2fr) 48px repeat(4, minmax(64px, 1fr));
	}
	.app-row {
		width: 100%;
		border: 0;
		background: none;
		color: inherit;
		font: inherit;
		text-align: left;
	}
	.app-row:hover {
		background: var(--hairline);
	}
	.app-row.busy .up {
		color: var(--up);
		font-weight: 650;
	}
	.app-name {
		font-weight: 600;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	.hosts {
		padding: 6px 12px 8px;
		font-size: 11px;
		color: var(--fg-muted);
		background: color-mix(in srgb, var(--hairline) 50%, transparent);
		overflow-wrap: anywhere;
	}
	.note {
		color: var(--fg-muted);
		font-size: 11px;
	}
	@container (max-width: 560px) {
		.apps .head,
		.apps .row {
			grid-template-columns: minmax(0, 1fr) 76px 76px;
		}
		.apps .head > :nth-child(2),
		.apps .head > :nth-child(5),
		.apps .head > :nth-child(6),
		.apps .row > :nth-child(2),
		.apps .row > :nth-child(5),
		.apps .row > :nth-child(6) {
			display: none;
		}
	}
	.count {
		color: var(--fg-muted);
		font-size: 12px;
	}
	.empty {
		display: grid;
		justify-items: center;
		gap: 10px;
		padding: 28px;
		text-align: center;
		color: var(--fg-muted);
	}
	.error {
		color: var(--danger);
	}
	.list {
		overflow: hidden;
		container-type: inline-size;
	}
	.head {
		font-size: 11px;
		color: var(--fg-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
		border-bottom: 1px solid var(--hairline);
	}
	.row + .row {
		border-top: 1px solid var(--hairline);
	}
	.row span {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.num {
		text-align: right;
	}
	small {
		color: var(--fg-muted);
	}
	.x {
		border: 0;
		background: none;
		padding: 4px;
		min-width: 28px;
		min-height: 28px;
		color: var(--fg-muted);
	}
	.head,
	.row {
		display: grid;
		grid-template-columns: minmax(120px, 2.2fr) minmax(70px, 1fr) minmax(70px, 1fr) 68px 68px 44px 24px;
		gap: 8px;
		align-items: center;
		padding: 7px 12px;
	}
	@container (max-width: 520px) {
		.head,
		.row {
			grid-template-columns: minmax(0, 1fr) 64px 64px 24px;
		}
		.head > :nth-child(2),
		.head > :nth-child(3),
		.head > :nth-child(6),
		.row > :nth-child(2),
		.row > :nth-child(3),
		.row > :nth-child(6) {
			display: none;
		}
	}
	@container (min-width: 521px) and (max-width: 680px) {
		.head,
		.row {
			grid-template-columns: minmax(0, 2fr) minmax(0, 1fr) 64px 64px 24px;
		}
		.head > :nth-child(3),
		.head > :nth-child(6),
		.row > :nth-child(3),
		.row > :nth-child(6) {
			display: none;
		}
	}
</style>
