<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { formatBytes, formatDuration, formatRate, latencyLabel, latencyTone, profileFlag } from '$lib/format';
	import { usage } from '$lib/usage.svelte';
	import PowerButton from '$lib/components/PowerButton.svelte';
	import Segmented from '$lib/components/Segmented.svelte';
	import Sparkline from '$lib/components/Sparkline.svelte';

	const modes = [
		{ value: 'tun' as const, label: 'Full System' },
		{ value: 'proxy' as const, label: 'Proxy Only' }
	];

	let now = $state(Math.floor(Date.now() / 1000));
	let showServerList = $state(false);

	onMount(() => {
		app.init();
		usage.loadToday();
		const timer = setInterval(() => (now = Math.floor(Date.now() / 1000)), 1000);
		
		const handleKey = (e: KeyboardEvent) => {
			if (e.key === 'Escape') {
				if (showServerList) {
					showServerList = false;
				} else {
					api.hideTray();
				}
			}
		};
		window.addEventListener('keydown', handleKey);

		return () => {
			clearInterval(timer);
			window.removeEventListener('keydown', handleKey);
		};
	});

	const isProtected = $derived(app.status.state === 'connected');
	const isConnecting = $derived(app.status.state === 'connecting');
	const statusBadge = $derived(
		isProtected ? 'PROTECTED' : isConnecting ? 'CONNECTING…' : 'UNPROTECTED'
	);
	const elapsed = $derived(app.status.since ? formatDuration(Math.max(0, now - app.status.since)) : null);
	const activeFlag = $derived(
		app.activeProfile ? profileFlag(app.activeProfile.name, app.activeProfile.server) : '🌐'
	);
</script>

