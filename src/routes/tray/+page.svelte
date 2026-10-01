<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { formatBytes, formatDuration, formatRate } from '$lib/format';
	import { usage } from '$lib/usage.svelte';
	import PowerButton from '$lib/components/PowerButton.svelte';
	import Segmented from '$lib/components/Segmented.svelte';
	import Sparkline from '$lib/components/Sparkline.svelte';

	const modes = [
		{ value: 'tun' as const, label: 'TUN' },
		{ value: 'proxy' as const, label: 'Proxy' }
	];

	let now = $state(Math.floor(Date.now() / 1000));

	onMount(() => {
		app.init();
		usage.loadToday();
		const timer = setInterval(() => (now = Math.floor(Date.now() / 1000)), 1000);
		return () => clearInterval(timer);
	});

	const statusText = $derived(
		{ disconnected: 'Not connected', connecting: 'Connecting…', connected: 'Connected' }[app.status.state]
	);
	const elapsed = $derived(app.status.since ? formatDuration(Math.max(0, now - app.status.since)) : null);
</script>

<div class="popover">
	<header>
		<span class="dot {app.status.state}"></span>
		<strong>{statusText}</strong>
		{#if elapsed}<span class="time tabular">{elapsed}</span>{/if}
	</header>

	<div class="hero">
		<PowerButton state={app.status.state} size={76} disabled={app.busy || !app.activeProfile} onclick={() => app.toggle()} />
		<div class="who">
			<strong>{app.activeProfile?.name ?? 'No profile'}</strong>
			{#if app.connected}
				<span class="mono">{app.exitIp ?? '…'}{#if app.delayMs !== null}{' · '}{app.delayMs} ms{/if}</span>
			{:else}
				<span>{app.profiles.length ? 'Ready' : 'Open the app to add a profile'}</span>
			{/if}
		</div>
	</div>

	<div class="speeds">
		<div><span class="caption">Download</span><strong class="tabular down">{formatRate(app.traffic.down)}</strong></div>
		<div><span class="caption">Upload</span><strong class="tabular up">{formatRate(app.traffic.up)}</strong></div>
	</div>
	<Sparkline samples={app.history} height={64} />
	<div class="today">
		<span>Today <strong class="tabular">{formatBytes((usage.today?.up ?? 0) + (usage.today?.down ?? 0))}</strong></span>
		{#if usage.quota?.limit}<span class="tabular">{usage.quota.percent.toFixed(0)}% of cap</span>{/if}
	</div>

	<div class="controls">
		{#if app.profiles.length > 1}
			<select
				value={app.activeId}
				disabled={app.locked}
				aria-label="Profile"
				onchange={(e) => app.select(e.currentTarget.value)}
			>
				{#each app.profiles as profile (profile.id)}<option value={profile.id}>{profile.name}</option>{/each}
			</select>
		{/if}
		<Segmented options={modes} value={app.mode} label="Mode" disabled={app.locked} onchange={(m) => app.setMode(m)} />
	</div>

	<footer>
		<button onclick={() => api.openMainWindow()}>Open window</button>
		<button class="quit" onclick={() => app.quit()}>Quit</button>
	</footer>
</div>

<style>
	:global(body) {
		background: transparent;
	}
	.popover {
		height: 100vh;
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 14px 16px 12px;
		border: 1px solid var(--hairline);
		border-radius: 12px;
		overflow: hidden;
	}
	header {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.time {
		margin-left: auto;
		color: var(--fg-muted);
	}
	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--fg-muted);
	}
	.dot.connected {
		background: var(--ok);
	}
	.dot.connecting {
		background: var(--warn);
	}
	.hero {
		display: flex;
		align-items: center;
		gap: 14px;
	}
	.who {
		display: grid;
		min-width: 0;
		color: var(--fg-muted);
	}
	.who strong {
		color: var(--fg);
		font-size: 14px;
	}
	.speeds {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
	}
	.speeds div {
		display: grid;
	}
	.speeds strong {
		font-size: 16px;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	.today {
		display: flex;
		justify-content: space-between;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.today strong {
		color: var(--fg);
	}
	.controls {
		display: grid;
		gap: 8px;
		margin-top: auto;
	}
	select {
		width: 100%;
		padding: 6px 8px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--hairline);
		background: var(--surface);
	}
	footer {
		display: flex;
		justify-content: space-between;
		border-top: 1px solid var(--hairline);
		padding-top: 8px;
	}
	footer button {
		border: 0;
		background: none;
		padding: 4px 6px;
		border-radius: 6px;
		color: var(--accent);
	}
	footer .quit {
		color: var(--fg-muted);
	}
</style>
