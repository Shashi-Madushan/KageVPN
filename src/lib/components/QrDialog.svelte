<script lang="ts">
	import { api, type Profile } from '$lib/api';
	import { app } from '$lib/app.svelte';

	let { profile, onclose }: { profile: Profile | null; onclose: () => void } = $props();
	let dialog: HTMLDialogElement;
	let svg = $state('');
	let link = $state('');

	$effect(() => {
		if (profile && !dialog.open) {
			svg = '';
			link = '';
			dialog.showModal();
			Promise.all([api.profileQr(profile.id), api.profileLink(profile.id)])
				.then(([image, url]) => ((svg = image), (link = url)))
				.catch((e) => app.notify(String(e)));
		}
		if (!profile && dialog.open) dialog.close();
	});

	async function copy() {
		await navigator.clipboard.writeText(link).catch(() => null);
		app.notify('Link copied.', 'info');
	}
</script>

<dialog bind:this={dialog} onclose={onclose}>
	<h3>{profile?.name}</h3>
	<div class="qr">{@html svg}</div>
	<p class="hint">Scan with another device, or copy the share link. The certificate pin is not included.</p>
	<div class="actions">
		<button onclick={copy} disabled={!link}>Copy link</button>
		<button class="primary" onclick={onclose}>Done</button>
	</div>
</dialog>

<style>
	dialog {
		width: min(320px, 92vw);
		border: 1px solid var(--hairline);
		border-radius: var(--radius);
		background: var(--surface-strong);
		color: var(--fg);
		padding: 18px;
		text-align: center;
		box-shadow: 0 20px 60px rgb(0 0 0 / 0.35);
	}
	dialog::backdrop {
		background: rgb(0 0 0 / 0.3);
	}
	h3 {
		margin: 0 0 10px;
		font-size: 15px;
	}
	.qr {
		min-height: 220px;
		background: white;
		border-radius: 10px;
		padding: 6px;
		display: grid;
		place-items: center;
	}
	.qr :global(svg) {
		width: 100%;
		height: auto;
		max-width: 240px;
	}
	.hint {
		color: var(--fg-muted);
		font-size: 11px;
	}
	.actions {
		display: flex;
		justify-content: center;
		gap: 8px;
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
</style>
