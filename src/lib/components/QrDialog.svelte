<script lang="ts">
	import { api, type Profile } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import Dialog from './Dialog.svelte';

	let { profile, onclose }: { profile: Profile | null; onclose: () => void } = $props();
	let svg = $state('');
	let link = $state('');
	let copied = $state(false);

	$effect(() => {
		if (!profile) return;
		svg = '';
		link = '';
		copied = false;
		Promise.all([api.profileQr(profile.id), api.profileLink(profile.id)])
			.then(([image, url]) => ((svg = image), (link = url)))
			.catch((e) => app.notify(String(e)));
	});

	async function copy() {
		await navigator.clipboard.writeText(link).catch(() => null);
		copied = true;
		setTimeout(() => (copied = false), 1800);
	}
</script>

<Dialog open={profile !== null} title={profile?.name ?? ''} subtitle="Scan with another device to add this server." width={340} {onclose}>
	<div class="qr">{#if svg}{@html svg}{:else}<span class="loading">Generating…</span>{/if}</div>
	<div class="link-row">
		<span class="mono link" title={link}>{link || '…'}</span>
	</div>
	<p class="note">The certificate pin is not included in the link.</p>

	{#snippet footer()}
		<button class="btn" onclick={copy} disabled={!link}>{copied ? 'Copied ✓' : 'Copy link'}</button>
		<button class="btn primary" onclick={onclose}>Done</button>
	{/snippet}
</Dialog>

<style>
	.qr {
		display: grid;
		place-items: center;
		min-height: 236px;
		padding: 8px;
		border-radius: 10px;
		background: white;
		overflow: hidden;
	}
	.qr :global(svg) {
		display: block;
		width: min(100%, 236px);
		height: auto;
		aspect-ratio: 1;
	}
	.loading {
		color: #6e6e73;
		font-size: 12px;
	}
	.link-row {
		padding: 7px 10px;
		border-radius: 8px;
		background: var(--hairline);
	}
	.link {
		display: block;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 11px;
		user-select: text;
	}
	.note {
		margin: -6px 0 0;
		font-size: 11px;
		color: var(--fg-muted);
	}
</style>
