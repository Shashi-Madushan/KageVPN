<script lang="ts">
	import { ANY_LINK, isServerLink } from '$lib/protocols';
	import { app } from '$lib/app.svelte';
	import { importMessage } from '$lib/format';
	import Dialog from './Dialog.svelte';

	let {
		open = $bindable(),
		initial = '',
		onmanual
	}: { open: boolean; initial?: string; onmanual?: () => void } = $props();

	let text = $state('');
	let error = $state<string | null>(null);
	let dragging = $state(false);
	let busy = $state(false);
	let fileInput: HTMLInputElement;

	const found = $derived(text.split(/\s+/).filter((t) => isServerLink(t)).length);

	$effect(() => {
		if (open) {
			error = null;
			dragging = false;
			text = initial;
			if (!initial) pasteFromClipboard();
		}
	});

	async function pasteFromClipboard() {
		try {
			const clip = (await navigator.clipboard.readText()).trim();
			if (ANY_LINK.test(clip)) text = clip;
		} catch {
			// clipboard access can be denied; the user can paste manually
		}
	}

	async function loadFile(file: File | undefined) {
		if (!file) return;
		try {
			text = await file.text();
			error = null;
		} catch (err) {
			error = `Could not read the file: ${err}`;
		}
	}

	function ondrop(e: DragEvent) {
		e.preventDefault();
		dragging = false;
		loadFile(e.dataTransfer?.files?.[0]);
	}

	async function save() {
		busy = true;
		const { summary, error: failure } = await app.importText(text);
		busy = false;
		error = failure;
		if (summary) {
			app.notify(importMessage(summary), 'info');
			text = '';
			open = false;
		}
	}

	function manual() {
		open = false;
		onmanual?.();
	}
</script>

<Dialog {open} title="Add server" subtitle="Paste one or more server links, or drop a text file." onclose={() => (open = false)}>
	<div
		class="drop"
		class:dragging
		ondragover={(e) => (e.preventDefault(), (dragging = true))}
		ondragleave={() => (dragging = false)}
		{ondrop}
		role="region"
		aria-label="Drop a file with links"
	>
		<textarea class="input mono" bind:value={text} rows="6" spellcheck="false" placeholder="vless://…  trojan://…  ss://…  hysteria2://…" aria-label="Server links"></textarea>
		{#if dragging}<div class="overlay">Drop the file to load its links</div>{/if}
	</div>

	<div class="meta">
		<span class="found" class:ok={found > 0}>
			{#if found > 0}{found} link{found === 1 ? '' : 's'} ready to add{:else if text.trim()}No supported links found{:else}Nothing pasted yet{/if}
		</span>
		<span class="links">
			<input type="file" accept=".txt,.json,text/plain" hidden bind:this={fileInput} onchange={(e) => loadFile(e.currentTarget.files?.[0])} />
			<button type="button" class="linkbtn" onclick={() => fileInput.click()}>Choose file…</button>
			<button type="button" class="linkbtn" onclick={manual}>Enter details manually</button>
		</span>
	</div>

	{#if error}<p class="error" role="alert">{error}</p>{/if}

	{#snippet footer()}
		<button class="btn" onclick={() => (open = false)}>Cancel</button>
		<button class="btn primary" disabled={found === 0 || busy} onclick={save}>
			{busy ? 'Adding…' : found > 1 ? `Add ${found} servers` : 'Add server'}
		</button>
	{/snippet}
</Dialog>

<style>
	.drop {
		position: relative;
	}
	textarea {
		width: 100%;
		font-size: 11.5px;
		line-height: 1.5;
		display: block;
	}
	.drop.dragging textarea {
		border-color: var(--accent);
	}
	.overlay {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		border: 2px dashed var(--accent);
		border-radius: 8px;
		background: color-mix(in srgb, var(--accent) 14%, var(--surface-strong));
		color: var(--accent);
		font-weight: 600;
		pointer-events: none;
	}
	.meta {
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 6px 12px;
		margin-top: -4px;
		font-size: 12px;
	}
	.found {
		color: var(--fg-muted);
	}
	.found.ok {
		color: var(--ok-text);
		font-weight: 600;
	}
	.links {
		display: flex;
		gap: 14px;
	}
	.error {
		margin: 0;
		padding: 8px 10px;
		border-radius: 8px;
		font-size: 12px;
		color: var(--danger);
		background: color-mix(in srgb, var(--danger) 10%, transparent);
	}
</style>
