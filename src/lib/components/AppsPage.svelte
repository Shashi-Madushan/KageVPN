<script lang="ts">
	import type { RoutingSettings } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { reconcileDraft } from '$lib/appRules';
	import type { Page } from '$lib/pages';
	import AppsTab from './AppsTab.svelte';

	let { onnavigate }: { onnavigate?: (page: Page) => void } = $props();

	// Per-app choices are routing rules underneath, so they are edited in a copy of the routing
	// settings and saved like the Routing page does. Caps and blocks made by a cap are not part of
	// the copy's edits: they are kept as stored.
	let draft = $state<RoutingSettings | null>(null);
	let synced = '';

	$effect(() => {
		if (!app.routing) return;
		const next = reconcileDraft(draft ? ($state.snapshot(draft) as RoutingSettings) : null, $state.snapshot(app.routing) as RoutingSettings, synced);
		synced = next.synced;
		if (next.changed) draft = next.draft;
	});

	const dirty = $derived(draft !== null && JSON.stringify($state.snapshot(draft)) !== JSON.stringify($state.snapshot(app.routing)));
</script>

<div class="page">
	{#if draft}
		<AppsTab {draft} {onnavigate} />
		{#if dirty}
			<div class="savebar" role="status">
				<span>You have unsaved changes. They apply the next time you connect.</span>
				<button class="btn" onclick={() => app.routing && (draft = structuredClone($state.snapshot(app.routing) as RoutingSettings))}>Discard</button>
				<button class="btn primary" onclick={() => draft && app.saveRouting($state.snapshot(draft) as RoutingSettings)}>Save changes</button>
			</div>
		{/if}
	{:else}
		<p class="loading">Reading your settings…</p>
	{/if}
</div>

<style>
	.page {
		display: grid;
		gap: 16px;
		align-content: start;
	}
	.loading {
		color: var(--fg-muted);
		text-align: center;
		margin: 0;
		padding: 24px;
	}
	.savebar {
		position: sticky;
		bottom: 0;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-radius: 10px;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		box-shadow: 0 8px 24px rgb(0 0 0 / 0.25);
	}
	.savebar span {
		flex: 1;
	}
</style>
