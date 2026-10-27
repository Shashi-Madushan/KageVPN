<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
	import { api } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { formatBytes, formatDuration, formatRate, latencyLabel, latencyTone, planSummary, profileFlag } from '$lib/format';
	import Icon from '$lib/components/Icon.svelte';
	import MascotButton from '$lib/components/MascotButton.svelte';
	import { usage } from '$lib/usage.svelte';
	import PrivacyToggle from '$lib/components/PrivacyToggle.svelte';
	import { privacyKey } from '$lib/privacy.svelte';
	import Sensitive from '$lib/components/Sensitive.svelte';
	import Select from '$lib/components/Select.svelte';
	import Segmented from '$lib/components/Segmented.svelte';
	import Sparkline from '$lib/components/Sparkline.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';

	const WIDTH = 272;
	const modes = [
		{ value: 'tun' as const, label: 'Full System' },
		{ value: 'proxy' as const, label: 'Proxy Only' }
	];
	const tone = $derived(app.disconnecting ? 'checking' : app.tunnelDown ? 'down' : app.verifying ? 'checking' : app.protectedNow ? 'ok' : 'off');
	const via = $derived(app.connected ? 'tunnel' : 'server');
	const check = $derived(
		app.check && app.checkVia === via ? app.check : app.connected && app.delayMs !== null ? { ms: app.delayMs } : undefined
	);
	const pinging = $derived(!!check && 'pending' in check);

	let now = $state(Math.floor(Date.now() / 1000));
	let popover: HTMLDivElement;

	/** The window follows the content, so the popover never shows an empty band or clips. */
	function fitWindow() {
		const height = Math.ceil(popover.getBoundingClientRect().height);
		if (height && Math.abs(height - window.innerHeight) > 1) {
			getCurrentWindow().setSize(new LogicalSize(WIDTH, height)).catch(() => {});
		}
	}

	onMount(() => {
		// The popover opens first; data fills in afterwards so the open itself never stalls.
		app.init().then(() => setTimeout(() => usage.loadToday(), 0));
		const timer = setInterval(() => (now = Math.floor(Date.now() / 1000)), 1000);
		
		const handleKey = (e: KeyboardEvent) => {
			if (e.key === 'Escape') api.hideTray();
		};
		window.addEventListener('keydown', handleKey);
		const observer = new ResizeObserver(fitWindow);
		observer.observe(popover);

		return () => {
			clearInterval(timer);
			window.removeEventListener('keydown', handleKey);
			observer.disconnect();
		};
	});

	const serverValue = $derived(app.auto.enabled ? 'auto' : (app.activeId ?? ''));
	/** " · 42 ms" once a server has been timed, so the list shows which one is quick. */
	function latencyNote(id: string): string {
		const found = app.latency[id];
		return found && 'ms' in found ? ` · ${found.ms} ms` : '';
	}
	const serverOptions = $derived([
		...(app.profiles.length > 1 ? [{ value: 'auto', label: 'Auto (fastest server)', icon: '↯' }] : []),
		...app.profiles.map((p) => ({ value: p.id, label: p.name.replace(/^\p{RI}{2}\s*/u, '') + latencyNote(p.id), icon: profileFlag(p.name, p.server) }))
	]);

	const isConnecting = $derived(app.status.state === 'connecting');
	const headline = $derived(
		app.disconnecting ? 'Disconnecting…' : app.tunnelDown ? 'No internet' : app.verifying ? 'Checking…' : app.protectedNow ? "You're protected" : isConnecting ? 'Connecting…' : "You're not protected"
	);
	const sub = $derived(
		app.disconnecting
			? 'Closing the tunnel…'
			: app.tunnelDown
			? (app.status.reason?.message ?? 'Traffic is not getting through the tunnel.')
			: isConnecting
				? 'Establishing secure tunnel…'
				: !app.profiles.length
					? 'Add a server to get started'
					: app.connected
						? null
						: null
	);
	const elapsed = $derived(app.status.since ? formatDuration(Math.max(0, now - app.status.since)) : null);
	/** What the active server's provider reports: data left and days left, when it reports either. */
	const plan = $derived.by(() => {
		const subId = app.auto.enabled ? null : app.activeProfile?.subscriptionId;
		const info = app.subscriptions.find((s) => s.id === subId)?.userInfo;
		return info ? planSummary(info, now) : null;
	});
	const statusText = $derived(
		app.disconnecting ? 'Disconnecting…' : app.tunnelDown ? 'No internet' : app.verifying ? 'Checking…' : app.protectedNow ? 'Protected' : isConnecting ? 'Connecting…' : 'Not connected'
	);
	const activeName = $derived(
		app.auto.enabled ? 'Auto (fastest)' : (app.activeProfile?.name.replace(/^\p{RI}{2}\s*/u, '') ?? null)
	);
