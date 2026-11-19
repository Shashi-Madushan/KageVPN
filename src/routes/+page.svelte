<script lang="ts">
	import WindowControls from '$lib/components/WindowControls.svelte';
	import { platform } from '$lib/platform.svelte';
	import { onMount } from 'svelte';
	import { app } from '$lib/app.svelte';
	import { importMessage } from '$lib/format';
	import { header } from '$lib/header.svelte';
	import { core } from '$lib/core.svelte';
	import { appUpdate } from '$lib/appUpdate.svelte';
	import { downloads } from '$lib/downloads.svelte';
	import { clipboardWatch } from '$lib/clipboard.svelte';
	import { onboarding } from '$lib/onboarding.svelte';
	import ConnectionsPage from '$lib/components/ConnectionsPage.svelte';
	import CorePage from '$lib/components/CorePage.svelte';
	import DiagnosticsPage from '$lib/components/DiagnosticsPage.svelte';
	import HomePage from '$lib/components/HomePage.svelte';
	import DownloadsMenu from '$lib/components/DownloadsMenu.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import CrashLogDialog from '$lib/components/CrashLogDialog.svelte';
	import FeedbackDialog from '$lib/components/FeedbackDialog.svelte';
	import EngineInstallDialog from '$lib/components/EngineInstallDialog.svelte';
	import OnboardingDialog from '$lib/components/OnboardingDialog.svelte';
	import PoweredBy from '$lib/components/PoweredBy.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import LogView from '$lib/components/LogView.svelte';
	import ProfilesPage from '$lib/components/ProfilesPage.svelte';
	import RoutingPage from '$lib/components/RoutingPage.svelte';
	import SettingsView from '$lib/components/SettingsView.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import AppsPage from '$lib/components/AppsPage.svelte';
	import { pageForKey, type Page } from '$lib/pages';
	import WhatsNewPage from '$lib/components/WhatsNewPage.svelte';
	import { whatsNew } from '$lib/whatsNew.svelte';
	import UsagePage from '$lib/components/UsagePage.svelte';

	const titles: Record<Page, string> = {
		home: 'Home',
		profiles: 'Servers',
		usage: 'Data usage',
		routing: 'Routing',
		apps: 'App controls',
		connections: 'Connections',
		diagnostics: 'Diagnostics',
		core: 'Engine',
		logs: 'Logs',
		settings: 'Settings',
		whatsnew: "What's new"
	};

	const subtitles: Record<Page, string> = {
		home: 'Connection status and live traffic',
		profiles: 'Choose, import and test your servers',
		usage: '',
		routing: 'Decide which traffic goes through the VPN',
		apps: 'Choose how each app uses the VPN, and set data limits',
		connections: 'Live connections passing through the tunnel',
		diagnostics: 'Speed, leak and connectivity checks',
		core: 'The sing-box engine that powers the tunnel',
		logs: 'Technical output for troubleshooting',
		settings: 'Startup, reliability, data limit and shortcuts',
		whatsnew: 'Recent changes and new features'
	};

	let page = $state<Page>('home');

	function onkeydown(e: KeyboardEvent) {
		// Cmd on a Mac, Ctrl on Windows (where Cmd does not exist and the Windows key belongs to the system).
		if (!(platform.isMac ? e.metaKey || e.ctrlKey : e.ctrlKey) || e.altKey || e.shiftKey) return;
		const next = pageForKey(e.key);
		if (next) {
			e.preventDefault();
			page = next;
		}
	}

	onMount(() => {
		app.init().then(() => appUpdate.init());
		downloads.init();
		core.init();
		clipboardWatch.start();
		return () => clipboardWatch.stop();
	});

	$effect(() => {
		if (app.pendingImport) page = 'profiles';
	});

	// First launch with nothing set up: show the tour once. Existing users are marked as seen.
	$effect(() => {
		if (!app.loaded || onboarding.seen) return;
		if (app.profiles.length === 0) onboarding.show();
		else onboarding.markSeen();
	});

	async function importOffer() {
		const text = clipboardWatch.offer;
		if (!text) return;
		clipboardWatch.dismiss();
		const { summary, error } = await app.importText(text);
		if (summary) app.notify(importMessage(summary), 'info');
		else if (error) app.notify(error);
	}

	const badges = $derived<Partial<Record<Page, string>>>((core.updateAvailable ? { core: '1' } : {}));

	$effect(() => {
		if (page === 'whatsnew') whatsNew.markSeen();
	});
</script>

<svelte:window {onkeydown} />

