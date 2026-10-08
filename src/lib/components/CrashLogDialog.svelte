<script lang="ts">
	import { api } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { crashLog } from '$lib/crashLog.svelte';
	import Dialog from './Dialog.svelte';

	let text = $state('');
	let loading = $state(false);

	$effect(() => {
		if (!crashLog.open) return;
		loading = true;
		api
			.crashLog()
			.then((t) => (text = t))
			.catch((e) => app.notify(String(e)))
			.finally(() => (loading = false));
	});

	async function copy() {
		await navigator.clipboard.writeText(text).catch(() => null);
		app.notify('Copied the crash log.', 'info');
	}

	async function clear() {
		await api.clearCrashLog().catch((e) => app.notify(String(e)));
		text = '';
	}
</script>

<Dialog open={crashLog.open} title="Crash log" subtitle="Every time the VPN stopped without being asked to, newest last." width={620} onclose={() => (crashLog.open = false)}>
	{#if loading && !text}
		<p class="none">Loading…</p>
	{:else if text}
		<pre class="log mono">{text}</pre>
	{:else}
		<p class="none">Nothing has crashed. Entries appear here if sing-box or the tunnel stops on its own.</p>
	{/if}
	{#snippet footer()}
		<button class="btn" disabled={!text} onclick={copy}>Copy</button>
		<button class="btn danger" disabled={!text} onclick={clear}>Clear</button>
	{/snippet}
</Dialog>

<style>
	.log {
		margin: 0;
		max-height: 50vh;
		overflow: auto;
		font-size: 11.5px;
		line-height: 1.5;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		user-select: text;
	}
	.none {
		margin: 0;
		color: var(--fg-muted);
		font-size: 13px;
	}
</style>
