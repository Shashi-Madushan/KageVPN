<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { formatBytes, formatDuration, formatRate, profileFlag } from '$lib/format';
	import { usage } from '$lib/usage.svelte';
	import PowerButton from '$lib/components/PowerButton.svelte';
	import PrivacyToggle from '$lib/components/PrivacyToggle.svelte';
	import Sensitive from '$lib/components/Sensitive.svelte';
	import Select from '$lib/components/Select.svelte';
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

	const serverValue = $derived(app.auto.enabled ? 'auto' : (app.activeId ?? ''));
	const serverOptions = $derived([
		...(app.profiles.length > 1 ? [{ value: 'auto', label: 'Auto (fastest server)', icon: '↯' }] : []),
		...app.profiles.map((p) => ({ value: p.id, label: p.name.replace(/^\p{RI}{2}\s*/u, ''), icon: profileFlag(p.name, p.server) }))
	]);

	const isProtected = $derived(app.status.state === 'connected');
	const isConnecting = $derived(app.status.state === 'connecting');
	const headline = $derived(
		isProtected ? "You're protected" : isConnecting ? 'Connecting…' : "You're not protected"
	);
	const elapsed = $derived(app.status.since ? formatDuration(Math.max(0, now - app.status.since)) : null);
	const activeName = $derived(
		app.auto.enabled ? 'Auto (fastest)' : (app.activeProfile?.name.replace(/^\p{RI}{2}\s*/u, '') ?? null)
	);
</script>

<div class="popover">
	<div class="eye-slot"><PrivacyToggle /></div>
	<button class="close-btn" onclick={() => api.hideTray()} aria-label="Close" title="Close (Esc)">✕</button>

	<section class="hero">
		<PowerButton state={app.status.state} size={76} disabled={app.busy || !app.activeProfile} onclick={() => app.toggle()} />
		<h1 class="headline {app.status.state}">{headline}</h1>
		<p class="sub">
			{#if app.connected}
				{#if activeName}<span class="name" title={activeName}>{activeName}</span>{/if}
				{#if elapsed}<span class="dim tabular">{elapsed}</span>{/if}
				{#if app.delayMs !== null}<span class="dim tabular">{app.delayMs} ms</span>{/if}
			{:else if isConnecting}
				<span class="dim">Establishing secure tunnel…</span>
			{:else if app.profiles.length}
				<span class="dim">Press the button to connect</span>
			{:else}
				<span class="dim">Add a server to get started</span>
			{/if}
		</p>
		{#if app.connected && app.exitIp}<span class="ip mono tabular"><Sensitive>{app.exitIp}</Sensitive></span>{/if}
	</section>

	{#if app.profiles.length}
		<div class="pick">
		<Select
			grow
			value={serverValue}
			options={serverOptions}
			disabled={app.locked}
			label="Server"
			placeholder="No servers"
			onchange={(id) => {
				if (id === 'auto') return app.setAuto({ ...app.auto, enabled: true });
				if (app.auto.enabled) app.setAuto({ ...app.auto, enabled: false });
				app.select(id);
			}}
		/>
		</div>
	{:else}
		<button class="btn primary add" onclick={() => api.openMainWindow()}>Add a server…</button>
	{/if}

	{#if app.profiles.length}
	<section class="stats group" aria-label="Traffic">
		{#if app.connected}
			<div class="speeds">
				<div><span class="caption">Download</span><strong class="tabular down">{formatRate(app.traffic.down)}</strong></div>
				<div><span class="caption">Upload</span><strong class="tabular up">{formatRate(app.traffic.up)}</strong></div>
			</div>
			<div class="graph"><Sparkline samples={app.history} height={34} /></div>
		{/if}
		<div class="today">
			<span>Today <strong class="tabular">{formatBytes((usage.today?.up ?? 0) + (usage.today?.down ?? 0))}</strong></span>
			{#if usage.quota?.limit}<span class="tabular">{usage.quota.percent.toFixed(0)}% of cap</span>{/if}
		</div>
	</section>

	<Segmented options={modes} value={app.mode} label="Mode" disabled={app.locked} onchange={(m) => app.setMode(m)} />
	{#if app.locked}<p class="lock">Disconnect to change server or mode.</p>{/if}
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
		padding: 16px 14px 8px;
		background: color-mix(in srgb, var(--surface-strong) 92%, transparent);
		backdrop-filter: blur(24px);
		-webkit-backdrop-filter: blur(24px);
		border: 1px solid var(--hairline);
		border-radius: 12px;
		overflow: hidden;
		box-sizing: border-box;
	}
	.eye-slot {
		position: absolute;
		top: 8px;
		left: 8px;
	}
	.close-btn {
		position: absolute;
		top: 8px;
		right: 8px;
		width: 28px;
		height: 28px;
		border: 0;
		border-radius: 6px;
		background: none;
		color: var(--fg-muted);
		font-size: 12px;
	}
	.close-btn:hover {
		background: var(--hairline);
		color: var(--fg);
	}
	.hero {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		justify-items: center;
		gap: 4px;
		padding-top: 4px;
	}
	.headline {
		margin: 8px 0 0;
		font-size: 18px;
		font-weight: 650;
		letter-spacing: -0.01em;
	}
	.headline.connected {
		color: var(--ok-text);
	}
	.headline.connecting {
		color: var(--warn);
	}
	.sub {
		margin: 0;
		display: flex;
		align-items: center;
		gap: 8px;
		max-width: 100%;
		min-width: 0;
		font-size: 12px;
	}
	.name {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-weight: 600;
	}
	.dim {
		color: var(--fg-muted);
		white-space: nowrap;
	}
	.sub .dim + .dim::before {
		content: '·';
		margin-right: 8px;
	}
	.name + .dim::before {
		content: '·';
		margin-right: 8px;
	}
	.ip {
		font-size: 11px;
		color: var(--fg-muted);
		user-select: text;
	}
	.add {
		min-height: 34px;
	}
	.pick {
		display: flex;
	}
	.stats {
		padding: 10px 12px 8px;
		display: grid;
		gap: 6px;
	}
	.speeds {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}
	.speeds div {
		display: grid;
	}
	.speeds strong {
		font-size: 14px;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	.today {
		display: flex;
		justify-content: space-between;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.today strong {
		color: var(--fg);
	}
	.lock {
		margin: -4px 0 0;
		font-size: 11px;
		color: var(--fg-muted);
		text-align: center;
	}
	footer {
		margin-top: auto;
		display: flex;
		justify-content: space-between;
		border-top: 1px solid var(--hairline);
		padding-top: 6px;
	}
	.foot-btn {
		min-height: 28px;
		border: 0;
		background: none;
		padding: 4px 8px;
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
