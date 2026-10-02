<script lang="ts">
	import { app } from '$lib/app.svelte';
	import { core } from '$lib/core.svelte';
	import { usage } from '$lib/usage.svelte';
	import { formatBytes, formatDuration, formatRate, latencyLabel, latencyTone, profileFlag } from '$lib/format';
	import HintBanner from './HintBanner.svelte';
	import PowerButton from './PowerButton.svelte';
	import Segmented from './Segmented.svelte';
	import Sparkline from './Sparkline.svelte';
	import StatTile from './StatTile.svelte';
	import type { Page } from './Sidebar.svelte';

	let { onnavigate }: { onnavigate: (page: Page) => void } = $props();

	const modes = [
		{ value: 'tun' as const, label: 'TUN (Full System)' },
		{ value: 'proxy' as const, label: 'Proxy Only' }
	];

	let now = $state(Math.floor(Date.now() / 1000));
	let showQuickPicker = $state(false);

	$effect(() => {
		usage.loadToday();
		const clock = setInterval(() => (now = Math.floor(Date.now() / 1000)), 1000);
		const refresh = setInterval(() => app.connected && usage.loadToday(), 15000);
		return () => {
			clearInterval(clock);
			clearInterval(refresh);
		};
	});

	const isProtected = $derived(app.status.state === 'connected');
	const isConnecting = $derived(app.status.state === 'connecting');
	const statusText = $derived(
		isProtected ? 'Protected' : isConnecting ? 'Connecting…' : 'Unprotected'
	);
	const statusSub = $derived(
		isProtected
			? `${app.status.mode === 'tun' ? 'System traffic encrypted' : 'Local proxy active'}`
			: isConnecting
				? 'Negotiating handshake…'
				: 'Your traffic is unencrypted'
	);
	const elapsed = $derived(app.status.since ? formatDuration(Math.max(0, now - app.status.since)) : null);
	const activeFlag = $derived(
		app.activeProfile ? profileFlag(app.activeProfile.name, app.activeProfile.server) : '🌐'
	);

	async function copyExitIp() {
		if (!app.exitIp) return;
		await navigator.clipboard.writeText(app.exitIp).catch(() => null);
		app.notify(`Copied IP: ${app.exitIp}`, 'info');
	}
</script>

