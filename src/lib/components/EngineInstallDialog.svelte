<script lang="ts">
	import { core } from '$lib/core.svelte';
	import { formatBytes } from '$lib/format';
	import Dialog from './Dialog.svelte';

	let { open }: { open: boolean } = $props();

	let looking = $state(false);
	let error = $state<string | null>(null);

	// Look up the newest stable release when the prompt opens so the user sees what they approve.
	$effect(() => {
		if (!open || core.releases.length || looking) return;
		looking = true;
		error = null;
		core.check().finally(() => {
			looking = false;
			if (!core.releases.length) error = 'Could not reach GitHub. Check your connection and try again.';
		});
	});

	const release = $derived(core.newest);
	const close = () => (core.promptInstall = false);
</script>

<Dialog {open} title="Download the VPN engine" subtitle="One-time setup" width={420} onclose={close}>
	<p class="text">
		SingBox GUI needs the sing-box engine to connect. It will be downloaded from the official SagerNet release on GitHub, checked against its published checksum, and kept inside this app's own folder.
	</p>
	<p class="meta">
		{#if release}
			sing-box {release.version}{release.size ? ` · ${formatBytes(release.size)}` : ''}
		{:else if error}
			<span class="bad">{error}</span>
		{:else}
			Finding the latest version…
		{/if}
	</p>
	<p class="text note">The download runs in the background, so you can keep using the app.</p>
	{#snippet footer()}
		<button class="btn" onclick={close}>Not now</button>
		<button class="btn primary" disabled={!release} onclick={() => release && core.installBackground(release.version)}>Download</button>
	{/snippet}
</Dialog>

<style>
	.text {
		margin: 0 0 10px;
		line-height: 1.5;
		color: var(--fg-muted);
		font-size: 13px;
	}
	.meta {
		margin: 0 0 10px;
		font-size: 13px;
		font-weight: 600;
	}
	.bad {
		color: var(--danger);
		font-weight: 400;
	}
	.note {
		margin: 0;
		font-size: 12px;
	}
</style>
