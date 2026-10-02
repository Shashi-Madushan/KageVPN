<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		open,
		title,
		subtitle,
		width = 440,
		onclose,
		bare = false,
		children,
		footer
	}: {
		open: boolean;
		title: string;
		subtitle?: string;
		width?: number;
		onclose: () => void;
		/** Hides the title bar so the body can use the whole dialog (the title stays as its accessible name). */
		bare?: boolean;
		children: Snippet;
		footer?: Snippet;
	} = $props();

	let dialog: HTMLDialogElement;
	const titleId = `dialog-${Math.random().toString(36).slice(2, 8)}`;

	$effect(() => {
		if (open && !dialog.open) dialog.showModal();
		if (!open && dialog.open) dialog.close();
	});

	// A click on the backdrop lands on the <dialog> element itself, not its content.
	function onclick(event: MouseEvent) {
		if (event.target === dialog) onclose();
	}
</script>

<dialog bind:this={dialog} style:--dialog-w="{width}px" aria-labelledby={titleId} {onclick} onclose={onclose}>
	{#if bare}
		<span id={titleId} class="sr-only">{title}</span>
		<button type="button" class="x floating" onclick={onclose} aria-label="Close">✕</button>
	{:else}
		<header>
			<div>
				<h2 id={titleId}>{title}</h2>
				{#if subtitle}<p>{subtitle}</p>{/if}
			</div>
			<button type="button" class="x" onclick={onclose} aria-label="Close">✕</button>
		</header>
	{/if}
	<div class="body" class:bare>{@render children()}</div>
	{#if footer}<footer>{@render footer()}</footer>{/if}
</dialog>

<style>
	dialog {
		width: min(var(--dialog-w), calc(100vw - 32px));
		max-height: calc(100vh - 48px);
		padding: 0;
		border: 1px solid var(--hairline);
		border-radius: 12px;
		background: var(--dialog-bg);
		color: var(--fg);
		box-shadow: 0 24px 64px rgb(0 0 0 / 0.4);
		overflow: hidden;
	}
	dialog[open] {
		display: flex;
		flex-direction: column;
	}
	dialog::backdrop {
		background: rgb(0 0 0 / 0.35);
	}
	header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
		padding: 16px 16px 4px 20px;
	}
	h2 {
		margin: 0;
		font-size: 15px;
		font-weight: 650;
	}
	header p {
		margin: 3px 0 0;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.x {
		flex: none;
		width: 28px;
		height: 28px;
		margin-top: -2px;
		border: 0;
		border-radius: 6px;
		background: none;
		color: var(--fg-muted);
		font-size: 12px;
	}
	.x:hover {
		background: var(--hairline);
		color: var(--fg);
	}
	.body {
		padding: 12px 20px 16px;
		overflow-y: auto;
		display: grid;
		gap: 14px;
		align-content: start;
	}
	.body.bare {
		padding: 0;
		gap: 0;
	}
	.x.floating {
		position: absolute;
		z-index: 2;
		top: 10px;
		right: 10px;
		margin: 0;
	}
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip: rect(0 0 0 0);
		white-space: nowrap;
	}
	.body > :global(*) {
		min-width: 0;
	}
	footer {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		padding: 12px 20px;
		border-top: 1px solid var(--hairline);
		background: color-mix(in srgb, var(--hairline) 30%, transparent);
	}
</style>
