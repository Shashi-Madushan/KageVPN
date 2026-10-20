<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type LiveConnection, type RouteAction, type Rule } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import ConfirmDialog from './ConfirmDialog.svelte';
	import HeaderActions from './HeaderActions.svelte';
	import { formatBytes, formatRateOrDash } from '$lib/format';

	let rows = $state<LiveConnection[]>([]);
	let query = $state('');
	let error = $state<string | null>(null);

	type Tally = { up: number; down: number };
	/** `target` is the routing rule that would match this app: its bundle, or its process name. */
	type AppRow = { name: string; open: number; rate: Tally; total: Tally; hosts: string[]; target: Pick<Rule, 'kind' | 'value'> | null };

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

	type SortKey = 'name' | 'conns' | 'up' | 'down' | 'sent' | 'received';
	let sortKey = $state<SortKey>('down');
	let sortDesc = $state(true);

	function sortBy(key: SortKey) {
		if (sortKey === key) sortDesc = !sortDesc;
		else {
			sortKey = key;
			sortDesc = key !== 'name';
		}
	}

	const sortValue = (a: AppRow, key: SortKey): number | string =>
		({ name: a.name.toLowerCase(), conns: a.open, up: a.rate.up, down: a.rate.down, sent: a.total.up, received: a.total.down })[key];

	const columns: { key: SortKey; label: string; num: boolean }[] = [
		{ key: 'name', label: 'App', num: false },
		{ key: 'conns', label: 'Conns', num: true },
		{ key: 'up', label: '↑ Upload', num: true },
		{ key: 'down', label: '↓ Download', num: true },
		{ key: 'sent', label: '↑ Sent', num: true },
		{ key: 'received', label: '↓ Received', num: true }
	];

	const apps = $derived.by((): AppRow[] => {
		const q = query.trim().toLowerCase();
		const byName = new Map<string, AppRow>();
		for (const c of rows) {
			const name = appName(c.process);
			if (q && !`${name} ${c.host} ${c.destinationIp}`.toLowerCase().includes(q)) continue;
			const row = byName.get(name) ?? { name, open: 0, rate: rates[name] ?? { up: 0, down: 0 }, total: totals[name] ?? { up: 0, down: 0 }, hosts: [], target: null };
			row.open++;
			if (c.appPath) row.target = { kind: 'app', value: c.appPath };
			else if (!row.target && c.process) row.target = { kind: 'processName', value: c.process };
			const host = c.host || c.destinationIp;
			if (host && !row.hosts.includes(host) && row.hosts.length < 8) row.hosts.push(host);
			byName.set(name, row);
		}
		return [...byName.values()].sort((a, b) => b.rate.up + b.rate.down - (a.rate.up + a.rate.down) || b.total.up + b.total.down - (a.total.up + a.total.down));
	});

	/** The fastest app right now, so each row's activity bar is relative to it. */
	const peak = $derived(Math.max(1, ...apps.map((a) => a.rate.up + a.rate.down)));
	const sortedApps = $derived.by(() => {
		const dir = sortDesc ? -1 : 1;
		return [...apps].sort((a, b) => {
			const x = sortValue(a, sortKey);
			const y = sortValue(b, sortKey);
			const order = typeof x === 'string' ? x.localeCompare(y as string) : x - (y as number);
			return order * dir || a.name.localeCompare(b.name);
		});
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

	const actionWords: Record<RouteAction, { short: string; title: string }> = {
		proxy: { short: 'Through VPN', title: 'Send {app} through the VPN?' },
		direct: { short: 'Direct', title: 'Send {app} directly, outside the VPN?' },
		block: { short: 'Block', title: 'Block {app}?' }
	};

	let routing = $state<{ row: AppRow; action: RouteAction } | null>(null);

	/** The rule that already decides where this app's traffic goes, if any. */
	function currentRule(row: AppRow) {
		if (app.routing?.mode !== 'rule' || !row.target) return null;
		return app.routing.rules.find((r) => r.enabled && r.kind === row.target!.kind && r.value === row.target!.value) ?? null;
	}

	const routeMessage = $derived.by(() => {
		if (!routing || !app.routing) return '';
		const mode = app.routing.mode;
		const modeNote =
			mode === 'rule' ? '' : ` Routing switches to Rules mode; everything else still goes ${mode === 'direct' ? 'direct' : 'through the VPN'}.`;
		const when = app.connected ? ' The VPN reconnects to apply it.' : ' It applies the next time you connect.';
		return `This adds a rule at the top of your routing rules, so it wins over broader ones.${modeNote}${when}`;
	});

	async function applyRoute() {
		if (!routing?.row.target) return;
		const { row, action } = routing;
		routing = null;
		if (!(await app.routeApp(row.target!, action))) return;
		if (app.connected) await app.restartTunnel();
		else app.notify(`Rule added for ${row.name}.`, 'info');
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
			<div class="head" role="row">
				{#each columns as col (col.key)}
					<button class="th" class:num={col.num} class:on={sortKey === col.key} onclick={() => sortBy(col.key)} title="Sort by {col.label.replace(/^[↑↓] /, '')}">
						{col.label}{#if sortKey === col.key}<span class="arrow" aria-hidden="true">{sortDesc ? '▾' : '▴'}</span>{/if}
					</button>
				{/each}
			</div>
			{#each sortedApps as a (a.name)}
				<button class="row app-row" onclick={() => (expanded = expanded === a.name ? null : a.name)} aria-expanded={expanded === a.name}>
					<span class="app-cell">
						<span class="avatar" aria-hidden="true">{a.name.charAt(0).toUpperCase()}</span>
						<span class="app-name" title={a.name}>{a.name}</span>
					</span>
					<span class="num tabular"><span class="pill">{a.open}</span></span>
					<span class="num tabular up" class:idle={a.rate.up < 1}>{formatRateOrDash(a.rate.up)}</span>
					<span class="num tabular down" class:idle={a.rate.down < 1}>{formatRateOrDash(a.rate.down)}</span>
					<span class="num tabular dim">{formatBytes(a.total.up)}</span>
					<span class="num tabular dim">{formatBytes(a.total.down)}</span>
					<span class="meter" aria-hidden="true"><i style="width:{Math.min(100, ((a.rate.up + a.rate.down) / peak) * 100)}%"></i></span>
				</button>
				{#if expanded === a.name}
					{@const rule = currentRule(a)}
					<div class="detail">
						<div class="hosts mono">{a.hosts.join(' · ')}{a.open > a.hosts.length ? ' …' : ''}</div>
						{#if a.target}
							<div class="route-bar">
								<span>Route {a.name}{rule ? ` (now: ${actionWords[rule.action].short})` : ''}</span>
								{#each ['proxy', 'direct', 'block'] as const as action (action)}
									<button class="btn small" class:active={rule?.action === action} disabled={rule?.action === action} onclick={() => (routing = { row: a, action })}>{actionWords[action].short}</button>
								{/each}
							</div>
						{/if}
					</div>
				{/if}
			{/each}
		</div>
		<small class="note">Speeds are measured every 2 seconds. Sent and Received count what moved since you opened this page. Click a column to sort. Click an app to route it through the VPN, directly or not at all.</small>
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

<ConfirmDialog
	open={routing !== null}
	title={routing ? actionWords[routing.action].title.replace('{app}', routing.row.name) : ''}
	message={routeMessage}
	confirmLabel={app.connected ? 'Add rule and reconnect' : 'Add rule'}
	danger={routing?.action === 'block'}
	onconfirm={applyRoute}
	onclose={() => (routing = null)}
/>

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
		grid-template-columns: minmax(140px, 2fr) 56px repeat(4, minmax(92px, 1fr));
		column-gap: 12px;
	}
	.apps .head {
		position: sticky;
		top: 0;
		background: var(--surface-strong);
		padding-block: 4px;
	}
	.th {
		border: 0;
		background: none;
		color: inherit;
		font: inherit;
		text-transform: inherit;
		letter-spacing: inherit;
		text-align: left;
		padding: 6px 0;
		white-space: nowrap;
	}
	.th.num {
		text-align: right;
	}
	.th:hover,
	.th.on {
		color: var(--fg);
	}
	.arrow {
		margin-left: 4px;
		font-size: 9px;
	}
	.app-row {
		position: relative;
		width: 100%;
		border: 0;
		background: none;
		color: inherit;
		font: inherit;
		text-align: left;
		padding-block: 10px;
	}
	.app-row:hover {
		background: var(--hairline);
	}
	.app-cell {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
	}
	.avatar {
		flex: none;
		display: grid;
		place-items: center;
		width: 24px;
		height: 24px;
		border-radius: 7px;
		background: var(--hairline);
		color: var(--fg-muted);
		font-size: 12px;
		font-weight: 700;
	}
	.pill {
		display: inline-block;
		min-width: 22px;
		padding: 1px 7px;
		border-radius: 999px;
		background: var(--hairline);
		text-align: center;
		font-size: 12px;
	}
	.dim {
		color: var(--fg-muted);
	}
	.idle {
		color: var(--fg-muted);
		opacity: 0.6;
	}
	.meter {
		position: absolute;
		left: 12px;
		right: 12px;
		bottom: 3px;
		height: 2px;
		overflow: hidden;
		padding: 0 !important;
	}
	.meter i {
		display: block;
		height: 100%;
		border-radius: 2px;
		background: var(--accent);
		opacity: 0.55;
		transition: width 0.4s ease;
	}
	.app-name {
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	.detail {
		background: color-mix(in srgb, var(--hairline) 50%, transparent);
	}
	.route-bar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		padding: 0 12px 10px;
		font-size: 12px;
	}
	.route-bar span {
		flex: 1;
		min-width: 140px;
		color: var(--fg-muted);
	}
	.route-bar .btn {
		min-height: 24px;
		padding: 2px 10px;
		font-size: 12px;
	}
	.route-bar .active {
		border-color: var(--accent);
		color: var(--accent);
	}
	.hosts {
		padding: 6px 12px 8px;
		font-size: 11px;
		color: var(--fg-muted);
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