<div class="home">
	{#if core.missing}
		<div class="banner card danger">
			<div class="banner-text">
				<strong>sing-box engine not installed</strong>
				<small>The VPN core binary is missing, so connections cannot start.</small>
			</div>
			<button class="primary" onclick={() => onnavigate('core')}>Install Engine</button>
		</div>
	{:else if core.updateAvailable && core.newest}
		<div class="banner card info">
			<div class="banner-text">
				<strong>Core update available: {core.newest.version}</strong>
				<small>A newer release is available from the sing-box repository.</small>
			</div>
			<button onclick={() => onnavigate('core')}>Update</button>
		</div>
	{/if}

	{#if app.status.error}
		<div class="banner card error-box">
			<div class="banner-text">
				<strong>Connection failed</strong>
				<small>{app.status.error}</small>
			</div>
			<button class="primary" onclick={() => app.toggle()}>Retry</button>
		</div>
	{/if}

	<!-- First-run onboarding banner when no profiles exist -->
	{#if app.profiles.length === 0}
		<section class="onboarding card">
			<div class="onboard-header">
				<h3>Welcome to SingBox GUI</h3>
				<p>Get your private connection running in 3 simple steps:</p>
			</div>
			<div class="steps-grid">
				<div class="step-card">
					<span class="step-num">{core.missing ? '1' : '✓'}</span>
					<div>
						<strong>Core Engine</strong>
						<small>{core.missing ? 'Install sing-box binary' : 'Core ready'}</small>
					</div>
					{#if core.missing}
						<button onclick={() => onnavigate('core')}>Install</button>
					{/if}
				</div>
				<div class="step-card active">
					<span class="step-num">2</span>
					<div>
						<strong>Add Server Profile</strong>
						<small>Paste a vless:// link or subscription URL</small>
					</div>
					<button class="primary" onclick={() => onnavigate('profiles')}>Add Profile</button>
				</div>
				<div class="step-card">
					<span class="step-num">3</span>
					<div>
						<strong>Connect</strong>
						<small>Click Power button to secure traffic</small>
					</div>
				</div>
			</div>
		</section>
	{/if}

	<div class="grid">
		<section class="hero card" class:on={app.connected}>
			<!-- Protection State Badge -->
			<div class="protection-badge {app.status.state}">
				<span class="dot"></span>
				<span>{statusText.toUpperCase()}</span>
				{#if app.reliability.killSwitch}
					<span class="ks-pill" title="Kill switch is active">🛡️ Kill Switch</span>
				{/if}
			</div>

			<PowerButton state={app.status.state} disabled={app.busy || !app.activeProfile} onclick={() => app.toggle()} />

			<div class="status">
				<strong class="state-title {app.status.state}">{statusText}</strong>
				<span class="state-sub">{statusSub}{#if elapsed}{' · '}<span class="tabular">{elapsed}</span>{/if}</span>
			</div>

			{#if app.connected}
				<div class="chips">
					{#if app.exitIp}
						<button class="chip mono click-chip" onclick={copyExitIp} title="Click to copy IP">
							{app.exitIp} 📋
						</button>
					{/if}
					{#if app.delayMs !== null}
						<span class="chip ok tabular">{app.delayMs} ms</span>
					{/if}
					<span class="chip">{app.status.mode === 'tun' ? 'TUN (System)' : 'Proxy Mode'}</span>
				</div>
			{/if}

			<!-- Interactive Server Card -->
			<div class="server-card">
				<div class="server-info">
					<span class="server-flag">{activeFlag}</span>
					<div class="server-details">
						<strong class="server-title">{app.auto.enabled ? '⚡ Auto (fastest)' : (app.activeProfile?.name ?? 'No profile selected')}</strong>
						<small class="server-sub mono">
							{#if app.activeProfile}
								{app.activeProfile.server}:{app.activeProfile.port} · {app.activeProfile.security}
							{:else}
								Add or select a server to begin
							{/if}
						</small>
					</div>
				</div>

				<div class="server-actions">
					{#if app.activeProfile && app.latency[app.activeProfile.id]}
						<span class="ping-pill {latencyTone(app.latency[app.activeProfile.id])}">
							{latencyLabel(app.latency[app.activeProfile.id])}
						</span>
					{/if}
					{#if app.profiles.length > 1}
						<button class="change-btn" disabled={app.locked} onclick={() => (showQuickPicker = !showQuickPicker)}>
							{showQuickPicker ? 'Close' : 'Change'}
						</button>
					{:else}
						<button class="change-btn" onclick={() => onnavigate('profiles')}>Servers</button>
					{/if}
				</div>

				{#if showQuickPicker}
					<div class="quick-picker">
						<label class="auto-opt">
							<input
								type="checkbox"
								checked={app.auto.enabled}
								onchange={(e) => {
									app.setAuto({ ...app.auto, enabled: e.currentTarget.checked });
									showQuickPicker = false;
								}}
							/>
							<span>⚡ Auto: Always use lowest latency server</span>
						</label>
						<div class="picker-list">
							{#each app.profiles as profile (profile.id)}
								<button
									class="picker-item"
									class:active={profile.id === app.activeId && !app.auto.enabled}
									onclick={() => {
										if (app.auto.enabled) app.setAuto({ ...app.auto, enabled: false });
										app.select(profile.id);
										showQuickPicker = false;
									}}
								>
									<span class="p-flag">{profileFlag(profile.name, profile.server)}</span>
									<span class="p-name">{profile.name}</span>
									<span class="p-ping {latencyTone(app.latency[profile.id])}">
										{latencyLabel(app.latency[profile.id])}
									</span>
								</button>
							{/each}
						</div>
					</div>
				{/if}
			</div>

			<div class="controls">
				<Segmented options={modes} value={app.mode} label="Connection Mode" disabled={app.locked} onchange={(m) => app.setMode(m)} />
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

			<!-- Quick Actions Toolbar -->
			<div class="quick-tools">
				<button class="tool-btn" onclick={() => onnavigate('diagnostics')}>
					<span>⚡</span> Speed Test
				</button>
				<button class="tool-btn" onclick={() => onnavigate('diagnostics')}>
					<span>🛡️</span> Leak Check
				</button>
				<button class="tool-btn" onclick={() => onnavigate('routing')}>
					<span>🔀</span> Routing Rules
				</button>
			</div>
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
		grid-template-columns: minmax(0, 1.15fr) minmax(0, 0.85fr);
		gap: 14px;
	}
	@media (max-width: 720px) {
		.grid {
			grid-template-columns: 1fr;
		}
	}
	.banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 14px;
		border-radius: var(--radius);
	}
	.banner.danger {
		border-color: var(--danger);
		background: color-mix(in srgb, var(--danger) 12%, var(--surface));
	}
	.banner.info {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 10%, var(--surface));
	}
	.banner.error-box {
		border-color: var(--danger);
		background: color-mix(in srgb, var(--danger) 14%, var(--surface));
		color: var(--danger);
	}
	.banner-text {
		display: grid;
		gap: 2px;
	}
	.banner-text strong {
		font-size: 13px;
	}
	.banner-text small {
		color: var(--fg-muted);
		font-size: 11px;
	}
	.banner button {
		border: 1px solid var(--hairline);
		border-radius: 8px;
		padding: 5px 12px;
		background: var(--surface-strong);
		font-weight: 600;
	}
	.banner button.primary {
		background: var(--accent);
		border-color: var(--accent);
		color: white;
	}
	/* Onboarding card */
	.onboarding {
		padding: 16px;
		display: grid;
		gap: 12px;
		background: linear-gradient(180deg, color-mix(in srgb, var(--accent) 8%, var(--surface)), var(--surface));
		border-color: var(--accent);
	}
	.onboard-header h3 {
		margin: 0;
		font-size: 15px;
	}
	.onboard-header p {
		margin: 2px 0 0;
		color: var(--fg-muted);
		font-size: 12px;
	}
	.steps-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 10px;
	}
	@media (max-width: 600px) {
		.steps-grid {
			grid-template-columns: 1fr;
		}
	}
	.step-card {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 12px;
		border-radius: var(--radius-sm);
		background: var(--surface-strong);
		border: 1px solid var(--hairline);
	}
	.step-card.active {
		border-color: var(--accent);
	}
	.step-num {
		width: 24px;
		height: 24px;
		border-radius: 50%;
		background: var(--hairline);
		display: grid;
		place-items: center;
		font-size: 11px;
		font-weight: 700;
		flex: none;
	}
	.step-card.active .step-num {
		background: var(--accent);
		color: white;
	}
	.step-card div {
		display: grid;
		flex: 1;
		min-width: 0;
	}
	.step-card strong {
		font-size: 12px;
	}
	.step-card small {
		font-size: 10px;
		color: var(--fg-muted);
	}
	.step-card button {
		border: 0;
		padding: 4px 8px;
		border-radius: 6px;
		font-size: 11px;
		font-weight: 600;
		background: var(--surface);
	}
	.step-card button.primary {
		background: var(--accent);
		color: white;
	}
	/* Hero */
	.hero {
		display: grid;
		justify-items: center;
		align-content: start;
		gap: 12px;
		padding: 18px 18px 16px;
		transition: background 0.3s;
	}
	.hero.on {
		background: linear-gradient(180deg, color-mix(in srgb, var(--ok) 12%, var(--surface)), var(--surface));
	}
	.protection-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 10.5px;
		font-weight: 750;
		letter-spacing: 0.05em;
		padding: 3px 10px;
		border-radius: 999px;
		background: var(--hairline);
		color: var(--fg-muted);
	}
	.protection-badge.connected {
		background: color-mix(in srgb, var(--ok) 18%, transparent);
		color: var(--ok);
	}
	.protection-badge.connecting {
		background: color-mix(in srgb, var(--warn) 18%, transparent);
		color: var(--warn);
	}
	.protection-badge .dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}
	.ks-pill {
		margin-left: 4px;
		padding: 1px 6px;
		border-radius: 4px;
		background: var(--surface-strong);
		font-size: 9.5px;
		font-weight: 600;
		color: var(--fg);
	}
	.status {
		display: grid;
		justify-items: center;
		gap: 2px;
	}
	.state-title {
		font-size: 18px;
		font-weight: 700;
		color: var(--fg);
	}
	.state-title.connected {
		color: var(--ok);
	}
	.state-title.connecting {
		color: var(--warn);
	}
	.state-sub {
		color: var(--fg-muted);
		font-size: 12px;
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
		color: var(--fg);
		border: 0;
	}
	.chip.click-chip {
		cursor: pointer;
		background: var(--surface-strong);
		border: 1px solid var(--hairline);
	}
	.chip.click-chip:hover {
		border-color: var(--accent);
	}
	.chip.ok {
		background: color-mix(in srgb, var(--ok) 18%, transparent);
		color: var(--ok);
		font-weight: 600;
	}
	/* Server card */
	.server-card {
		width: 100%;
		display: flex;
		flex-direction: column;
		background: var(--surface-strong);
		border: 1px solid var(--hairline);
		border-radius: var(--radius-sm);
		padding: 10px 12px;
		gap: 8px;
		position: relative;
	}
	.server-info {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
		flex: 1;
	}
	.server-flag {
		font-size: 22px;
		flex: none;
	}
	.server-details {
		display: grid;
		gap: 1px;
		min-width: 0;
		flex: 1;
	}
	.server-title {
		font-size: 13.5px;
		font-weight: 650;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.server-sub {
		font-size: 11px;
		color: var(--fg-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.server-actions {
		display: flex;
		align-items: center;
		justify-content: space-between;
		border-top: 1px solid var(--hairline);
		padding-top: 6px;
	}
	.ping-pill {
		font-size: 11px;
		padding: 2px 6px;
		border-radius: 4px;
		font-variant-numeric: tabular-nums;
		font-weight: 600;
	}
	.ping-pill.ok {
		color: var(--ok);
		background: color-mix(in srgb, var(--ok) 14%, transparent);
	}
	.ping-pill.warn {
		color: var(--warn);
		background: color-mix(in srgb, var(--warn) 14%, transparent);
	}
	.ping-pill.bad {
		color: var(--danger);
		background: color-mix(in srgb, var(--danger) 14%, transparent);
	}
	.change-btn {
		margin-left: auto;
		border: 1px solid var(--hairline);
		background: var(--surface);
		border-radius: 6px;
		padding: 3px 10px;
		font-size: 11px;
		font-weight: 600;
		color: var(--accent);
	}
	.quick-picker {
		display: flex;
		flex-direction: column;
		gap: 6px;
		border-top: 1px solid var(--hairline);
		padding-top: 8px;
	}
	.auto-opt {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 11px;
		font-weight: 600;
		color: var(--accent);
		cursor: pointer;
	}
	.picker-list {
		max-height: 120px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.picker-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 5px 8px;
		border-radius: 6px;
		border: 0;
		background: none;
		text-align: left;
		font-size: 12px;
	}
	.picker-item:hover {
		background: var(--hairline);
	}
	.picker-item.active {
		background: color-mix(in srgb, var(--accent) 15%, transparent);
		color: var(--accent);
		font-weight: 600;
	}
	.p-flag {
		font-size: 14px;
	}
	.p-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.p-ping {
		font-size: 10px;
	}
	.p-ping.ok {
		color: var(--ok);
	}
	.p-ping.warn {
		color: var(--warn);
	}
	.p-ping.bad {
		color: var(--danger);
	}
	.controls {
		width: 100%;
	}
	/* Speed */
	.speed {
		padding: 16px 14px 12px;
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
	}
	.meter div {
		height: 100%;
		background: var(--accent);
	}
	.meter.warn div {
		background: var(--warn);
	}
	.quick-tools {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 6px;
		margin-top: 4px;
		padding-top: 8px;
		border-top: 1px solid var(--hairline);
	}
	.tool-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 4px;
		padding: 6px 4px;
		border-radius: 6px;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		font-size: 11px;
		font-weight: 500;
	}
	.tool-btn:hover {
		background: var(--hairline);
		border-color: var(--accent);
	}
</style>
