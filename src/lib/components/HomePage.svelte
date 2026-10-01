<script lang="ts">
	import { app } from '$lib/app.svelte';
	import { core } from '$lib/core.svelte';
	import { usage } from '$lib/usage.svelte';
	import { formatBytes, formatDuration, formatRate } from '$lib/format';
	import HintBanner from './HintBanner.svelte';
	import PowerButton from './PowerButton.svelte';
	import Segmented from './Segmented.svelte';
	import Sparkline from './Sparkline.svelte';
	import StatTile from './StatTile.svelte';
	import type { Page } from './Sidebar.svelte';

	let { onnavigate }: { onnavigate: (page: Page) => void } = $props();

	const modes = [
		{ value: 'tun' as const, label: 'TUN (system-wide)' },
		{ value: 'proxy' as const, label: 'Proxy only' }
	];

	let now = $state(Math.floor(Date.now() / 1000));
	$effect(() => {
		usage.loadToday();
		const clock = setInterval(() => (now = Math.floor(Date.now() / 1000)), 1000);
		const refresh = setInterval(() => app.connected && usage.loadToday(), 15000);
		return () => {
			clearInterval(clock);
			clearInterval(refresh);
		};
	});

	const statusText = $derived(
		{ disconnected: 'Not connected', connecting: 'Connecting…', connected: 'Connected' }[app.status.state]
	);
	const elapsed = $derived(app.status.since ? formatDuration(Math.max(0, now - app.status.since)) : null);
</script>

<div class="home">
	{#if core.missing}
		<div class="banner card">
			<span>No sing-box core was found, so the VPN cannot start.</span>
			<button onclick={() => onnavigate('core')}>Install a core</button>
		</div>
	{:else if core.updateAvailable && core.newest}
		<div class="banner card info">
			<span>sing-box {core.newest.version} is available.</span>
			<button onclick={() => onnavigate('core')}>View update</button>
		</div>
	{/if}

	<div class="grid">
		<section class="hero card" class:on={app.connected}>
			<PowerButton state={app.status.state} disabled={app.busy || !app.activeProfile} onclick={() => app.toggle()} />
			<div class="status">
				<strong class={app.status.state}>{statusText}</strong>
				<span>{app.activeProfile?.name ?? 'No profile selected'}{#if elapsed}{' · '}<span class="tabular">{elapsed}</span>{/if}</span>
			</div>
			{#if app.connected}
				<div class="chips">
					{#if app.exitIp}<span class="chip mono">{app.exitIp}</span>{/if}
					{#if app.delayMs !== null}<span class="chip tabular">{app.delayMs} ms</span>{/if}
					<span class="chip">{app.status.mode === 'tun' ? 'TUN' : 'Proxy'}</span>
				</div>
			{/if}
			<div class="controls">
				<Segmented options={modes} value={app.mode} label="Mode" disabled={app.locked} onchange={(m) => app.setMode(m)} />
				{#if app.profiles.length}
					<select value={app.activeId} disabled={app.locked} aria-label="Profile" onchange={(e) => app.select(e.currentTarget.value)}>
						{#each app.profiles as profile (profile.id)}<option value={profile.id}>{profile.name}</option>{/each}
					</select>
				{:else}
					<button class="link" onclick={() => onnavigate('profiles')}>Add your first profile</button>
				{/if}
			</div>
		</section>

		<section class="speed card">
			<div class="tiles">
				<StatTile label="Download" value={formatRate(app.traffic.down)} sub="{formatBytes(app.traffic.totalDown)} this session" tone="down" />
				<StatTile label="Upload" value={formatRate(app.traffic.up)} sub="{formatBytes(app.traffic.totalUp)} this session" tone="up" />
			</div>
			<Sparkline samples={app.history} height={72} />
			<div class="today">
				<span>Today <strong class="tabular">{formatBytes((usage.today?.up ?? 0) + (usage.today?.down ?? 0))}</strong></span>
				{#if usage.quota?.limit}
					<span class="tabular">{usage.quota.percent.toFixed(0)}% of cap</span>
				{/if}
			</div>
			{#if usage.quota?.limit}
				<div class="meter" class:warn={usage.quota.percent >= 80}><div style:width="{Math.min(100, usage.quota.percent)}%"></div></div>
			{/if}
		</section>
	</div>

	<HintBanner />
</div>

<style>
	.home {
		display: grid;
		gap: 14px;
		align-content: start;
	}
	.grid {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		gap: 14px;
	}
	@media (max-width: 720px) {
		.grid {
			grid-template-columns: 1fr;
		}
	}
	.hero {
		display: grid;
		justify-items: center;
		align-content: start;
		gap: 12px;
		padding: 22px 16px 16px;
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
	.controls {
		display: grid;
		gap: 8px;
		width: 100%;
		margin-top: 4px;
	}
	select {
		width: 100%;
		padding: 6px 8px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
	}
	.speed {
		padding: 16px 14px 10px;
		display: grid;
		gap: 12px;
		align-content: start;
	}
	.tiles {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}
	.today {
		display: flex;
		justify-content: space-between;
		color: var(--fg-muted);
		font-size: 12px;
		padding: 0 2px;
	}
	.today strong {
		color: var(--fg);
	}
	.meter {
		height: 5px;
		border-radius: 3px;
		background: var(--hairline);
		overflow: hidden;
		margin-bottom: 4px;
	}
	.meter div {
		height: 100%;
		background: var(--accent);
	}
	.meter.warn div {
		background: var(--warn);
	}
	.banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 14px;
		border-color: var(--warn);
		background: color-mix(in srgb, var(--warn) 12%, var(--surface));
	}
	.banner.info {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 10%, var(--surface));
	}
	.banner button,
	.link {
		border: 0;
		border-radius: 8px;
		padding: 5px 12px;
		background: var(--accent);
		color: white;
		font-weight: 600;
	}
	.link {
		background: none;
		color: var(--accent);
	}
</style>
