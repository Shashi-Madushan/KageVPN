<script lang="ts">
	import { app } from '$lib/app.svelte';
	import { importMessage } from '$lib/format';

	let { open = $bindable(), initial = '' }: { open: boolean; initial?: string } = $props();
	let text = $state('');
	let error = $state<string | null>(null);
	let dragging = $state(false);
	let fileInput: HTMLInputElement;
	let dialog: HTMLDialogElement;

	const found = $derived(text.split(/\s+/).filter((t) => /^vless:\/\//i.test(t)).length);

	$effect(() => {
		if (open && !dialog.open) {
			error = null;
			text = initial;
			dragging = false;
			dialog.showModal();
			if (!initial) pasteFromClipboard();
		}
		if (!open && dialog.open) dialog.close();
	});

	async function pasteFromClipboard() {
		try {
			const clip = (await navigator.clipboard.readText()).trim();
			if (/vless:\/\//i.test(clip)) text = clip;
		} catch {
			// clipboard access can be denied; the user can paste manually
		}
	}

	async function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragging = false;
		const file = e.dataTransfer?.files?.[0];
		if (file) {
			try {
				const content = await file.text();
				text = content;
			} catch (err) {
				error = `Could not read file: ${err}`;
			}
		}
	}

	async function handleFile(e: Event) {
		const target = e.currentTarget as HTMLInputElement;
		const file = target.files?.[0];
		if (file) {
			try {
				const content = await file.text();
				text = content;
			} catch (err) {
				error = `Could not read file: ${err}`;
			}
		}
	}

	async function save() {
		const { summary, error: failure } = await app.importText(text);
		error = failure;
		if (summary) {
			app.notify(importMessage(summary), 'info');
			text = '';
			open = false;
		}
	}
</script>

<dialog bind:this={dialog} onclose={() => (open = false)}>
	<h3>Import profiles</h3>
	<p class="hint">Paste <span class="mono">vless://</span> links, or drag and drop a text file.</p>

	<div
		class="dropzone"
		class:dragging
		ondragover={(e) => { e.preventDefault(); dragging = true; }}
		ondragleave={() => (dragging = false)}
		ondrop={handleDrop}
		role="region"
		aria-label="Drop file zone"
	>
		<textarea
			bind:value={text}
			placeholder="vless://…"
			rows="7"
			spellcheck="false"
			class="mono"
			aria-label="VLESS Links"
		></textarea>
		{#if dragging}
			<div class="drop-overlay">Drop file to load links</div>
		{/if}
	</div>

	<div class="file-row">
		<input type="file" bind:this={fileInput} accept=".txt,.json" class="file-hidden" onchange={handleFile} />
		<button type="button" class="file-btn" onclick={() => fileInput.click()}>📁 Choose text file…</button>
		{#if text}<span class="hint count">{found} link{found === 1 ? '' : 's'} detected</span>{/if}
	</div>

	{#if error}<p class="error">{error}</p>{/if}
	<div class="actions">
		<button onclick={() => (open = false)}>Cancel</button>
		<button class="primary" disabled={found === 0} onclick={save}>Import {found > 1 ? `${found} profiles` : ''}</button>
	</div>
</dialog>

<style>
	dialog {
		width: min(440px, 92vw);
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
	.dropzone {
		position: relative;
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
		display: block;
	}
	.dropzone.dragging textarea {
		border-color: var(--accent);
	}
	.drop-overlay {
		position: absolute;
		inset: 0;
		background: color-mix(in srgb, var(--accent) 18%, var(--surface-strong));
		border: 2px dashed var(--accent);
		border-radius: var(--radius-sm);
		display: grid;
		place-items: center;
		font-size: 13px;
		font-weight: 600;
		color: var(--accent);
		pointer-events: none;
	}
	.file-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-top: 8px;
	}
	.file-hidden {
		display: none;
	}
	.file-btn {
		border: 1px solid var(--hairline);
		background: var(--surface);
		border-radius: 6px;
		padding: 4px 10px;
		font-size: 11px;
		cursor: pointer;
	}
	.count {
		margin: 0;
		font-size: 11px;
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