</script>

<div class="popover" bind:this={popover}>
	<button class="icon-btn close" onclick={() => api.hideTray()} aria-label="Close" title="Close (Esc)"><svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7" /></svg></button>

	{#if !app.loaded}
		<div class="skeleton" aria-hidden="true">
			<div class="center"><Skeleton width="88px" height="88px" radius="50%" /></div>
			<div class="center"><Skeleton width="170px" height="22px" radius="8px" /></div>
			<div class="center"><Skeleton width="80px" height="26px" radius="999px" /></div>
			<Skeleton height="38px" radius="10px" />
			<Skeleton height="38px" radius="12px" />
			<Skeleton height="132px" radius="12px" />
			<Skeleton height="34px" radius="10px" />
		</div>
	{:else}
		<section class="hero {tone}">
			<MascotButton size={76} />
			<h1 class="headline {tone}">{headline}</h1>
			{#if sub}<p class="sub">{sub}{#if isConnecting} · tap to cancel{/if}</p>{/if}
			{#if app.activeProfile || app.auto.enabled}
				<button class="ping {latencyTone(check)}" onclick={() => app.checkServer()} disabled={pinging || !app.activeProfile} title={app.connected ? 'Check that traffic flows through the tunnel' : 'Check that the server answers'}>
					<Icon name="bolt" size={13} /><span class="tabular">{check ? (pinging ? 'Pinging…' : latencyLabel(check)) : 'Test latency'}</span>
				</button>
			{:else if !sub && app.profiles.length}
				<p class="sub">Tap the mascot to connect</p>
			{/if}
		</section>

		{#if app.profiles.length}
			{#snippet serverPicker()}
				<Select grow value={serverValue} options={serverOptions} label="Server" placeholder="No servers" onchange={(choice) => app.chooseServer(choice)} />
			{/snippet}
			{#if app.connected || !app.locked}
				<!-- While connected the server changes in place; only the mode needs a disconnect. -->
				<div class="pick" title={app.connected ? 'Switch server without disconnecting' : undefined}>{@render serverPicker()}</div>
			{/if}

			<section class="card info" aria-label="Connection">
				<span class="elapsed tabular" title="Time connected">{elapsed && app.connected ? elapsed : '--:--:--'}</span>
				<span class="ip" title="Exit IP">
					{#if !app.connected}
						<span class="none">Not connected</span>
					{:else if app.exitIp}
						<span class="ip-text"><Sensitive id={privacyKey.exitIp}><span class="mono tabular">{app.exitIp}</span></Sensitive></span>
						<PrivacyToggle id={privacyKey.exitIp} label="IP address" />
					{:else if app.tunnelDown}
						<span class="none">No IP</span>
					{:else}
						<Skeleton width="84px" height="14px" />
					{/if}
				</span>
			</section>

			<section class="card usage" aria-label="Data usage">
				<div class="head">
					<h2 class="title">Data usage</h2>
					<span class="today tabular" title="Total used today">Today: {#if usage.todayLoading}…{:else}{usage.today ? formatBytes(usage.today.up + usage.today.down) : '—'}{/if}</span>
				</div>
				<div class="speeds">
					<strong class="rate down tabular" title="Download">↓ {formatRate(app.traffic.down)}</strong>
					<strong class="rate up tabular" title="Upload">↑ {formatRate(app.traffic.up)}</strong>
				</div>
				{#if plan}<p class="plan" class:low={plan.low} title="What your provider reports for this server's subscription">{plan.text}</p>{/if}
				<div class="graph">
					<Sparkline samples={app.history} height={52} fill />
				</div>
			</section>

			<div class="mode" title={app.connected ? 'Disconnect to change mode' : undefined}>
				<Segmented options={modes} value={app.mode} label="Mode" disabled={app.locked} onchange={(m) => app.setMode(m)} />
			</div>
			{#if app.busy}<p class="lock">Disconnect to change server or mode</p>{/if}
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
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px 10px 4px;
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
		top: 4px;
		right: 6px;
		z-index: 1;
		width: 22px;
		height: 22px;
		border-radius: 50%;
	}
	.icon-btn:hover {
		background: var(--hairline);
		color: var(--fg);
	}
	.icon-btn:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 1px;
	}
	.popover > :global(*) {
		flex-shrink: 0;
	}
	.center {
		display: grid;
		justify-items: center;
	}
	.skeleton {
		display: grid;
		gap: 10px;
	}
	/* The mascot leads, its state is the headline under it, and the latency pill closes the header. */
	.hero {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		text-align: center;
	}
	.headline {
		margin: 0;
		font-size: 14px;
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
		margin: -2px 0 0;
		max-width: 100%;
		font-size: 12px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		color: var(--fg-muted);
	}
	.ping {
		--c: var(--fg-muted);
		display: inline-flex;
		align-items: center;
		gap: 5px;
		min-height: 24px;
		padding: 0 12px;
		border: 1px solid color-mix(in srgb, var(--c) 40%, transparent);
		border-radius: 999px;
		background: color-mix(in srgb, var(--c) 12%, transparent);
		color: var(--c);
		font-size: 11.5px;
		font-weight: 700;
	}
	.ping.ok {
		--c: var(--ok-text);
	}
	.ping.warn {
		--c: var(--warn);
	}
	.ping.bad {
		--c: var(--danger);
	}
	.ping:hover:not(:disabled) {
		background: color-mix(in srgb, var(--c) 20%, transparent);
	}
	.pick {
		display: flex;
	}
	.card {
		border: 1px solid var(--hairline);
		border-radius: 12px;
		background: var(--surface);
	}
	/* Time on the left, exit IP on the right, split by a hairline. */
	.info {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr);
		align-items: center;
		min-height: 34px;
	}
	.elapsed {
		padding: 0 10px;
		border-right: 1px solid var(--hairline);
		line-height: 34px;
		font-size: 12px;
		font-weight: 600;
		color: var(--fg);
	}
	.ip {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 6px;
		min-width: 0;
		padding: 0 6px 0 10px;
		font-size: 12px;
		color: var(--fg);
		user-select: text;
	}
	.ip .none {
		color: var(--fg-muted);
	}
	.ip-text {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.usage {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 8px 10px;
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 6px;
	}
	.title {
		margin: 0;
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--fg);
	}
	.speeds {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 6px;
		white-space: nowrap;
	}
	.rate {
		font-size: 13px;
		font-weight: 700;
	}
	.today {
		flex: none;
		padding: 2px 8px;
		border-radius: 999px;
		background: var(--accent);
		color: #fff;
		font-size: 11px;
		font-weight: 700;
	}
	.plan {
		margin: 0;
		text-align: center;
		font-size: 11px;
		color: var(--fg-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.plan.low {
		color: var(--warn);
		font-weight: 600;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	.graph {
		position: relative;
		width: 100%;
		height: 52px;
	}
	/* Sized by the row, not by the chart's own aspect ratio. */
	.graph :global(svg) {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
	}
	.mode {
		font-size: 12px;
	}
	.lock {
		margin: -2px 0 0;
		font-size: 11px;
		color: var(--fg-muted);
		text-align: center;
	}
	.add {
		min-height: 34px;
	}
	footer {
		display: flex;
		justify-content: space-between;
		border-top: 1px solid var(--hairline);
		padding-top: 4px;
	}
	.foot-btn {
		min-height: 24px;
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