<div class="popover">
	<header>
		<div class="badge {app.status.state}">
			<span class="dot"></span>
			<span>{statusBadge}</span>
		</div>
		{#if elapsed}
			<span class="time tabular">{elapsed}</span>
		{/if}
		<button class="close-btn" onclick={() => api.hideTray()} aria-label="Close tray" title="Close (Esc)">✕</button>
	</header>

	{#if app.status.error}
		<div class="error-banner card">
			<span>{app.status.error}</span>
			<button onclick={() => app.toggle()}>Retry</button>
		</div>
	{/if}

	<div class="hero">
		<PowerButton state={app.status.state} size={70} disabled={app.busy || !app.activeProfile} onclick={() => app.toggle()} />
		<div class="meta">
			<div class="server-title">
				<span class="flag">{activeFlag}</span>
				<strong class="server-name" title={app.activeProfile?.name ?? 'No profile selected'}>
					{app.activeProfile?.name ?? 'No profile selected'}
				</strong>
			</div>
			<div class="sub-meta">
				{#if app.connected}
					<span class="mono ip">{app.exitIp ?? 'Verifying IP…'}</span>
					{#if app.delayMs !== null}
						<span class="pill ok">{app.delayMs} ms</span>
					{/if}
				{:else}
					<span class="ready-text">{app.profiles.length ? 'Ready to connect' : 'Add a profile in app'}</span>
				{/if}
			</div>
		</div>
	</div>

	<!-- Quick Server & Auto Switcher -->
	<div class="server-selector card">
		<div class="selector-header">
			<button class="selector-toggle" disabled={app.locked || app.profiles.length === 0} onclick={() => (showServerList = !showServerList)}>
				<span class="selector-label">Location:</span>
				<strong class="selector-name">{app.auto.enabled ? '⚡ Auto (fastest)' : (app.activeProfile?.name ?? 'None')}</strong>
				<span class="arrow">{showServerList ? '▲' : '▼'}</span>
			</button>
		</div>

		{#if showServerList}
			<div class="dropdown-list">
				<label class="auto-row">
					<input
						type="checkbox"
						checked={app.auto.enabled}
						onchange={(e) => {
							app.setAuto({ ...app.auto, enabled: e.currentTarget.checked });
							showServerList = false;
						}}
					/>
					<span>⚡ Auto-select fastest server</span>
				</label>
				<div class="list-scroll">
					{#each app.profiles as profile (profile.id)}
						<button
							class="list-item"
							class:selected={profile.id === app.activeId && !app.auto.enabled}
							onclick={() => {
								if (app.auto.enabled) app.setAuto({ ...app.auto, enabled: false });
								app.select(profile.id);
								showServerList = false;
							}}
						>
							<span class="item-flag">{profileFlag(profile.name, profile.server)}</span>
							<span class="item-name">{profile.name}</span>
							<span class="item-ping {latencyTone(app.latency[profile.id])}">
								{latencyLabel(app.latency[profile.id])}
							</span>
						</button>
					{/each}
				</div>
			</div>
		{/if}
	</div>

	<div class="traffic card">
		<div class="speed-grid">
			<div>
				<span class="caption">Download</span>
				<strong class="tabular down">{formatRate(app.traffic.down)}</strong>
			</div>
			<div>
				<span class="caption">Upload</span>
				<strong class="tabular up">{formatRate(app.traffic.up)}</strong>
			</div>
		</div>
		<div class="graph">
			<Sparkline samples={app.history} height={46} />
		</div>
		<div class="quota-row">
			<span>Today: <strong class="tabular">{formatBytes((usage.today?.up ?? 0) + (usage.today?.down ?? 0))}</strong></span>
			{#if usage.quota?.limit}
				<span class="tabular">{usage.quota.percent.toFixed(0)}% of cap</span>
			{/if}
		</div>
		{#if usage.quota?.limit}
			<div class="meter" class:warn={usage.quota.percent >= 80}>
				<div style:width="{Math.min(100, usage.quota.percent)}%"></div>
			</div>
		{/if}
	</div>

	<div class="mode-control">
		<Segmented options={modes} value={app.mode} label="Connection Mode" disabled={app.locked} onchange={(m) => app.setMode(m)} />
	</div>

	<footer>
		<button class="footer-btn" onclick={() => api.openMainWindow()}>Open SingBox GUI</button>
		<button class="footer-btn quit" onclick={() => app.quit()}>Quit</button>
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
		gap: 10px;
		padding: 12px 14px 10px;
		background: color-mix(in srgb, var(--surface-strong) 92%, transparent);
		backdrop-filter: blur(24px);
		-webkit-backdrop-filter: blur(24px);
		border: 1px solid var(--hairline);
		border-radius: 12px;
		overflow: hidden;
		box-sizing: border-box;
	}
	header {
		display: flex;
		align-items: center;
		gap: 8px;
		flex: none;
	}
	.badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0.04em;
		padding: 3px 8px;
		border-radius: 999px;
		background: var(--hairline);
		color: var(--fg-muted);
	}
	.badge.connected {
		background: color-mix(in srgb, var(--ok) 18%, transparent);
		color: var(--ok);
	}
	.badge.connecting {
		background: color-mix(in srgb, var(--warn) 18%, transparent);
		color: var(--warn);
	}
	.dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: currentColor;
	}
	.time {
		font-size: 11px;
		color: var(--fg-muted);
		margin-left: 4px;
	}
	.close-btn {
		margin-left: auto;
		border: 0;
		background: none;
		color: var(--fg-muted);
		font-size: 12px;
		padding: 2px 6px;
		border-radius: 4px;
	}
	.close-btn:hover {
		background: var(--hairline);
		color: var(--fg);
	}
	.error-banner {
		padding: 6px 10px;
		font-size: 11px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		background: color-mix(in srgb, var(--danger) 14%, var(--surface));
		border-color: var(--danger);
		color: var(--danger);
	}
	.error-banner button {
		border: 1px solid var(--danger);
		background: transparent;
		color: var(--danger);
		border-radius: 4px;
		font-size: 10px;
		font-weight: 600;
		padding: 2px 6px;
	}
	.hero {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 4px 2px;
		flex: none;
	}
	.meta {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
		flex: 1;
	}
	.server-title {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}
	.flag {
		font-size: 15px;
		flex: none;
	}
	.server-name {
		font-size: 13.5px;
		font-weight: 650;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--fg);
	}
	.sub-meta {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		color: var(--fg-muted);
		flex-wrap: wrap;
	}
	.ip {
		font-size: 11px;
	}
	.pill {
		font-size: 10px;
		padding: 1px 5px;
		border-radius: 4px;
		background: var(--hairline);
	}
	.pill.ok {
		background: color-mix(in srgb, var(--ok) 18%, transparent);
		color: var(--ok);
		font-weight: 600;
	}
	.ready-text {
		color: var(--fg-muted);
	}
	.server-selector {
		padding: 4px;
		display: flex;
		flex-direction: column;
		position: relative;
	}
	.selector-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		border: 0;
		background: none;
		padding: 4px 6px;
		border-radius: 6px;
		text-align: left;
		font-size: 12px;
	}
	.selector-toggle:hover:not(:disabled) {
		background: var(--hairline);
	}
	.selector-label {
		color: var(--fg-muted);
		font-size: 11px;
	}
	.selector-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.arrow {
		font-size: 9px;
		color: var(--fg-muted);
	}
	.dropdown-list {
		margin-top: 4px;
		border-top: 1px solid var(--hairline);
		padding-top: 4px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.auto-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 6px;
		font-size: 11px;
		font-weight: 600;
		color: var(--accent);
		cursor: pointer;
	}
	.list-scroll {
		max-height: 90px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
	.list-item {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 6px;
		border: 0;
		background: none;
		border-radius: 6px;
		text-align: left;
		font-size: 11px;
	}
	.list-item:hover {
		background: var(--hairline);
	}
	.list-item.selected {
		background: color-mix(in srgb, var(--accent) 15%, transparent);
		color: var(--accent);
		font-weight: 600;
	}
	.item-flag {
		font-size: 13px;
	}
	.item-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.item-ping {
		font-size: 10px;
		color: var(--fg-muted);
	}
	.item-ping.ok {
		color: var(--ok);
	}
	.item-ping.warn {
		color: var(--warn);
	}
	.item-ping.bad {
		color: var(--danger);
	}
	.traffic {
		padding: 8px 10px 6px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.speed-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}
	.speed-grid div {
		display: grid;
	}
	.speed-grid strong {
		font-size: 15px;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	.graph {
		margin: 2px 0;
	}
	.quota-row {
		display: flex;
		justify-content: space-between;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.quota-row strong {
		color: var(--fg);
	}
	.meter {
		height: 4px;
		border-radius: 2px;
		background: var(--hairline);
		overflow: hidden;
	}
	.meter div {
		height: 100%;
		background: var(--accent);
	}
	.meter.warn div {
		background: var(--warn);
	}
	.mode-control {
		margin-top: auto;
		flex: none;
	}
	footer {
		display: flex;
		justify-content: space-between;
		border-top: 1px solid var(--hairline);
		padding-top: 6px;
		flex: none;
	}
	.footer-btn {
		border: 0;
		background: none;
		padding: 4px 6px;
		border-radius: 6px;
		color: var(--accent);
		font-size: 12px;
		font-weight: 500;
	}
	.footer-btn:hover {
		background: var(--hairline);
	}
	.footer-btn.quit {
		color: var(--fg-muted);
	}
	.footer-btn.quit:hover {
		color: var(--danger);
	}
</style>
