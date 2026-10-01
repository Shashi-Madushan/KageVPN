<script lang="ts">
	import { app } from '$lib/app.svelte';

	let { open = $bindable() }: { open: boolean } = $props();
	let link = $state('');
	let error = $state<string | null>(null);
	let dialog: HTMLDialogElement;

	$effect(() => {
		if (open && !dialog.open) {
			error = null;
			dialog.showModal();
			pasteFromClipboard();
		}
		if (!open && dialog.open) dialog.close();
	});

	async function pasteFromClipboard() {
		try {
			const text = (await navigator.clipboard.readText()).trim();
			if (text.startsWith('vless://')) link = text;
		} catch {
			// clipboard access can be denied; the user can paste manually
		}
	}

	async function save() {
		error = await app.importLink(link);
		if (!error) {
			link = '';
			open = false;
		}
	}
</script>

<dialog bind:this={dialog} onclose={() => (open = false)}>
	<h3>Import profile</h3>
	<p class="hint">Paste a <span class="mono">vless://</span> share link.</p>
	<textarea bind:value={link} placeholder="vless://…" rows="5" spellcheck="false" class="mono"></textarea>
	{#if error}<p class="error">{error}</p>{/if}
	<div class="actions">
		<button onclick={() => (open = false)}>Cancel</button>
		<button class="primary" disabled={!link.trim()} onclick={save}>Save</button>
	</div>
</dialog>

<style>
	dialog {
		width: min(380px, 92vw);
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
		margin: 2px 0 10px;
		color: var(--fg-muted);
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
