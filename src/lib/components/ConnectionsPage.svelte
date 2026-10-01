<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type LiveConnection } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { formatBytes } from '$lib/format';

	let rows = $state<LiveConnection[]>([]);
	let query = $state('');
	let error = $state<string | null>(null);

	async function load() {
		if (!app.connected) {
			rows = [];
			return;
		}
		try {
			rows = await api.connections();
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
		<input type="search" placeholder="Filter by host, app or rule" bind:value={query} aria-label="Filter connections" />
		<span class="count tabular">{shown.length} open</span>
		<button class="danger" disabled={!rows.length} onclick={closeAll}>Close all</button>
	</div>

	{#if !app.connected}
		<div class="card empty">Connect to see live connections.</div>
	{:else if error}
		<div class="card empty error">{error}</div>
	{:else if shown.length === 0}
		<div class="card empty">No open connections.</div>
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
		display: flex;
		gap: 10px;
		align-items: center;
	}
	.toolbar input {
		flex: 1;
		min-width: 0;
		padding: 6px 10px;
		border-radius: 8px;
		border: 1px solid var(--hairline);
		background: var(--surface);
		user-select: text;
	}
	.count {
		color: var(--fg-muted);
		font-size: 12px;
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
	button:disabled {
		opacity: 0.4;
	}
	.empty {
		padding: 28px;
		text-align: center;
		color: var(--fg-muted);
	}
	.error {
		color: var(--danger);
	}
	.list {
		overflow: hidden;
	}
	.head,
	.row {
		display: grid;
		grid-template-columns: minmax(120px, 2.2fr) minmax(70px, 1fr) minmax(70px, 1fr) 68px 68px 44px 24px;
		gap: 8px;
		align-items: center;
		padding: 7px 12px;
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
		padding: 0;
		color: var(--fg-muted);
	}
</style>
