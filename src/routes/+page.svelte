<script lang="ts">
	import { onMount } from 'svelte';
	import { app } from '$lib/app.svelte';
	import { formatBytes, formatDuration, formatRate } from '$lib/format';
	import HintBanner from '$lib/components/HintBanner.svelte';
	import ImportDialog from '$lib/components/ImportDialog.svelte';
	import LogView from '$lib/components/LogView.svelte';
	import PowerButton from '$lib/components/PowerButton.svelte';
	import ProfileList from '$lib/components/ProfileList.svelte';
	import Segmented from '$lib/components/Segmented.svelte';
	import SettingsView from '$lib/components/SettingsView.svelte';
	import Sparkline from '$lib/components/Sparkline.svelte';
	import StatTile from '$lib/components/StatTile.svelte';

	type Tab = 'home' | 'logs' | 'settings';
	const tabs: { value: Tab; label: string }[] = [
		{ value: 'home', label: 'Home' },
		{ value: 'logs', label: 'Logs' },
		{ value: 'settings', label: 'Settings' }
	];
	const modes = [
		{ value: 'tun' as const, label: 'TUN (system-wide)' },
		{ value: 'proxy' as const, label: 'Proxy only' }
	];

	let tab = $state<Tab>('home');
	let importing = $state(false);
	let now = $state(Math.floor(Date.now() / 1000));

	onMount(() => {
		app.init();
		const timer = setInterval(() => (now = Math.floor(Date.now() / 1000)), 1000);
		return () => clearInterval(timer);
	});

	const statusText = $derived(
		{ disconnected: 'Not connected', connecting: 'Connecting…', connected: 'Connected' }[app.status.state]
	);
	const elapsed = $derived(app.status.since ? formatDuration(Math.max(0, now - app.status.since)) : null);
</script>

<div class="shell">
	<header data-tauri-drag-region>
		<Segmented options={tabs} value={tab} label="Section" onchange={(t) => (tab = t)} />
	</header>

	<main>
		{#if tab === 'home'}
			<section class="hero card" class:on={app.connected}>
				<PowerButton state={app.status.state} disabled={app.busy || !app.activeProfile} onclick={() => app.toggle()} />
				<div class="status">
					<strong class={app.status.state}>{statusText}</strong>
					<span>
						{app.activeProfile?.name ?? 'No profile selected'}{#if elapsed}{' · '}<span class="tabular">{elapsed}</span>{/if}
					</span>
				</div>
				{#if app.connected}
					<div class="chips">
						{#if app.exitIp}<span class="chip mono">{app.exitIp}</span>{/if}
						{#if app.delayMs !== null}<span class="chip tabular">{app.delayMs} ms</span>{/if}
						<span class="chip">{app.status.mode === 'tun' ? 'TUN' : 'Proxy'}</span>
					</div>
				{/if}
			</section>

			<section class="speed card">
				<div class="tiles">
					<StatTile label="Download" value={formatRate(app.traffic.down)} sub="{formatBytes(app.traffic.totalDown)} total" tone="down" />
					<StatTile label="Upload" value={formatRate(app.traffic.up)} sub="{formatBytes(app.traffic.totalUp)} total" tone="up" />
				</div>
				<Sparkline samples={app.history} />
			</section>

			<Segmented options={modes} value={app.mode} label="Mode" disabled={app.locked} onchange={(m) => app.setMode(m)} />
			<HintBanner />
			<ProfileList onadd={() => (importing = true)} />
		{:else if tab === 'logs'}
			<div class="logs-root"><LogView /></div>
		{:else}
			<SettingsView />
		{/if}
	</main>
</div>

<ImportDialog bind:open={importing} />

<style>
	.shell {
		height: 100vh;
		display: flex;
		flex-direction: column;
	}
	header {
		padding: 10px 92px 8px;
		flex: none;
	}
	main {
		flex: 1;
		overflow-y: auto;
		padding: 6px 16px 18px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.hero {
		display: grid;
		justify-items: center;
		gap: 12px;
		padding: 22px 16px 18px;
		transition: background 0.3s;
	}
	.hero.on {
		background: linear-gradient(180deg, color-mix(in srgb, var(--ok) 12%, var(--surface)), var(--surface));
	}
	.status {
		display: grid;
		justify-items: center;
		color: var(--fg-muted);
	}
	.status strong {
		font-size: 16px;
		color: var(--fg);
	}
	.status .connected {
		color: var(--ok);
	}
	.status .connecting {
		color: var(--warn);
	}
	.chips {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
		justify-content: center;
	}
	.chip {
		font-size: 11px;
		padding: 2px 8px;
		border-radius: 999px;
		background: var(--hairline);
		user-select: text;
	}
	.speed {
		padding: 14px 14px 8px;
		display: grid;
		gap: 10px;
	}
	.tiles {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}
	.logs-root {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-height: 0;
	}
</style>
