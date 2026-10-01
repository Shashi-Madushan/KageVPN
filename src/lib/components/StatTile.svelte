<script lang="ts">
	let {
		label,
		value,
		sub = '',
		tone = 'default'
	}: { label: string; value: string; sub?: string; tone?: 'default' | 'down' | 'up' } = $props();
</script>

<div class="tile">
	<span class="caption">{label}</span>
	<strong class="tabular {tone}">{value}</strong>
	{#if sub}<small class="tabular">{sub}</small>{/if}
</div>

<style>
	.tile {
		display: grid;
		gap: 2px;
		min-width: 0;
	}
	strong {
		font-size: 17px;
		font-weight: 600;
		white-space: nowrap;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	small {
		color: var(--fg-muted);
	}
</style>
