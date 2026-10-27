<script lang="ts">
	import { onMount } from 'svelte';
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

<div class="popover">
	<button class="icon-btn close" onclick={() => api.hideTray()} aria-label="Close" title="Close (Esc)"><svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7" /></svg></button>

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
		<section class="hero {tone}">
			<div class="hero-head">
				<MascotButton size={52} />
				<div class="hero-text">
					<h1 class="headline {tone}">{headline}</h1>
					{#if sub}<p class="sub">{sub}</p>{:else if elapsed && app.connected}<p class="sub tabular">Connected {elapsed}</p>{/if}
				</div>
			</div>
			{#if app.profiles.length}
				<button class="cta" class:stop={app.connected || app.busy} disabled={app.disconnecting} onclick={() => app.toggle()}>
					{app.disconnecting ? 'Disconnecting…' : app.busy ? 'Cancel' : app.connected ? 'Disconnect' : 'Connect'}
				</button>
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
			<section class="card sheet" aria-label="Connection and data usage">
				<div class="info">
					<div class="cell ip">
						<span class="k">Exit IP</span>
						<span class="v">
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
					</div>
					<button class="cell latency {latencyTone(check)}" onclick={() => app.checkServer()} disabled={pinging || !app.activeProfile} title={app.connected ? 'Check that traffic flows through the tunnel' : 'Check that the server answers'}>
						<span class="k">Latency</span>
						<span class="v tabular"><Icon name="bolt" size={12} />{check ? (pinging ? 'Pinging…' : latencyLabel(check)) : 'Test'}</span>
					</button>
				</div>
				<div class="traffic">
					<div class="speeds">
						<div class="speed"><span class="k">↓ Download</span><strong class="v down tabular">{formatRate(app.traffic.down)}</strong></div>
						<div class="speed"><span class="k">↑ Upload</span><strong class="v up tabular">{formatRate(app.traffic.up)}</strong></div>
					</div>
					<div class="graph">
						<Sparkline samples={app.history} height={72} fill />
					</div>
					<div class="foot">
						<span class="today tabular" title="Total used today">Today: {#if usage.todayLoading}…{:else}{usage.today ? formatBytes(usage.today.up + usage.today.down) : '—'}{/if}</span>
						{#if plan}<span class="plan" class:low={plan.low} title="What your provider reports for this server's subscription">{plan.text}</span>{/if}
					</div>
				</div>
			</section>

			<div title={app.connected ? 'Disconnect to change mode' : undefined}>
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
		height: 100vh;
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 30px 8px 6px;
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
	/* A plain header, not another card: the boxes below carry the structure. */
	.hero {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 0 2px;
	}
	.hero-head {
		display: flex;
		align-items: center;
		gap: 10px;
		/* Clear of the close button. */
		padding-right: 24px;
	}
	.hero-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.cta {
		min-height: 34px;
		padding: 0 14px;
		border: 0;
		border-radius: 8px;
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
	.headline {
		margin: 0;
		font-size: 15px;
		font-weight: 700;
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
		font-size: 12px;
		white-space: nowrap;
		color: var(--fg-muted);
	}
	.ip-text {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
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
	.info {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		border-bottom: 1px solid var(--hairline);
	}
	.cell {
		display: grid;
		gap: 2px;
		min-width: 0;
		padding: 6px 10px;
		text-align: left;
	}
	.cell .k,
	.speed .k {
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.04em;
		color: var(--fg-muted);
	}
	.cell .v {
		display: flex;
		align-items: center;
		gap: 4px;
		min-width: 0;
		min-height: 20px;
		font-size: 13px;
		font-weight: 600;
		color: var(--fg);
		user-select: text;
	}
	.cell .none {
		font-weight: 500;
		color: var(--fg-muted);
	}
	.latency {
		border: 0;
		border-left: 1px solid var(--hairline);
		border-radius: 0;
		background: none;
		min-width: 88px;
	}
	.latency:hover:not(:disabled) {
		background: var(--hairline);
	}
	.latency.ok .v {
		color: var(--ok-text);
	}
	.latency.warn .v {
		color: var(--warn);
	}
	.latency.bad .v {
		color: var(--danger);
	}
	/* One grouped card. The chart takes up or gives back slack, so the footer is never pushed out. */
	.popover > .sheet {
		flex-shrink: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
		min-width: 0;
		padding: 0;
		overflow: hidden;
	}
	.traffic {
		flex: 1 1 auto;
		min-height: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 10px 10px;
	}
	.speeds {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}
	.speed {
		display: grid;
		gap: 1px;
		min-width: 0;
	}
	.speed .v {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 17px;
		font-weight: 700;
		line-height: 1.2;
	}
	.foot {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		font-size: 11px;
		color: var(--fg-muted);
		white-space: nowrap;
	}
	.today {
		flex: none;
		padding: 2px 10px;
		border-radius: 999px;
		background: var(--accent);
		color: #fff;
		font-size: 12px;
		font-weight: 700;
	}
	.plan {
		min-width: 0;
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
		flex: 1 1 46px;
		min-height: 20px;
		max-height: 64px;
	}
	/* Sized by the row, not by the chart's own aspect ratio. */
	.graph :global(svg) {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
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
