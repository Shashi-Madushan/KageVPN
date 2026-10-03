<script lang="ts">
	import { app } from '$lib/app.svelte';

	let { align = 'center' }: { align?: 'center' | 'start' } = $props();
</script>

{#if app.info.brand}
	<div class="powered" class:start={align === 'start'}>
		<span>Powered by</span>
		<!-- The artwork is dark ink on a light backdrop, so it always sits on its own white chip. -->
		<span class="chip"><img src="/apzur-logo.svg" alt={app.info.brand} draggable="false" /></span>
	</div>
{/if}

<style>
	.powered {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		color: var(--fg-muted);
		font-size: 12px;
		user-select: none;
	}
	.powered.start {
		justify-content: flex-start;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		padding: 3px 8px;
		border-radius: 6px;
		background: #fff;
		box-shadow: 0 0 0 1px var(--hairline);
	}
	img {
		display: block;
		height: 14px;
		width: auto;
	}
</style>
