<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { formatBytes, formatDuration, formatRate, profileFlag } from '$lib/format';
	import { usage } from '$lib/usage.svelte';
	import PowerButton from '$lib/components/PowerButton.svelte';
	import Segmented from '$lib/components/Segmented.svelte';
	import Sparkline from '$lib/components/Sparkline.svelte';

	const modes = [
		{ value: 'tun' as const, label: 'Full System' },
		{ value: 'proxy' as const, label: 'Proxy Only' }
	];

	let now = $state(Math.floor(Date.now() / 1000));

	onMount(() => {
		app.init();
		usage.loadToday();
		const timer = setInterval(() => (now = Math.floor(Date.now() / 1000)), 1000);
		
		const handleKey = (e: KeyboardEvent) => {
			if (e.key === 'Escape') api.hideTray();
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
		isProtected ? 'PROTECTED' : isConnecting ? 'CONNECTING…' : 'NOT CONNECTED'
	);
	const elapsed = $derived(app.status.since ? formatDuration(Math.max(0, now - app.status.since)) : null);
	const activeFlag = $derived(
		app.activeProfile ? profileFlag(app.activeProfile.name, app.activeProfile.server) : '🌐'
	);
</script>

<div class="popover">
	<header>
		<div class="status-pill {app.status.state}">
			<span class="dot"></span>
			<span>{statusBadge}</span>
		</div>
		{#if elapsed}
			<span class="time tabular">{elapsed}</span>
		{/if}
		<button class="close-btn" onclick={() => api.hideTray()} aria-label="Close tray" title="Close (Esc)">✕</button>
	</header>

	<div class="hero">
		<PowerButton state={app.status.state} size={62} disabled={app.busy || !app.activeProfile} onclick={() => app.toggle()} />
		<div class="hero-info">
			<div class="hero-name-row">
				<span class="flag">{activeFlag}</span>
				<strong class="hero-name" title={app.activeProfile?.name ?? 'No profile selected'}>
					{app.activeProfile?.name ?? 'No profile'}
				</strong>
			</div>
			<div class="hero-sub">
				{#if app.connected}
					<span class="mono ip-text">{app.exitIp ?? 'Verifying…'}</span>
					{#if app.delayMs !== null}
						<span class="ping-pill ok">{app.delayMs} ms</span>
					{/if}
				{:else}
					<span class="ready-text">{app.profiles.length ? 'Tap button to connect' : 'Add profile in app'}</span>
				{/if}
			</div>
		</div>
	</div>

	<!-- Clean server select that never overflows height -->
	<div class="server-bar card">
		<div class="server-pick-row">
			<span class="flag-icon">{activeFlag}</span>
			<select
				value={app.activeId}
				disabled={app.locked}
				aria-label="Server Profile"
				onchange={(e) => {
					if (app.auto.enabled) app.setAuto({ ...app.auto, enabled: false });
					app.select(e.currentTarget.value);
				}}
			>
				{#each app.profiles as profile (profile.id)}
					<option value={profile.id}>{profileFlag(profile.name, profile.server)} {profile.name}</option>
				{/each}
			</select>
		</div>
		{#if app.profiles.length > 1}
			<label class="auto-check">
				<input
					type="checkbox"
					checked={app.auto.enabled}
					onchange={(e) => app.setAuto({ ...app.auto, enabled: e.currentTarget.checked })}
				/>
				<span>⚡ Auto-select fastest server</span>
			</label>
		{/if}
	</div>

	<!-- Traffic stats -->
	<div class="traffic-box card">
		<div class="speeds-row">
			<div class="speed-col">
				<span class="caption">Download</span>
				<strong class="tabular down">{formatRate(app.traffic.down)}</strong>
			</div>
			<div class="speed-col">
				<span class="caption">Upload</span>
				<strong class="tabular up">{formatRate(app.traffic.up)}</strong>
			</div>
		</div>
		<div class="graph">
			<Sparkline samples={app.history} height={38} />
		</div>
		<div class="today-row">
			<span>Today: <strong class="tabular">{formatBytes((usage.today?.up ?? 0) + (usage.today?.down ?? 0))}</strong></span>
			{#if usage.quota?.limit}
				<span class="tabular">{usage.quota.percent.toFixed(0)}% of cap</span>
			{/if}
		</div>
	</div>

	<!-- Mode Toggle -->
	<div class="mode-row">
		<Segmented options={modes} value={app.mode} label="Mode" disabled={app.locked} onchange={(m) => app.setMode(m)} />
	</div>

	<!-- Footer -->
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
		justify-content: space-between;
		padding: 12px 14px 10px;
		background: color-mix(in srgb, var(--surface-strong) 92%, transparent);
		backdrop-filter: blur(24px);
		-webkit-backdrop-filter: blur(24px);
		border: 1px solid var(--hairline);
		border-radius: 12px;
		overflow: hidden;
		box-sizing: border-box;
		gap: 8px;
	}
	header {
		display: flex;
		align-items: center;
		gap: 8px;
		flex: none;
	}
	.status-pill {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 10.5px;
		font-weight: 700;
		letter-spacing: 0.04em;
		padding: 2px 8px;
		border-radius: 999px;
		background: var(--hairline);
		color: var(--fg-muted);
	}
	.status-pill.connected {
		background: color-mix(in srgb, var(--ok) 18%, transparent);
		color: var(--ok);
	}
	.status-pill.connecting {
		background: color-mix(in srgb, var(--warn) 18%, transparent);
		color: var(--warn);
	}
	.dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}
	.time {
		font-size: 11px;
		color: var(--fg-muted);
		margin-left: 2px;
	}
	.close-btn {
		margin-left: auto;
		border: 0;
		background: none;
		color: var(--fg-muted);
		font-size: 12px;
		padding: 2px 6px;
		border-radius: 4px;
		cursor: pointer;
	}
	.close-btn:hover {
		background: var(--hairline);
		color: var(--fg);
	}
	.hero {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 2px 0;
		flex: none;
	}
	.hero-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}
	.hero-name-row {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}
	.flag {
		font-size: 15px;
		flex: none;
	}
	.hero-name {
		font-size: 13.5px;
		font-weight: 650;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--fg);
	}
	.hero-sub {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.ip-text {
		font-size: 11px;
	}
	.ping-pill {
		font-size: 10px;
		padding: 1px 5px;
		border-radius: 4px;
		font-weight: 600;
	}
	.ping-pill.ok {
		background: color-mix(in srgb, var(--ok) 18%, transparent);
		color: var(--ok);
	}
	.ready-text {
		color: var(--fg-muted);
	}
	.server-bar {
		padding: 6px 8px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		flex: none;
	}
	.server-pick-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.flag-icon {
		font-size: 14px;
		flex: none;
	}
	.server-pick-row select {
		flex: 1;
		min-width: 0;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		padding: 4px 6px;
		border-radius: 6px;
		font-size: 11.5px;
	}
	.auto-check {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 10.5px;
		font-weight: 600;
		color: var(--accent);
		cursor: pointer;
		padding-left: 2px;
	}
	.traffic-box {
		padding: 8px 10px 6px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		flex: none;
	}
	.speeds-row {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}
	.speed-col {
		display: grid;
	}
	.speed-col strong {
		font-size: 14px;
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
	.today-row {
		display: flex;
		justify-content: space-between;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.today-row strong {
		color: var(--fg);
	}
	.mode-row {
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
		padding: 3px 6px;
		border-radius: 6px;
		color: var(--accent);
		font-size: 11.5px;
		font-weight: 500;
		cursor: pointer;
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
