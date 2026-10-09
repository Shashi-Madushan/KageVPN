<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { formatBytes, formatDuration, formatRate, latencyLabel, latencyTone, profileFlag } from '$lib/format';
	import Icon from '$lib/components/Icon.svelte';
	import { usage } from '$lib/usage.svelte';
	import PowerButton from '$lib/components/PowerButton.svelte';
	import PrivacyToggle from '$lib/components/PrivacyToggle.svelte';
	import { privacyKey } from '$lib/privacy.svelte';
	import Sensitive from '$lib/components/Sensitive.svelte';
	import Select from '$lib/components/Select.svelte';
	import Segmented from '$lib/components/Segmented.svelte';
	import Sparkline from '$lib/components/Sparkline.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';

	const modes = [
		{ value: 'tun' as const, label: 'Full System' },
		{ value: 'proxy' as const, label: 'Proxy Only' }
	];
	const tone = $derived(app.tunnelDown ? 'down' : app.verifying ? 'checking' : app.protectedNow ? 'ok' : 'off');
	const via = $derived(app.connected ? 'tunnel' : 'server');
	const check = $derived(
		app.check && app.checkVia === via ? app.check : app.connected && app.delayMs !== null ? { ms: app.delayMs } : undefined
	);
	const pinging = $derived(!!check && 'pending' in check);

	let now = $state(Math.floor(Date.now() / 1000));

	onMount(() => {
		// The popover opens first; data fills in afterwards so the open itself never stalls.
		app.init().then(() => setTimeout(() => usage.loadToday(), 0));
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

	const serverValue = $derived(app.auto.enabled ? 'auto' : (app.activeId ?? ''));
	const serverOptions = $derived([
		...(app.profiles.length > 1 ? [{ value: 'auto', label: 'Auto (fastest server)', icon: '↯' }] : []),
		...app.profiles.map((p) => ({ value: p.id, label: p.name.replace(/^\p{RI}{2}\s*/u, ''), icon: profileFlag(p.name, p.server) }))
	]);

	const isConnecting = $derived(app.status.state === 'connecting');
	const headline = $derived(
		app.tunnelDown ? 'No internet' : app.verifying ? 'Checking…' : app.protectedNow ? "You're protected" : isConnecting ? 'Connecting…' : "You're not protected"
	);
	const sub = $derived(
		app.tunnelDown
			? (app.status.reason?.message ?? 'Traffic is not getting through the tunnel.')
			: isConnecting
				? 'Establishing secure tunnel…'
				: !app.profiles.length
					? 'Add a server to get started'
					: app.connected
						? null
						: 'Press the shield to connect'
	);
	const elapsed = $derived(app.status.since ? formatDuration(Math.max(0, now - app.status.since)) : null);
	const activeName = $derived(
		app.auto.enabled ? 'Auto (fastest)' : (app.activeProfile?.name.replace(/^\p{RI}{2}\s*/u, '') ?? null)
	);
</script>

<div class="popover">
	<button class="icon-btn close" onclick={() => api.hideTray()} aria-label="Close" title="Close (Esc)">✕</button>

	{#if !app.loaded}
		<div class="skeleton" aria-hidden="true">
			<div class="center"><Skeleton width="112px" height="112px" radius="50%" /></div>
			<div class="center"><Skeleton width="170px" height="22px" radius="8px" /></div>
			<Skeleton height="48px" radius="12px" />
			<Skeleton height="56px" radius="12px" />
			<Skeleton height="128px" radius="12px" />
			<Skeleton height="34px" radius="10px" />
		</div>
	{:else}
		<section class="hero">
			<button
				class="shield {tone}"
				disabled={app.busy || !app.activeProfile}
				onclick={() => app.toggle()}
				aria-label={app.connected ? 'Disconnect from VPN' : 'Connect to VPN'}
				aria-busy={isConnecting}
			>
				<svg viewBox="0 0 64 72" aria-hidden="true">
					<path class="outline" d="M32 4 56 13v22c0 15-10 27-24 33C18 62 8 50 8 35V13L32 4Z" />
					<g class="power"><path d="M32 24v14" /><path d="M23.5 30.5a13 13 0 1 0 17 0" /></g>
				</svg>
				{#if isConnecting}<i class="ring"></i>{/if}
			</button>
			<h1 class="headline {tone}">{headline}</h1>
			{#if sub}<p class="sub">{sub}</p>{/if}
		</section>

		{#if app.profiles.length}
			{#if app.locked}
				<div class="strip" aria-label="Connection details">
					<span class="cell grow" title={activeName ?? ''}>
						<span class="flag">{app.activeProfile ? profileFlag(app.activeProfile.name, app.activeProfile.server) : '↯'}</span>
						<span class="two"><small>Server</small><strong>{activeName ?? '—'}</strong></span>
					</span>
					<span class="cell"><strong class="tabular">{elapsed ?? '—'}</strong></span>
					<span class="cell ip">
						{#if app.exitIp}
							<span class="ip-text"><Sensitive id={privacyKey.exitIp}><span class="mono tabular">{app.exitIp}</span></Sensitive></span>
							<PrivacyToggle id={privacyKey.exitIp} label="IP address" />
						{:else if app.tunnelDown}
							<small>No IP</small>
						{:else}
							<Skeleton width="84px" height="14px" />
						{/if}
					</span>
				</div>
			{:else}
				<div class="pick">
					<Select
						grow
						value={serverValue}
						options={serverOptions}
						label="Server"
						placeholder="No servers"
						onchange={(id) => {
							if (id === 'auto') return app.setAuto({ ...app.auto, enabled: true });
							if (app.auto.enabled) app.setAuto({ ...app.auto, enabled: false });
							app.select(id);
						}}
					/>
				</div>
			{/if}

			<button class="card latency {latencyTone(check)}" onclick={() => app.checkServer()} disabled={pinging || !app.activeProfile} title={app.connected ? 'Check that traffic flows through the tunnel' : 'Check that the server answers'}>
				<span class="label">Latency</span>
				<span class="value tabular"><Icon name="bolt" size={16} />{check ? (pinging ? 'Pinging…' : latencyLabel(check)) : 'Tap to test'}</span>
			</button>

			<section class="card usage" aria-label="Data usage">
				<span class="label">Data usage</span>
				<div class="rates">
					<span class="down tabular">↓ {formatRate(app.traffic.down)}</span>
					<span class="up tabular">↑ {formatRate(app.traffic.up)}</span>
					<span class="today tabular">
						Today: {#if usage.todayLoading}<Skeleton width="36px" height="11px" />{:else}{usage.today ? formatBytes(usage.today.up + usage.today.down) : '—'}{/if}
					</span>
				</div>
				<div class="graph">
					<Sparkline samples={app.history} height={46} />
					{#if !app.connected}<span class="graph-empty">Connect to see live traffic</span>{/if}
				</div>
			</section>

			<Segmented options={modes} value={app.mode} label="Mode" disabled={app.locked} onchange={(m) => app.setMode(m)} />
			{#if app.locked}<p class="lock">Disconnect to change server or mode</p>{/if}
		{:else}
			<button class="btn primary add" onclick={() => api.openMainWindow()}>Add a server…</button>
		{/if}
	{/if}

	<footer>
		<button class="foot-btn" onclick={() => api.openMainWindow()}>Open {app.info.name}</button>
		<button class="foot-btn quit" onclick={() => app.quit()}>Quit</button>
	</footer>
</div>

<style>
	:global(body) {
		background: transparent;
	}
	.popover {
		position: relative;
		height: 100vh;
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 14px 12px 6px;
		background: color-mix(in srgb, var(--surface-strong) 92%, transparent);
		backdrop-filter: blur(24px);
		-webkit-backdrop-filter: blur(24px);
		border: 1px solid var(--hairline);
		border-radius: 12px;
		overflow: hidden;
		box-sizing: border-box;
	}
	.icon-btn {
		width: 28px;
		height: 28px;
		display: grid;
		place-items: center;
		border: 0;
		border-radius: 7px;
		background: none;
		color: var(--fg-muted);
		font-size: 12px;
	}
	.icon-btn.close {
		position: absolute;
		top: 8px;
		right: 8px;
		z-index: 1;
	}
	.icon-btn:hover {
		background: var(--hairline);
		color: var(--fg);
	}
	.center {
		display: grid;
		justify-items: center;
	}
	.skeleton {
		display: grid;
		gap: 10px;
	}
	.hero {
		display: grid;
		justify-items: center;
		gap: 4px;
		padding-top: 6px;
	}
	.shield {
		--c: var(--fg-muted);
		position: relative;
		width: 96px;
		height: 96px;
		display: grid;
		place-items: center;
		padding: 0;
		border: 1px solid var(--hairline);
		border-radius: 50%;
		background: radial-gradient(circle at 50% 40%, var(--surface-strong), color-mix(in srgb, var(--c) 8%, var(--surface-strong)));
		color: var(--c);
		transition: box-shadow 0.25s, color 0.25s, transform 0.1s;
	}
	.shield:active:not(:disabled) {
		transform: scale(0.97);
	}
	.shield.ok {
		--c: var(--ok);
		box-shadow: 0 0 0 4px color-mix(in srgb, var(--ok) 22%, transparent), 0 0 34px color-mix(in srgb, var(--ok) 38%, transparent);
	}
	.shield.checking {
		--c: var(--warn);
		box-shadow: 0 0 0 4px color-mix(in srgb, var(--warn) 25%, transparent);
	}
	.shield.down {
		--c: var(--danger);
		box-shadow: 0 0 0 4px color-mix(in srgb, var(--danger) 25%, transparent), 0 0 28px color-mix(in srgb, var(--danger) 30%, transparent);
	}
	.shield svg {
		width: 58%;
		height: 58%;
		overflow: visible;
	}
	.outline {
		fill: color-mix(in srgb, var(--c) 10%, transparent);
		stroke: currentColor;
		stroke-width: 3;
		stroke-linejoin: round;
	}
	.power path {
		fill: none;
		stroke: currentColor;
		stroke-width: 4;
		stroke-linecap: round;
	}
	.ring {
		position: absolute;
		inset: -3px;
		border-radius: 50%;
		border: 3px solid transparent;
		border-top-color: var(--warn);
		animation: spin 1.1s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.ring {
			animation: none;
		}
	}
	.headline {
		margin: 8px 0 0;
		font-size: 17px;
		font-weight: 800;
		letter-spacing: 0.03em;
		text-transform: uppercase;
	}
	.headline.ok {
		color: var(--ok-text);
	}
	.headline.checking {
		color: var(--warn);
	}
	.headline.down {
		color: var(--danger);
	}
	.sub {
		margin: 0;
		max-width: 100%;
		text-align: center;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.strip {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto minmax(0, 1.1fr);
		align-items: stretch;
		overflow: hidden;
		border: 1px solid var(--hairline);
		border-radius: 12px;
		background: var(--surface);
		font-size: 12px;
	}
	.cell {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 7px 8px;
		min-width: 0;
	}
	.cell:nth-child(2) strong {
		font-size: 11.5px;
	}
	.cell + .cell {
		border-left: 1px solid var(--hairline);
	}
	.cell.grow {
		overflow: hidden;
	}
	.two {
		display: grid;
		min-width: 0;
		line-height: 1.25;
	}
	.two small {
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--fg-muted);
	}
	.two strong {
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
		overflow-wrap: break-word;
		font-size: 12px;
	}
	.flag {
		font-size: 20px;
	}
	.ip {
		font-size: 10px;
		gap: 0;
		justify-content: space-between;
		color: var(--fg-muted);
		user-select: text;
	}
	.ip-text {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.ip :global(button) {
		flex: none;
	}
	.pick {
		display: flex;
	}
	.card {
		border: 1px solid var(--hairline);
		border-radius: 12px;
		background: var(--surface);
		padding: 8px 12px;
	}
	.label {
		font-size: 10.5px;
		font-weight: 700;
		letter-spacing: 0.07em;
		text-transform: uppercase;
	}
	.latency {
		display: grid;
		justify-items: center;
		gap: 2px;
		width: 100%;
		color: var(--fg);
	}
	.latency .value {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 17px;
		font-weight: 700;
	}
	.latency.ok {
		border-color: color-mix(in srgb, var(--ok) 45%, transparent);
		background: color-mix(in srgb, var(--ok) 10%, var(--surface));
		color: var(--ok-text);
	}
	.latency.warn {
		border-color: color-mix(in srgb, var(--warn) 55%, transparent);
		background: color-mix(in srgb, var(--warn) 13%, var(--surface));
		color: var(--warn);
	}
	.latency.bad {
		border-color: color-mix(in srgb, var(--danger) 50%, transparent);
		background: color-mix(in srgb, var(--danger) 10%, var(--surface));
		color: var(--danger);
	}
	.usage {
		display: grid;
		gap: 6px;
		justify-items: center;
	}
	.rates {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		width: 100%;
		font-size: 14px;
		font-weight: 700;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	.today {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 9px;
		border-radius: 999px;
		background: var(--accent);
		color: #fff;
		font-size: 11px;
		font-weight: 600;
	}
	.graph {
		position: relative;
		width: 100%;
	}
	.graph-empty {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.lock {
		margin: -4px 0 0;
		font-size: 10.5px;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		color: var(--fg-muted);
		text-align: center;
	}
	.add {
		min-height: 34px;
	}
	footer {
		margin-top: auto;
		display: flex;
		justify-content: space-between;
		border-top: 1px solid var(--hairline);
		padding-top: 4px;
	}
	.foot-btn {
		min-height: 26px;
		border: 0;
		background: none;
		padding: 3px 8px;
		border-radius: 6px;
		color: var(--accent);
		font-size: 12px;
	}
	.foot-btn:hover {
		background: var(--hairline);
	}
	.foot-btn.quit {
		color: var(--fg-muted);
	}
	.foot-btn.quit:hover {
		color: var(--danger);
	}
</style>