<div class="app" class:windows={platform.isWindows}>
	<!-- The window has no native title bar, so this strip is what drags it and zooms it on double-click. -->
	<div class="titlebar" data-tauri-drag-region>{#if platform.isWindows}<span class="app-name">{app.info.name}</span>{/if}</div>
	{#if platform.isWindows}<WindowControls />{/if}
	<aside data-tauri-drag-region>
		<Sidebar {page} {badges} onselect={(p) => (page = p)} />
		<div class="aside-foot">
			<ThemeToggle />
			<div class="brand"><PoweredBy /></div>
		</div>
	</aside>

	<section class="content">
		<header data-tauri-drag-region><div class="titles"><h1>{titles[page]}</h1>{#if subtitles[page]}<p>{subtitles[page]}</p>{/if}</div>
			<div class="actions">
				{#if page === 'home'}
					<button class="whatsnew" class:unread={whatsNew.unread} onclick={() => (page = 'whatsnew')}>
						What's new
					</button>
				{/if}
				{#if header.actions}{@render header.actions()}{/if}
				<DownloadsMenu />
			</div>
		</header>
		{#if clipboardWatch.offer}
			<div class="offer card">
				<span>A server link was copied.</span>
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
				<UsagePage onnavigate={(p) => (page = p)} />
			{:else if page === 'routing'}
				<RoutingPage onnavigate={(p) => (page = p)} />
			{:else if page === 'apps'}
				<AppsPage />
			{:else if page === 'connections'}
				<ConnectionsPage />
			{:else if page === 'diagnostics'}
				<DiagnosticsPage />
			{:else if page === 'core'}
				<CorePage />
			{:else if page === 'logs'}
				<LogView />
			{:else if page === 'whatsnew'}
				<WhatsNewPage onnavigate={(p) => (page = p)} />
			{:else}
				<SettingsView />
			{/if}
		</main>
	</section>
</div>

<OnboardingDialog onnavigate={(p) => (page = p)} />
<EngineInstallDialog open={core.promptInstall} />
<FeedbackDialog />
<CrashLogDialog />
<ConfirmDialog
	open={appUpdate.confirmRestart}
	title="Restart and disconnect?"
	message="The VPN will disconnect while the app restarts into the new version. It does not reconnect on its own unless auto-connect is on."
	confirmLabel="Restart"
	danger={false}
	onconfirm={() => appUpdate.restart(true)}
	onclose={() => (appUpdate.confirmRestart = false)}
/>

<style>
	.app {
		position: relative;
		height: 100vh;
		overflow: hidden;
		display: grid;
		grid-template-columns: var(--sidebar-w) minmax(0, 1fr);
		grid-template-rows: minmax(0, 1fr);
	}
	.titlebar {
		position: absolute;
		inset: 0 0 auto 0;
		height: 44px;
		z-index: 1;
	}
	/* Windows: the app's name sits at the left of the title strip, as in other Windows apps. */
	.app-name {
		position: absolute;
		left: 14px;
		top: 0;
		line-height: 32px;
		font-size: 12px;
		color: var(--fg-muted);
		pointer-events: none;
	}
	/* Windows: no window buttons on the left, but the strip is still there; and the header's buttons
	   keep clear of minimize, maximize and close on the right. */
	.windows .titlebar {
		height: 32px;
	}
	.windows aside {
		padding-top: 40px;
	}
	.windows header {
		padding-top: 36px;
	}
	/* Let presses on the title text fall through to the drag strip beneath it. */
	.titles {
		pointer-events: none;
	}
	aside {
		display: flex;
		flex-direction: column;
		padding-top: 52px;
		overflow: hidden;
	}
	.aside-foot {
		margin-top: auto;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 10px 10px 14px;
	}
	.brand {
		opacity: 0.8;
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
	@media (max-width: 759px) {
		/* The icon rail is too narrow for a wordmark. */
		.brand {
			display: none;
		}
		.app {
			/* Wide enough to hold the macOS window buttons, which end about 66px from the edge. */
			grid-template-columns: var(--rail-w) minmax(0, 1fr);
		}
	}
	header {
		padding: 14px var(--page-pad) 6px;
		width: 100%;
		/* The padding sits inside this box, so add it to the limit: the title and buttons then line
		   up with the cards below, which are --content-max wide inside main's own padding. */
		max-width: calc(var(--content-max) + 2 * var(--page-pad));
		margin-inline: auto;
		flex: none;
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
	}
	.whatsnew {
		padding: 0;
		border: 0;
		background: none;
		font-size: var(--text-sm);
		color: var(--fg-muted);
	}
	.whatsnew:hover {
		color: var(--fg);
		text-decoration: underline;
	}
	.whatsnew.unread {
		color: var(--accent);
		font-weight: 600;
	}
	.actions {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 8px;
		-webkit-app-region: no-drag;
		position: relative;
		z-index: 2;
	}
	h1 {
		margin: 0;
		font-size: 18px;
		font-weight: 650;
	}
	header p {
		margin: 2px 0 0;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.offer {
		width: calc(100% - 2 * var(--page-pad));
		max-width: var(--content-max);
		margin: 0 auto 6px;
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
		padding: 8px var(--page-pad) 20px;
	}
	main > :global(*) {
		max-width: var(--content-max);
		margin-inline: auto;
	}
	main.fill {
		display: flex;
		flex-direction: column;
		min-height: 0;
	}
</style>
