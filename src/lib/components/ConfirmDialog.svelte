<script lang="ts">
	import Dialog from './Dialog.svelte';

	let {
		open,
		title,
		message,
		confirmLabel = 'Delete',
		onconfirm,
		onclose
	}: {
		open: boolean;
		title: string;
		message: string;
		confirmLabel?: string;
		onconfirm: () => void;
		onclose: () => void;
	} = $props();
</script>

<Dialog {open} {title} width={380} {onclose}>
	<p class="message">{message}</p>
	{#snippet footer()}
		<button class="btn" onclick={onclose}>Cancel</button>
		<button class="btn danger-solid" onclick={onconfirm}>{confirmLabel}</button>
	{/snippet}
</Dialog>

<style>
	.message {
		margin: 0;
		font-size: 13px;
		line-height: 1.5;
		color: var(--fg-muted);
	}
	.danger-solid {
		background: var(--danger);
		border-color: var(--danger);
		color: white;
		font-weight: 600;
	}
</style>
