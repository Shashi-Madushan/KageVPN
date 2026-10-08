<script lang="ts">
	import { app } from '$lib/app.svelte';
	import Skeleton from './Skeleton.svelte';

	type Tone = 'ok' | 'bad' | 'pending';
	const cause = $derived(app.status.reason?.cause);
	const rows = $derived.by((): { label: string; tone: Tone; note: string }[] => {
		const pending = app.verifying;
		const serverBad = cause === 'serverUnreachable' || cause === 'handshake';
		return [
			{ label: 'Engine', tone: 'ok', note: 'Running' },
			{
				label: 'Server',
				tone: pending ? 'pending' : serverBad ? 'bad' : 'ok',
				note: pending ? 'Checking…' : cause === 'serverUnreachable' ? 'Not answering' : cause === 'handshake' ? 'Rejected the connection' : 'Reachable'
			},
			{
				label: 'Internet through tunnel',
				tone: pending ? 'pending' : app.tunnelDown ? 'bad' : 'ok',
				note: pending ? 'Checking…' : app.tunnelDown ? 'No traffic' : app.status.latencyMs != null ? `${app.status.latencyMs} ms` : 'Working'
			}
		];
	});
</script>

{#if app.connected}
	<section class="card health" aria-label="Connection health">
		<span class="caption">Connection health</span>
		{#each rows as row}
			<div class="row">
				<i class={row.tone}></i>
				<span>{row.label}</span>
				{#if row.tone === 'pending'}<Skeleton width="70px" height="12px" />{:else}<small class={row.tone}>{row.note}</small>{/if}
			</div>
		{/each}
	</section>
{/if}

<style>
	.health {
		display: grid;
		gap: 8px;
		padding: 12px 14px;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 12.5px;
	}
	.row small {
		margin-left: auto;
	}
	i {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--fg-muted);
	}
	i.ok {
		background: var(--ok);
	}
	i.bad {
		background: var(--danger);
	}
	i.pending {
		background: var(--warn);
	}
	small.ok {
		color: var(--fg-muted);
	}
	small.bad {
		color: var(--danger);
	}
</style>
