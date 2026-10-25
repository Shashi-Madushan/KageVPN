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
		<div class="statusbar {tone}">
			<span class="dot"></span><strong>{statusText}</strong>{#if elapsed}<span class="tabular dim">· {elapsed}</span>{/if}
			{#if app.profiles.length}
				<button class="pill {latencyTone(check)}" onclick={() => app.checkServer()} disabled={pinging || !app.activeProfile} title={app.connected ? 'Check that traffic flows through the tunnel' : 'Check that the server answers'}>
					<Icon name="bolt" size={12} />{check ? (pinging ? 'Pinging…' : latencyLabel(check)) : 'Test latency'}
				</button>
			{/if}
		</div>

		<section class="hero {tone}">
			<MascotButton size={92} />
			<div class="hero-text">
				<h1 class="headline {tone}">{headline}</h1>
				{#if sub}<p class="sub">{sub}</p>{/if}
				{#if app.profiles.length}
					<button class="cta" class:stop={app.connected || app.busy} disabled={app.disconnecting} onclick={() => app.toggle()}>
						{app.disconnecting ? 'Disconnecting…' : app.busy ? 'Cancel' : app.connected ? 'Disconnect' : 'Connect'}
					</button>
				{/if}
			</div>
		</section>

		{#if app.profiles.length}
			{#snippet serverPicker()}
				<Select grow value={serverValue} options={serverOptions} label="Server" placeholder="No servers" onchange={(choice) => app.chooseServer(choice)} />
			{/snippet}
			{#if app.connected || !app.locked}
				<!-- While connected the server changes in place; only the mode needs a disconnect. -->
				<div class="pick" title={app.connected ? 'Switch server without disconnecting' : undefined}>{@render serverPicker()}</div>
			{/if}
			{#if app.connected}
				<div class="ipline" aria-label="Exit IP">
					<small>Exit IP</small>
					{#if app.exitIp}
						<span class="ip-text"><Sensitive id={privacyKey.exitIp}><span class="mono tabular">{app.exitIp}</span></Sensitive></span>
						<PrivacyToggle id={privacyKey.exitIp} label="IP address" />
					{:else if app.tunnelDown}
						<small>No IP</small>
					{:else}
						<Skeleton width="84px" height="14px" />
					{/if}
				</div>
			{/if}

			<div class="tiles">
				<div class="tile"><small>Download</small><strong class="down tabular">{formatRate(app.traffic.down)}</strong></div>
				<div class="tile"><small>Upload</small><strong class="up tabular">{formatRate(app.traffic.up)}</strong></div>
			</div>

			<section class="card usage" class:with-plan={plan} aria-label="Data usage">
				<div class="today-row">
					<span>Today</span>
					<strong class="tabular">{#if usage.todayLoading}<Skeleton width="36px" height="11px" />{:else}{usage.today ? formatBytes(usage.today.up + usage.today.down) : '—'}{/if}</strong>
				</div>
				<div class="graph">
					<Sparkline samples={app.history} height={72} fill />
					{#if !app.connected}<span class="graph-empty">Connect to see live traffic</span>{/if}
				</div>
				{#if plan}<span class="plan" class:low={plan.low} title="What your provider reports for this server's subscription">{plan.text}</span>{/if}
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
	.statusbar {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		color: var(--fg-muted);
		padding-right: 32px;
		white-space: nowrap;
	}
	.statusbar .dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: currentColor;
	}
	.statusbar.ok {
		color: var(--ok-text);
	}
	.statusbar.checking {
		color: var(--warn);
	}
	.statusbar.down {
		color: var(--danger);
	}
	.statusbar .dim {
		color: var(--fg-muted);
	}
	.statusbar .pill {
		margin: 0 0 0 auto;
		min-height: 24px;
		padding: 2px 9px;
		font-size: 11.5px;
		white-space: nowrap;
	}
	.hero {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px 8px 4px;
		border: 1px solid var(--hairline);
		border-radius: 14px;
		background: var(--surface);
	}
	.hero.ok {
		background: linear-gradient(135deg, color-mix(in srgb, var(--ok) 13%, var(--surface)), var(--surface) 75%);
	}
	.hero-text {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 6px;
		min-width: 0;
		flex: 1;
	}
	.cta {
		min-height: 44px;
		align-self: stretch;
		margin-top: 6px;
		padding: 0 14px;
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
	.ipline {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 0 4px;
		font-size: 11px;
		color: var(--fg-muted);
		user-select: text;
	}
	.tiles {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}
	.tile {
		display: grid;
		gap: 2px;
		padding: 8px 12px;
		border: 1px solid var(--hairline);
		border-radius: 12px;
		background: var(--surface);
	}
	.tile small {
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--fg-muted);
	}
	.tile strong {
		font-size: 16px;
	}
	.today-row {
		display: flex;
		justify-content: space-between;
		width: 100%;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.today-row strong {
		color: var(--fg);
	}
	.headline {
		margin: 0;
		font-size: 17px;
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
	.label {
		font-size: 10.5px;
		font-weight: 700;
		letter-spacing: 0.07em;
		text-transform: uppercase;
	}
	.pill {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		min-height: 26px;
		margin-top: 6px;
		padding: 3px 12px;
		border: 1px solid var(--hairline);
		border-radius: 999px;
		background: var(--surface);
		color: var(--fg);
		font-size: 12.5px;
		font-weight: 700;
	}
	.pill:hover:not(:disabled) {
		border-color: var(--accent);
	}
	.pill.ok {
		border-color: color-mix(in srgb, var(--ok) 45%, transparent);
		background: color-mix(in srgb, var(--ok) 12%, var(--surface));
		color: var(--ok-text);
	}
	.pill.warn {
		border-color: color-mix(in srgb, var(--warn) 55%, transparent);
		background: color-mix(in srgb, var(--warn) 14%, var(--surface));
		color: var(--warn);
	}
	.pill.bad {
		border-color: color-mix(in srgb, var(--danger) 50%, transparent);
		background: color-mix(in srgb, var(--danger) 10%, var(--surface));
		color: var(--danger);
	}
	.usage {
		/* Takes the space left over, and gives it back when the rows above need it. */
		flex: 1 1 auto;
		min-height: 0;
		display: grid;
		grid-template-rows: auto minmax(56px, 1fr);
		gap: 8px;
		justify-items: center;
		padding-bottom: 10px;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	.usage.with-plan {
		grid-template-rows: auto minmax(48px, 1fr) auto;
	}
	.plan {
		width: 100%;
		margin-top: -4px;
		font-size: 11px;
		text-align: center;
		color: var(--fg-muted);
	}
	.plan.low {
		color: var(--warn);
		font-weight: 600;
	}
	.graph {
		position: relative;
		width: 100%;
		min-height: 0;
	}
	/* Sized by the row, not by the chart's own aspect ratio. */
	.graph :global(svg) {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
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
