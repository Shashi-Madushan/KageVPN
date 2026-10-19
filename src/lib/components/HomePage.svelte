<script lang="ts">
	import { app } from '$lib/app.svelte';
	import { privacy, privacyKey } from '$lib/privacy.svelte';
	import { protocols } from '$lib/protocols.svelte';
	import PingButton from './PingButton.svelte';
	import PrivacyToggle from './PrivacyToggle.svelte';
	import Sensitive from './Sensitive.svelte';
	import { appUpdate } from '$lib/appUpdate.svelte';
	import { core } from '$lib/core.svelte';
	import { downloads, STAGES } from '$lib/downloads.svelte';
	import { onboarding } from '$lib/onboarding.svelte';
	import { usage } from '$lib/usage.svelte';
	import { formatBytes, formatDuration, formatRate, latencyLabel, latencyTone, profileFlag } from '$lib/format';
	import Icon from './Icon.svelte';
	import HintBanner from './HintBanner.svelte';
	import HealthCard from './HealthCard.svelte';
	import Skeleton from './Skeleton.svelte';
	import StatusBanner from './StatusBanner.svelte';
	import MascotButton from './MascotButton.svelte';
	import Select from './Select.svelte';
	import Segmented from './Segmented.svelte';
	import Sparkline from './Sparkline.svelte';
	import StatTile from './StatTile.svelte';
	import type { Page } from './Sidebar.svelte';

	let { onnavigate }: { onnavigate: (page: Page) => void } = $props();

	const modes = [
		{ value: 'tun' as const, label: 'Full System (TUN)' },
		{ value: 'proxy' as const, label: 'Proxy Only' }
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

	const isConnecting = $derived(app.status.state === 'connecting');
	const statusText = $derived(
		app.status.blocked && app.reliability.killSwitch ? 'Blocked' : app.tunnelDown ? 'No Internet' : app.verifying ? 'Verifying…' : app.protectedNow ? 'Protected' : isConnecting ? 'Connecting…' : 'Not Connected'
	);
	const headline = $derived(
		app.status.blocked && app.reliability.killSwitch
			? 'Internet blocked by the kill switch'
			: app.tunnelDown
			? 'Connected, but no internet'
			: app.verifying
				? 'Checking your connection…'
				: app.protectedNow
					? "You're protected"
					: isConnecting
						? 'Connecting…'
						: "You're not protected"
	);
	const elapsed = $derived(app.status.since ? formatDuration(Math.max(0, now - app.status.since)) : null);
	const serverValue = $derived(app.auto.enabled ? 'auto' : (app.activeId ?? ''));
	const serverOptions = $derived([
		...(app.profiles.length > 1 ? [{ value: 'auto', label: 'Auto (fastest server)', icon: '↯' }] : []),
		...app.profiles.map((p) => ({
			value: p.id,
			label: p.name.replace(/^\p{RI}{2}\s*/u, ''),
			icon: profileFlag(p.name, p.server),
			hint: app.latency[p.id] ? latencyLabel(app.latency[p.id]) : undefined
		}))
	]);

	function pickServer(id: string) {
		if (id === 'auto') return app.setAuto({ ...app.auto, enabled: true });
		if (app.auto.enabled) app.setAuto({ ...app.auto, enabled: false });
		app.select(id);
	}

	async function copyExitIp() {
		if (!app.exitIp) return;
		await navigator.clipboard.writeText(app.exitIp).catch(() => null);
		app.notify(privacy.isHidden(privacyKey.exitIp) ? 'Copied IP address' : `Copied IP: ${app.exitIp}`, 'info');
	}
</script>

<div class="home">
	{#if core.downloading}
		<div class="banner card info" role="status">
			<div class="banner-text grow">
				<strong>Downloading the {core.coreName} engine…</strong>
				<small>{core.progress ? STAGES[core.progress.stage] : 'Starting'}{core.percent !== null ? ` ${core.percent.toFixed(0)}%` : ''}. You can keep using the app.</small>
				<div class="bar-track"><div class="bar-fill" class:indeterminate={core.percent === null} style:width={core.percent === null ? '100%' : `${core.percent}%`}></div></div>
			</div>
			<button onclick={() => core.cancel()}>Cancel</button>
		</div>
	{:else if core.missing}
		<div class="banner card danger">
			<div class="banner-text">
				<strong>{core.failure ? 'Engine download failed' : 'The VPN engine is not installed'}</strong>
				<small>{core.failure ?? 'Download {core.coreName} once and connections can start. It is verified before it is used.'}</small>
			</div>
			<button class="primary" onclick={() => (core.promptInstall = true)}>{core.failure ? 'Try again' : 'Download engine'}</button>
		</div>
	{:else if core.updateAvailable && core.latestVersion}
		<div class="banner card info">
			<div class="banner-text">
				<strong>Engine update available: {core.latestVersion}</strong>
				<small>{core.status?.settings.pendingVersion ? `${core.status.settings.pendingVersion} is ready and applies on the next connect.` : 'Downloads in the background and is verified before it is used.'}</small>
			</div>
			<button onclick={() => core.installBackground(core.latestVersion)}>Update</button>
		</div>
	{/if}

	{#if appUpdate.restartPending}
		<div class="banner card info" role="status">
			<div class="banner-text">
				<strong>{app.info.name} {appUpdate.restartPending} is installed</strong>
				<small>Restart to start using it.{app.locked ? ' This will disconnect the VPN.' : ''}</small>
			</div>
			<button class="primary" onclick={() => appUpdate.restart()}>Restart</button>
		</div>
	{:else if appUpdate.installing}
		<div class="banner card info" role="status">
			<div class="banner-text">
				<strong>Downloading the {app.info.name} update…</strong>
				<small>It installs when the download finishes. See the downloads menu for progress.</small>
			</div>
			<button onclick={() => downloads.items.find((i) => i.id === 'app')?.cancel?.()}>Cancel</button>
		</div>
	{:else if appUpdate.available}
		<div class="banner card info">
			<div class="banner-text">
				<strong>{app.info.name} {appUpdate.available.version} is available</strong>
				<small>{appUpdate.failure ?? 'Verified before it is installed. You choose when to restart.'}</small>
			</div>
			<button onclick={() => appUpdate.install()}>{appUpdate.failure ? 'Try again' : 'Update'}</button>
		</div>
	{/if}

	<StatusBanner {onnavigate} />

	<div class="grid">
		<!-- Connection -->
		<section class="hero card" class:on={app.connected}>
			<div class="card-header">
				<div class="status-pill {app.tunnelDown ? 'down' : app.verifying ? 'connecting' : app.status.state}">
					<span class="dot"></span>
					<strong>{statusText}</strong>
					{#if elapsed}<span class="elapsed tabular">{elapsed}</span>{/if}
				</div>
				<div class="chips">
					{#if app.connected && app.status.mode === 'proxy' && app.status.proxyPort}
						<span class="chip mono" title={app.reliability.proxyAuth ? 'Point apps at this address (SOCKS5 or HTTP). Login: user kage, password in Settings' : 'Point apps at this address (SOCKS5 or HTTP)'}>127.0.0.1:{app.status.proxyPort}</span>
					{/if}
					{#if app.reliability.killSwitch}
						<span class="chip" title="Kill switch is armed"><Icon name="shield" size={12} /> Kill switch</span>
					{/if}
					{#if app.reliability.autoReconnect}
						<span class="chip" title="Reconnects automatically if the tunnel drops"><Icon name="refresh" size={12} /> Auto-reconnect</span>
					{/if}
				</div>
			</div>

			<div class="power-center">
				<MascotButton size={150} />
				<h2 class="headline {app.tunnelDown ? 'down' : app.verifying ? 'connecting' : app.status.state}">{headline}</h2>
				<div class="ip-row">
					{#if app.connected}
						{#if app.exitIp}
							<button class="ip-chip mono" onclick={copyExitIp} title="Copy IP address"><Sensitive id={privacyKey.exitIp}>{app.exitIp}</Sensitive></button>
						{/if}
						{#if app.exitIp}<PrivacyToggle id={privacyKey.exitIp} label="IP address" />{/if}
						<PingButton />
					{:else}
						<span class="ready-sub">{app.profiles.length ? (isConnecting ? 'Establishing secure tunnel…' : 'Tap to connect') : 'Add a server to connect'}</span>
						{#if app.profiles.length && !isConnecting}<PingButton />{/if}
						{#if !app.profiles.length}<button class="linkbtn" onclick={() => onboarding.show()}>New here? Run the quick setup</button>{/if}
					{/if}
				</div>
			</div>

			<div class="controls">
				<div class="field">
					<span class="caption">Server</span>
					{#if app.profiles.length}
						<Select
							grow
							size="lg"
							label="Server"
							value={serverValue}
							options={serverOptions}
							disabled={app.locked}
							onchange={pickServer}
						/>
						{#if app.activeProfile && !app.auto.enabled}
							<small class="server-sub mono">
								<Sensitive id={privacyKey.server(app.activeProfile.id)}>{app.activeProfile.server}:{app.activeProfile.port}</Sensitive>
								<PrivacyToggle id={privacyKey.server(app.activeProfile.id)} label="server address" />
								· {protocols.label(app.activeProfile.protocol)}
							</small>
						{:else if app.auto.enabled}
							<small class="server-sub">Connects to the lowest-latency server</small>
						{/if}
					{:else}
						<button class="add-btn" onclick={() => onnavigate('profiles')}>Add a server</button>
					{/if}
				</div>
				<div class="field">
					<span class="caption">Mode</span>
					<Segmented options={modes} value={app.mode} label="Connection Mode" disabled={app.locked} onchange={(m) => app.setMode(m)} />
				</div>
			</div>
		</section>

		<!-- Traffic -->
		<section class="traffic card">
			<div class="stat-tiles">
				<div class="stat-box down">
					<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 3v9m0 0L4.5 8.5M8 12l3.5-3.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" /></svg>
					<StatTile label="Download" value={formatRate(app.traffic.down)} sub="{formatBytes(app.traffic.totalDown)} session" tone="down" />
				</div>
				<div class="stat-box up">
					<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 13V4m0 0L4.5 7.5M8 4l3.5 3.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" /></svg>
					<StatTile label="Upload" value={formatRate(app.traffic.up)} sub="{formatBytes(app.traffic.totalUp)} session" tone="up" />
				</div>
			</div>

			<div class="chart-container">
				<div class="chart-head">
					<span class="caption">Live activity</span>
					<span class="legend"><i class="l-down"></i>Down<i class="l-up"></i>Up</span>
				</div>
				<div class="chart-body">
					<Sparkline samples={app.history} height={96} />
					{#if !app.connected}<span class="chart-empty">Connect to see live traffic</span>{/if}
				</div>
			</div>

			<div class="today-block">
				<div class="today-header">
					<span class="caption">Today</span>
					{#if !usage.todayLoading}<strong class="tabular">{formatBytes((usage.today?.up ?? 0) + (usage.today?.down ?? 0))}</strong>{:else}<Skeleton width="56px" height="14px" />{/if}
				</div>
				{#if usage.quota?.limit}
					<div class="meter" class:warn={usage.quota.percent >= 80}>
						<div style:width="{Math.min(100, usage.quota.percent)}%"></div>
					</div>
					<small class="tabular quota-note">{usage.quota.percent.toFixed(0)}% of monthly cap</small>
				{/if}
			</div>
		</section>
	</div>

	<HealthCard />
	<HintBanner />
</div>

<style>
	.home {
		container-type: inline-size;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.grid {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		gap: 14px;
		align-items: stretch;
	}
	@container (max-width: 600px) {
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
	.banner-text.grow {
		flex: 1;
	}
	.bar-track {
		height: 4px;
		border-radius: 2px;
		background: var(--hairline);
		overflow: hidden;
		margin-top: 4px;
	}
	.bar-fill {
		height: 100%;
		background: var(--accent);
		transition: width 0.2s;
	}
	.bar-fill.indeterminate {
		animation: pulse 1.1s ease-in-out infinite;
	}
	@keyframes pulse {
		50% {
			opacity: 0.35;
		}
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
	/* Connection card */
	.hero {
		display: flex;
		flex-direction: column;
		padding: 18px 20px 20px;
		gap: 20px;
		transition: background 0.3s;
	}
	.hero.on {
		background: linear-gradient(180deg, color-mix(in srgb, var(--ok) 11%, var(--surface)), var(--surface) 70%);
	}
	.card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		min-height: 26px;
	}
	.status-pill {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		padding: 4px 11px 4px 9px;
		border-radius: 999px;
		background: var(--hairline);
		color: var(--fg-muted);
		font-size: 12px;
	}
	.status-pill.connected {
		background: color-mix(in srgb, var(--ok) 16%, transparent);
		color: var(--ok);
	}
	.status-pill.connecting {
		background: color-mix(in srgb, var(--warn) 16%, transparent);
		color: var(--warn);
	}
	.status-pill.down {
		background: color-mix(in srgb, var(--danger) 16%, transparent);
		color: var(--danger);
	}
	.status-pill .dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: currentColor;
	}
	.elapsed {
		font-size: 11.5px;
		opacity: 0.8;
		padding-left: 7px;
		border-left: 1px solid currentColor;
	}
	.headline {
		margin: 0;
		font-size: 20px;
		font-weight: 650;
		letter-spacing: -0.01em;
	}
	.headline.connected {
		color: var(--ok-text);
	}
	.headline.down {
		color: var(--danger);
	}
	.down-reason {
		color: var(--danger);
		max-width: 32ch;
		text-align: center;
	}
	.headline.connecting {
		color: var(--warn);
	}
	.chips {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: 6px;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 3px 9px;
		border-radius: 999px;
		background: var(--surface-strong);
		border: 1px solid var(--hairline);
		font-size: 11px;
		font-weight: 600;
		color: var(--fg-muted);
	}
	.ks-badge {
		padding: 3px 9px;
		border-radius: 999px;
		background: var(--surface-strong);
		border: 1px solid var(--hairline);
		font-size: 11px;
		font-weight: 600;
	}
	.power-center {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 14px;
		min-height: 170px;
	}
	.ip-row {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 24px;
		font-size: 12.5px;
	}
	.ip-chip {
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		padding: 3px 10px;
		border-radius: 999px;
		font-size: 12px;
	}
	.ip-chip:hover {
		border-color: var(--accent);
	}
	.ready-sub {
		color: var(--fg-muted);
	}
	.controls {
		display: grid;
		gap: 14px;
	}
	.field {
		display: grid;
		gap: 6px;
	}
	.server-sub {
		display: flex;
		align-items: center;
		gap: 4px;
		white-space: nowrap;
		font-size: 11px;
		color: var(--fg-muted);
		padding-left: 2px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.add-btn {
		border: 1px dashed var(--hairline);
		background: transparent;
		border-radius: 10px;
		padding: 10px;
		color: var(--accent);
		font-weight: 600;
	}
	.controls :global(.segmented button) {
		padding: 7px 10px;
	}
	/* Traffic card */
	.traffic {
		padding: 18px 20px 20px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.stat-tiles {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}
	.stat-box {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		min-width: 0;
		padding: 12px 14px;
		border: 1px solid var(--hairline);
		border-radius: 12px;
		background: var(--surface-strong);
	}
	.stat-box > svg {
		width: 22px;
		height: 22px;
		flex: none;
		padding: 4px;
		border-radius: 7px;
		margin-top: 1px;
	}
	.stat-box.down > svg {
		color: var(--down);
		background: color-mix(in srgb, var(--down) 14%, transparent);
	}
	.stat-box.up > svg {
		color: var(--up);
		background: color-mix(in srgb, var(--up) 14%, transparent);
	}
	.stat-box :global(.tile) {
		gap: 3px;
	}
	.stat-box :global(strong) {
		font-size: 19px;
	}
	.stat-box :global(small) {
		font-size: 11px;
	}
	.chart-container {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px 14px 10px;
		border: 1px solid var(--hairline);
		border-radius: 12px;
		background: var(--surface-strong);
	}
	.chart-head {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.legend {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.legend i {
		width: 7px;
		height: 7px;
		border-radius: 50%;
	}
	.legend .l-up {
		margin-left: 6px;
	}
	.l-down {
		background: var(--down);
	}
	.l-up {
		background: var(--up);
	}
	.chart-body {
		position: relative;
	}
	.chart-body :global(svg) {
		width: 100%;
		height: 120px !important;
	}
	.chart-empty {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	.today-block {
		display: grid;
		gap: 6px;
		padding: 0 2px;
	}
	.today-header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
	}
	.today-header strong {
		font-size: 14px;
	}
	.quota-note {
		color: var(--fg-muted);
		font-size: 11px;
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
</style>
