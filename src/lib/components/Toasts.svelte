<script lang="ts">
	import { fly } from 'svelte/transition';
	import { app } from '$lib/app.svelte';
</script>

<div class="toasts" aria-live="polite">
	{#each app.toasts as toast (toast.id)}
		<button class="toast {toast.kind}" transition:fly={{ y: 12, duration: 180 }} onclick={() => app.dismiss(toast.id)}>
			{toast.message}
		</button>
	{/each}
</div>

<style>
	.toasts {
		position: fixed;
		left: 12px;
		right: 12px;
		bottom: 12px;
		display: grid;
		gap: 8px;
		z-index: 50;
		pointer-events: none;
	}
	.toast {
		pointer-events: auto;
		text-align: left;
		border: 1px solid var(--hairline);
		border-left: 3px solid var(--accent);
		background: var(--surface-strong);
		border-radius: var(--radius-sm);
		padding: 9px 12px;
		font-size: 12px;
		box-shadow: 0 6px 20px rgb(0 0 0 / 0.25);
		user-select: text;
	}
	.error {
		border-left-color: var(--danger);
	}
</style>
