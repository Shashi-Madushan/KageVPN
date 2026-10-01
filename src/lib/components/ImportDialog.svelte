<script lang="ts">
	import { app } from '$lib/app.svelte';

	let { open = $bindable(), initial = '' }: { open: boolean; initial?: string } = $props();
	let text = $state('');
	let error = $state<string | null>(null);
	let dialog: HTMLDialogElement;

	const found = $derived(text.split(/\s+/).filter((t) => t.startsWith('vless://')).length);

	$effect(() => {
		if (open && !dialog.open) {
			error = null;
			text = initial;
			dialog.showModal();
			if (!initial) pasteFromClipboard();
		}
		if (!open && dialog.open) dialog.close();
	});

	async function pasteFromClipboard() {
		try {
			const clip = (await navigator.clipboard.readText()).trim();
			if (clip.includes('vless://')) text = clip;
		} catch {
			// clipboard access can be denied; the user can paste manually
		}
	}

	async function save() {
		const { summary, error: failure } = await app.importText(text);
		error = failure;
		if (summary) {
			app.notify(
				summary.failed.length ? `Imported ${summary.added}, skipped ${summary.failed.length}.` : `Imported ${summary.added} profile${summary.added === 1 ? '' : 's'}.`,
				'info'
			);
			text = '';
			open = false;
		}
	}
</script>

<dialog bind:this={dialog} onclose={() => (open = false)}>
	<h3>Import profiles</h3>
	<p class="hint">Paste one or many <span class="mono">vless://</span> links, one per line.</p>
	<textarea bind:value={text} placeholder="vless://…" rows="7" spellcheck="false" class="mono"></textarea>
	{#if text}<p class="hint">{found} link{found === 1 ? '' : 's'} detected</p>{/if}
	{#if error}<p class="error">{error}</p>{/if}
	<div class="actions">
		<button onclick={() => (open = false)}>Cancel</button>
		<button class="primary" disabled={found === 0} onclick={save}>Import {found > 1 ? `${found} profiles` : ''}</button>
	</div>
</dialog>

<style>
	dialog {
		width: min(420px, 92vw);
		border: 1px solid var(--hairline);
		border-radius: var(--radius);
		background: var(--surface-strong);
		color: var(--fg);
		padding: 18px;
		box-shadow: 0 20px 60px rgb(0 0 0 / 0.35);
	}
	dialog::backdrop {
		background: rgb(0 0 0 / 0.3);
	}
	h3 {
		margin: 0;
		font-size: 15px;
	}
	.hint {
		margin: 4px 0 10px;
		color: var(--fg-muted);
		font-size: 12px;
	}
	textarea {
		width: 100%;
		resize: none;
		background: var(--surface);
		border: 1px solid var(--hairline);
		border-radius: var(--radius-sm);
		padding: 9px;
		font-size: 11px;
		user-select: text;
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 14px;
	}
	button {
		border: 1px solid var(--hairline);
		background: var(--surface);
		border-radius: 8px;
		padding: 6px 16px;
	}
	button.primary {
		background: var(--accent);
		border-color: var(--accent);
		color: white;
		font-weight: 600;
	}
	.error {
		color: var(--danger);
		font-size: 12px;
		margin: 8px 0 0;
	}
</style>
