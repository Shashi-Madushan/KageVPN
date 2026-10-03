<script lang="ts">
	import { app } from '$lib/app.svelte';
	import ApzurLogo from './ApzurLogo.svelte';

	let { align = 'center', height = 11 }: { align?: 'center' | 'start'; height?: number } = $props();
</script>

{#if app.info.brand}
	<div class="powered" class:start={align === 'start'} aria-label="Powered by {app.info.brand}">
		<span>Powered by</span>
		<ApzurLogo {height} />
	</div>
{/if}

<style>
	.powered {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 7px;
		color: var(--fg-muted);
		font-size: 11px;
		user-select: none;
	}
	.powered.start {
		justify-content: flex-start;
	}
</style>
