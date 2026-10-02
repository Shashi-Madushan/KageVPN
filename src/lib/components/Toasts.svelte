<script lang="ts">
	import { fly } from 'svelte/transition';
	import { app } from '$lib/app.svelte';
</script>

<div class="toasts" aria-live="polite">
	{#each app.toasts as toast (toast.id)}
		<div
			class="toast {toast.kind}"
			role={toast.kind === 'error' ? 'alert' : 'status'}
			transition:fly={{ y: 12, duration: 180 }}
		>
			<span class="icon">{toast.kind === 'error' ? '⚠️' : 'ℹ️'}</span>
			<div class="message">{toast.message}</div>
			<button
				class="close"
				onclick={() => app.dismiss(toast.id)}
				aria-label="Dismiss notification"
				title="Dismiss"
			>
				✕
			</button>
		</div>
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
		z-index: 1000;
		pointer-events: none;
	}
	.toast {
		pointer-events: auto;
		display: flex;
		align-items: flex-start;
		gap: 8px;
		border: 1px solid var(--hairline);
		border-left: 3px solid var(--accent);
		background: var(--surface-strong);
		border-radius: var(--radius-sm);
		padding: 9px 12px;
		font-size: 12px;
		box-shadow: 0 8px 24px rgb(0 0 0 / 0.3);
		user-select: text;
	}
	.icon {
		font-size: 13px;
		flex: none;
		line-height: 1.4;
	}
	.message {
		flex: 1;
		min-width: 0;
		word-break: break-word;
		line-height: 1.4;
	}
	.close {
		flex: none;
		border: 0;
		background: none;
		color: var(--fg-muted);
		font-size: 11px;
		padding: 2px 4px;
		border-radius: 4px;
		cursor: pointer;
		line-height: 1;
	}
	.close:hover {
		background: var(--hairline);
		color: var(--fg);
	}
	.error {
		border-left-color: var(--danger);
	}
</style>
