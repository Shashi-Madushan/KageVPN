<script lang="ts">
	import { api } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { privacy, privacyKey } from '$lib/privacy.svelte';
	import { protocols } from '$lib/protocols.svelte';
	import PingButton from './PingButton.svelte';
	import PrivacyToggle from './PrivacyToggle.svelte';
	import Sensitive from './Sensitive.svelte';
	import { appUpdate } from '$lib/appUpdate.svelte';
	import { onMount } from 'svelte';
	import { caps } from '$lib/caps.svelte';
	import { periodWord } from '$lib/capView';
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
	import type { Page } from './Sidebar.svelte';


	onMount(() => caps.use());
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

	let repairing = $state(false);
	async function repairSystemProxy() {
		repairing = true;
		await api.repairSystemProxy().then(
			() => app.notify('Your proxy settings are back as they were.', 'info'),
			(e) => app.notify(String(e))
		);
		repairing = false;
	}

	const isConnecting = $derived(app.status.state === 'connecting');
	const statusText = $derived(
		app.disconnecting ? 'Disconnecting…' : app.status.blocked && app.reliability.killSwitch ? 'Blocked' : app.tunnelDown ? 'No Internet' : app.verifying ? 'Verifying…' : app.protectedNow ? 'Protected' : isConnecting ? 'Connecting…' : 'Not Connected'
	);
	const headline = $derived(
		app.disconnecting
			? 'Disconnecting…'
			: app.status.blocked && app.reliability.killSwitch
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
	{#each caps.pending as c (c.path)}
		<div class="banner card danger" role="status">
			<div class="banner-text grow">
				<strong>{c.name} reached its {formatBytes(c.limitBytes)} {periodWord(c.period)} data cap</strong>
				<small>Blocking stops it using the network until the {c.period} ends. The VPN reconnects once.</small>
			</div>
			<button class="primary" onclick={() => caps.block(c.path)}>Block now</button>
			<button onclick={() => caps.notNow(c.path)}>Not now</button>
		</div>
	{/each}
	{#if app.status.systemProxy?.state === 'failed'}
		<div class="banner card danger" role="status">
			<div class="banner-text grow">
				<strong>{app.status.systemProxy.action === 'set' ? 'Could not set the system proxy' : 'Your proxy settings may still point at Kage'}</strong>
				<small>{app.status.systemProxy.reason}</small>
			</div>
			<button class="primary" disabled={repairing} onclick={repairSystemProxy}>{repairing ? 'Repairing…' : 'Repair'}</button>
		</div>
	{/if}
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

	<div class="chips-row">
		{#if app.connected && app.status.mode === 'proxy' && app.status.proxyPort}
			<span class="chip mono" title={app.reliability.proxyAuth ? 'Point apps at this address (SOCKS5 or HTTP). Login: user kage, password in Settings' : 'Point apps at this address (SOCKS5 or HTTP)'}>127.0.0.1:{app.status.proxyPort}</span>
		{/if}
		{#if app.connected && app.status.systemProxy?.state === 'set'}
			<span class="chip" title="The macOS proxy settings point at the VPN; they come back when you disconnect">System proxy on</span>
		{/if}
		{#if app.reliability.killSwitch}
			<span class="chip" title="Kill switch is armed"><Icon name="shield" size={12} /> Kill switch</span>
		{/if}
		{#if app.reliability.autoReconnect}
			<span class="chip" title="Reconnects automatically if the tunnel drops"><Icon name="refresh" size={12} /> Auto-reconnect</span>
		{/if}
	</div>

	<div class="top">
		<!-- Connection -->
		<section class="hero card {app.disconnecting ? 'connecting' : app.tunnelDown ? 'down' : app.verifying ? 'connecting' : app.status.state}" class:on={app.connected}>
			<MascotButton size={88} />
			<div class="hero-text">
				<div class="status-line {app.disconnecting ? 'connecting' : app.tunnelDown ? 'down' : app.verifying ? 'connecting' : app.status.state}">
					<strong>{statusText}</strong>{#if elapsed}<span class="tabular"> · {elapsed}</span>{/if}
				</div>
				<h2 class="headline {app.disconnecting ? 'connecting' : app.tunnelDown ? 'down' : app.verifying ? 'connecting' : app.status.state}">{headline}</h2>
				<div class="ip-row">
					{#if app.connected}
						{#if app.exitIp}
							<button class="ip-chip mono" onclick={copyExitIp} title="Copy IP address"><Sensitive id={privacyKey.exitIp}>{app.exitIp}</Sensitive></button>
							<PrivacyToggle id={privacyKey.exitIp} label="IP address" />
						{/if}
						<PingButton />
					{:else}
						<span class="ready-sub">{app.profiles.length ? (isConnecting ? 'Establishing secure tunnel…' : 'Traffic leaves your Mac directly.') : 'Add a server to connect'}</span>
						{#if app.profiles.length && !isConnecting}<PingButton />{/if}
					{/if}
				</div>
				{#if app.profiles.length}
					<button class="cta" class:stop={app.connected || app.busy} disabled={app.disconnecting} onclick={() => app.toggle()}>
						{app.disconnecting ? 'Disconnecting…' : app.busy ? 'Cancel' : app.connected ? 'Disconnect' : 'Connect'}
					</button>
				{:else}
					<button class="linkbtn" onclick={() => onboarding.show()}>New here? Run the quick setup</button>
				{/if}
			</div>
		</section>

		<section class="card controls">
			<div class="field">
				<span class="caption">Server</span>
				{#if app.profiles.length}
					<Select grow size="lg" label="Server" value={serverValue} options={serverOptions} disabled={app.locked} onchange={pickServer} />
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
				<small class="server-sub">{app.mode === 'tun' ? 'Every app on your Mac goes through the VPN.' : 'Only apps pointed at the local proxy use the VPN.'}</small>
			</div>
		</section>
	</div>

	<div class="bottom">
		<section class="traffic card">
			<div class="chart-head">
				<span class="caption">Live activity</span>
				<span class="legend"><i class="l-down"></i>Down <b class="tabular down">{formatRate(app.traffic.down)}</b><i class="l-up"></i>Up <b class="tabular up">{formatRate(app.traffic.up)}</b></span>
			</div>
			<div class="chart-body">
				<Sparkline samples={app.history} height={96} />
				{#if !app.connected}<span class="chart-empty">No live traffic. Connect to see speeds.</span>{/if}
			</div>
			<div class="traffic-foot">
				<small class="tabular sess">{formatBytes(app.traffic.totalDown)} down · {formatBytes(app.traffic.totalUp)} up this session</small>
				<span class="today-pill tabular" title="Total used today">Today: {#if usage.todayLoading}…{:else}{formatBytes((usage.today?.up ?? 0) + (usage.today?.down ?? 0))}{/if}</span>
			</div>
			{#if usage.quota?.limit}
				<div class="quota">
					<div class="meter" class:warn={usage.quota.percent >= 80}>
						<div style:width="{Math.min(100, usage.quota.percent)}%"></div>
					</div>
					<small class="tabular quota-note">{usage.quota.percent.toFixed(0)}% of monthly cap</small>
				</div>
			{/if}
		</section>

		<div class="side">
			<HealthCard />
		</div>
	</div>
	<HintBanner />
</div>

<style>
	.home {
		container-type: inline-size;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.top {
		display: grid;
		grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr);
		gap: 10px;
	}
	.bottom {
		display: grid;
		grid-template-columns: minmax(0, 2.2fr) minmax(0, 1fr);
		gap: 10px;
	}
	.side {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	@container (max-width: 700px) {
		.top,
		.bottom {
			grid-template-columns: 1fr;
		}
	}
	.chips-row {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.chips-row:empty {
		display: none;
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
		align-items: center;
		gap: 14px;
		padding: 14px 18px;
		transition: background-color 0.3s;
	}
	.hero.on {
		background: color-mix(in srgb, var(--ok) 8%, var(--surface));
	}
	.hero-text {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 5px;
		min-width: 0;
	}
	.status-line {
		font-size: 11px;
		color: var(--fg-muted);
	}
	.status-line span {
		margin-left: 2px;
	}
	.status-line.connected {
		color: var(--ok-text);
	}
	.status-line.connecting {
		color: var(--warn);
	}
	.status-line.down {
		color: var(--danger);
	}
	.headline {
		margin: 0;
		font-size: 20px;
		font-weight: 600;
		line-height: 1.15;
		letter-spacing: -0.01em;
	}
	.headline.connected {
		color: var(--ok-text);
	}
	.headline.down {
		color: var(--danger);
	}
	.headline.connecting {
		color: var(--warn);
	}
	.cta {
		margin-top: 2px;
		min-height: 34px;
		padding: 0 20px;
		border: 0;
		border-radius: 9px;
		background: var(--accent);
		color: #fff;
		font-size: 13px;
		font-weight: 600;
	}
	.cta.stop {
		background: var(--danger);
	}
	.cta:disabled {
		opacity: 0.6;
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
	.ip-row {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 6px;
		min-height: 22px;
		font-size: 12px;
	}
	.ip-chip {
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		padding: 2px 9px;
		border-radius: 999px;
		font-size: 11.5px;	white-space: nowrap;
	}
	.ip-chip:hover {
		border-color: var(--accent);
	}
	.ready-sub {
		color: var(--fg-muted);
	}
	.controls {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px 16px;
	}
	.field {
		display: grid;
		gap: 4px;
	}
	.server-sub {
		display: flex;
		align-items: center;
		gap: 4px;
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
		padding: 5px 10px;
	}
	/* Traffic */
	.traffic {
		padding: 12px 16px 10px;
		display: flex;
		flex-direction: column;
		gap: 6px;
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
		font-size: 12px;
		color: var(--fg-muted);
	}
	.legend i {
		width: 7px;
		height: 7px;
		border-radius: 50%;
	}
	.legend .l-up {
		margin-left: 10px;
	}
	.legend b.down {
		color: var(--down);
	}
	.legend b.up {
		color: var(--up);
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
		height: 84px !important;
	}
	.chart-empty {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.sess {
		color: var(--fg-muted);
		font-size: 11px;
	}
	.traffic-foot {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
	}
	.today-pill {
		flex: none;
		padding: 3px 12px;
		border-radius: 999px;
		background: var(--surface);
		border: 1px solid var(--hairline);
		color: var(--fg);
		font-size: 12px;
		font-weight: 500;
	}
	.quota {
		display: grid;
		gap: 4px;
	}
	.side :global(.health) {
		flex: 1;
		align-content: start;
	}
	.quota-note {
		color: var(--fg-muted);
		font-size: 11px;
	}
	.meter {
		height: 6px;
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
