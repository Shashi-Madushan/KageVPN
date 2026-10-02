<script lang="ts">
	import { onMount } from 'svelte';
	import { app } from '$lib/app.svelte';
	import { core } from '$lib/core.svelte';
	import { clipboardWatch } from '$lib/clipboard.svelte';
	import ConnectionsPage from '$lib/components/ConnectionsPage.svelte';
	import CorePage from '$lib/components/CorePage.svelte';
	import DiagnosticsPage from '$lib/components/DiagnosticsPage.svelte';
	import HomePage from '$lib/components/HomePage.svelte';
	import LogView from '$lib/components/LogView.svelte';
	import ProfilesPage from '$lib/components/ProfilesPage.svelte';
	import RoutingPage from '$lib/components/RoutingPage.svelte';
	import SettingsView from '$lib/components/SettingsView.svelte';
	import Sidebar, { type Page } from '$lib/components/Sidebar.svelte';
	import Toasts from '$lib/components/Toasts.svelte';
	import UsagePage from '$lib/components/UsagePage.svelte';

	const titles: Record<Page, string> = {
		home: 'Home',
		profiles: 'Profiles',
		usage: 'Data usage',
		routing: 'Routing',
		connections: 'Connections',
		diagnostics: 'Diagnostics',
		core: 'sing-box core',
		logs: 'Logs',
		settings: 'Settings'
	};

	let page = $state<Page>('home');

	onMount(() => {
		app.init();
		core.init();
		clipboardWatch.start();
		return () => clipboardWatch.stop();
	});

	$effect(() => {
		if (app.pendingImport) page = 'profiles';
	});

	async function importOffer() {
		const text = clipboardWatch.offer;
		if (!text) return;
		clipboardWatch.dismiss();
		const { summary, error } = await app.importText(text);
		if (summary) app.notify(`Imported ${summary.added} profile${summary.added === 1 ? '' : 's'}.`, 'info');
		else if (error) app.notify(error);
	}

	const badges = $derived<Partial<Record<Page, string>>>(core.updateAvailable ? { core: '1' } : {});
</script>

<div class="app">
	<aside data-tauri-drag-region>
		<Sidebar {page} {badges} onselect={(p) => (page = p)} />
	</aside>

	<section class="content">
		<header data-tauri-drag-region><h1>{titles[page]}</h1></header>
		{#if clipboardWatch.offer}
			<div class="offer card">
				<span>A <span class="mono">vless://</span> link was copied.</span>
				<span class="spacer"></span>
				<button onclick={() => clipboardWatch.dismiss()}>Dismiss</button>
				<button class="primary" onclick={importOffer}>Import</button>
			</div>
		{/if}
		<main class:fill={page === 'logs'}>
			{#if page === 'home'}
				<HomePage onnavigate={(p) => (page = p)} />
			{:else if page === 'profiles'}
				<ProfilesPage />
			{:else if page === 'usage'}
				<UsagePage />
			{:else if page === 'routing'}
				<RoutingPage />
			{:else if page === 'connections'}
				<ConnectionsPage />
			{:else if page === 'diagnostics'}
				<DiagnosticsPage />
			{:else if page === 'core'}
				<CorePage />
			{:else if page === 'logs'}
				<LogView />
			{:else}
				<SettingsView />
			{/if}
		</main>
	</section>
	<Toasts />
</div>

<style>
	.app {
		height: 100vh;
		overflow: hidden;
		display: grid;
		grid-template-columns: 188px minmax(0, 1fr);
		grid-template-rows: minmax(0, 1fr);
	}
	aside {
		padding-top: 52px;
		overflow: hidden;
	}
	.content {
		display: flex;
		flex-direction: column;
		min-width: 0;
		min-height: 0;
		overflow: hidden;
		background: var(--content-bg);
		border-left: 1px solid var(--hairline);
	}
	header {
		padding: 14px 20px 6px;
		flex: none;
	}
	h1 {
		margin: 0;
		font-size: 18px;
		font-weight: 650;
	}
	.offer {
		margin: 0 20px 6px;
		padding: 8px 12px;
		display: flex;
		align-items: center;
		gap: 8px;
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 10%, var(--surface));
	}
	.offer .spacer {
		flex: 1;
	}
	.offer button {
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		border-radius: 8px;
		padding: 4px 12px;
	}
	.offer .primary {
		background: var(--accent);
		border-color: var(--accent);
		color: white;
		font-weight: 600;
	}
	main {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		overscroll-behavior: contain;
		padding: 8px 20px 20px;
	}
	main.fill {
		display: flex;
		flex-direction: column;
		min-height: 0;
	}
</style>
