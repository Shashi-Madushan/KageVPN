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
		<span class="caption">Health</span>
		<div class="chips">
			{#each rows as row}
				<span class="chip {row.tone}" title="{row.label}: {row.note}">
					<i></i>{row.label === 'Internet through tunnel' ? 'Internet' : row.label}
					{#if row.tone === 'pending'}<Skeleton width="34px" height="10px" />{:else if row.tone === 'bad'}<small>{row.note}</small>{/if}
				</span>
			{/each}
		</div>
	</section>
{/if}

<style>
	.health {
		display: grid;
		gap: 8px;
		padding: 12px 16px;
	}
	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 3px 10px;
		border-radius: 999px;
		font-size: 12px;
		background: color-mix(in srgb, var(--ok) 14%, transparent);
		color: var(--ok-text);
	}
	.chip.bad {
		background: color-mix(in srgb, var(--danger) 14%, transparent);
		color: var(--danger);
	}
	.chip.pending {
		background: color-mix(in srgb, var(--warn) 14%, transparent);
		color: var(--warn);
	}
	.chip small {
		font-size: 11px;
	}
	i {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: currentColor;
	}
</style>
