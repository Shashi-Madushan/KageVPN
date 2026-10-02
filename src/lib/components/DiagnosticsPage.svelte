<script lang="ts">
	import { api, type Diagnostics, type IpInfo, type SpeedResult } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { core } from '$lib/core.svelte';
	import HeaderActions from './HeaderActions.svelte';
	import Icon from './Icon.svelte';

	let result = $state<Diagnostics | null>(null);
	let speed = $state<SpeedResult | null>(null);
	let checking = $state(false);
	let testing = $state(false);
	let checkedAt = $state<Date | null>(null);

	async function run() {
		checking = true;
		result = await api.runDiagnostics().catch((e) => (app.notify(String(e)), null));
		checking = false;
		checkedAt = new Date();
	}

	async function runSpeed() {
		testing = true;
		speed = null;
		speed = await api.runSpeedTest().catch((e) => (app.notify(String(e)), null));
		testing = false;
	}

	const place = (info: IpInfo | null) => (info ? [info.city, info.country].filter(Boolean).join(', ') : '');

	const verdict = $derived.by(() => {
		if (!result) return null;
		if (!app.connected) return { tone: 'idle', text: 'Not connected. Connect to compare the tunnel with your system route.' };
		if (!result.tunnel) return { tone: 'bad', text: 'The tunnel did not answer. Check the connection.' };
		if (app.status.mode === 'tun' && result.system && result.system.ip !== result.tunnel.ip) {
			return { tone: 'bad', text: 'Possible leak: your system route shows a different IP than the tunnel.' };
		}
		if (app.status.mode === 'proxy') {
			return { tone: 'idle', text: 'Proxy-only mode: only apps using the local proxy go through the VPN.' };
		}
		return { tone: 'ok', text: 'No IP leak detected. System traffic exits through the tunnel.' };
	});

	async function copyReport() {
		const errors = app.logs.filter((l) => l.level === 'error').slice(-8).map((l) => l.text);
		const lines = [
			'SingBox GUI diagnostics',
			`State: ${app.status.state} (${app.status.mode})`,
			`Profile: ${app.activeProfile ? `${app.activeProfile.security}/${app.activeProfile.transport.kind}` : 'none'}`,
			`Core: ${core.status?.active ? `${core.status.active.version} (${core.status.active.source})` : 'none'}`,
			`Tunnel IP: ${result?.tunnel?.ip ?? 'n/a'} ${place(result?.tunnel ?? null)}`,
			`System IP: ${result?.system?.ip ?? 'n/a'} ${place(result?.system ?? null)}`,
			`DNS resolver: ${result?.dns ? `${result.dns.ip} (${result.dns.geo})` : 'n/a'}`,
			speed ? `Speed: ${speed.downMbps.toFixed(1)} Mbps down, ${speed.upMbps.toFixed(1)} Mbps up` : '',
			'Recent errors (review before sharing):',
			...errors
		].filter(Boolean);
		await navigator.clipboard.writeText(lines.join('\n')).catch(() => null);
		app.notify('Diagnostics copied.', 'info');
	}
</script>

<HeaderActions>
	<button class="btn" onclick={copyReport}>Copy report</button>
</HeaderActions>

<div class="page">
	<section class="card status {verdict?.tone ?? 'none'}" aria-live="polite">
		<div class="status-main">
			<span class="badge"><Icon name="shield" size={22} /></span>
			<div class="status-text">
				<strong>{verdict ? { ok: 'No leaks detected', bad: 'Needs attention', idle: 'Check complete' }[verdict.tone] : 'Leak check'}</strong>
				<span>{verdict?.text ?? 'Compares the tunnel IP, your system route and the DNS resolver the internet sees.'}</span>
				{#if checkedAt}<small>Last run {checkedAt.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</small>{/if}
			</div>
			<button class="btn primary" disabled={checking} onclick={run}>{checking ? 'Checking…' : result ? 'Run again' : 'Run checks'}</button>
		</div>
		{#if verdict && app.connected && app.status.mode === 'proxy'}
			<div class="remediation">
				<span>Want every app protected?</span>
				<button class="btn" onclick={() => app.setMode('tun')}>Switch to full system (TUN)</button>
			</div>
		{/if}
	</section>

	{#if result}
		<section class="grid">
			<div class="card cell"><span class="caption">Tunnel</span><strong class="mono">{result.tunnel?.ip ?? '—'}</strong><small>{place(result.tunnel) || 'Unknown location'}</small>{#if result.tunnel?.org}<small>{result.tunnel.org}</small>{/if}</div>
			<div class="card cell"><span class="caption">System route</span><strong class="mono">{result.system?.ip ?? '—'}</strong><small>{place(result.system) || 'Unknown location'}</small>{#if result.system?.org}<small>{result.system.org}</small>{/if}</div>
			<div class="card cell"><span class="caption">DNS resolver</span><strong class="mono">{result.dns?.ip ?? '—'}</strong><small>{result.dns?.geo || 'Unknown location'}</small></div>
		</section>
	{/if}

	<section class="card speed">
		<div class="speed-head">
			<div><strong>Speed test</strong><small>Measured through the tunnel with Cloudflare. Uses about 30 MB.</small></div>
			<button class="btn primary" disabled={testing || !app.connected} onclick={runSpeed}>{testing ? 'Testing…' : speed ? 'Test again' : 'Start test'}</button>
		</div>
		{#if !app.connected}
			<div class="hint"><small>Connect to the VPN to measure its speed.</small><button class="btn" disabled={app.busy || !app.activeProfile} onclick={() => app.toggle()}>Connect</button></div>
		{:else if testing}
			<div class="progress" aria-label="Speed test running"><div class="bar"></div></div>
		{:else if speed}
			<div class="results">
				<div><span class="caption">Download</span><strong class="big down tabular">{speed.downMbps.toFixed(1)}<small> Mbps</small></strong></div>
				<div><span class="caption">Upload</span><strong class="big up tabular">{speed.upMbps.toFixed(1)}<small> Mbps</small></strong></div>
			</div>
		{/if}
	</section>
</div>

<style>
	.page {
		display: grid;
		gap: 12px;
		align-content: start;
	}
	small {
		color: var(--fg-muted);
	}
	.status {
		padding: 16px 18px;
		display: grid;
		gap: 12px;
	}
	.status.ok {
		border-color: color-mix(in srgb, var(--ok) 50%, var(--hairline));
		background: color-mix(in srgb, var(--ok) 8%, var(--surface));
	}
	.status.bad {
		border-color: var(--danger);
		background: color-mix(in srgb, var(--danger) 8%, var(--surface));
	}
	.status-main {
		display: flex;
		align-items: center;
		gap: 14px;
		flex-wrap: wrap;
	}
	.badge {
		display: grid;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: 12px;
		flex: none;
		background: var(--hairline);
		color: var(--fg-muted);
	}
	.ok .badge {
		background: color-mix(in srgb, var(--ok) 20%, transparent);
		color: var(--ok);
	}
	.bad .badge {
		background: color-mix(in srgb, var(--danger) 20%, transparent);
		color: var(--danger);
	}
	.status-text {
		flex: 1;
		min-width: 200px;
		display: grid;
		gap: 2px;
	}
	.status-text strong {
		font-size: 15px;
	}
	.status-text span {
		color: var(--fg-muted);
	}
	.remediation {
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 8px;
		padding-top: 12px;
		border-top: 1px solid var(--hairline);
		color: var(--fg-muted);
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 10px;
	}
	.cell {
		display: grid;
		gap: 3px;
		padding: 12px 14px;
		align-content: start;
	}
	.cell strong {
		font-size: 15px;
		user-select: text;
	}
	.speed {
		padding: 14px 16px;
		display: grid;
		gap: 12px;
	}
	.speed-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 10px 16px;
	}
	.speed-head div {
		display: grid;
	}
	.hint {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		padding: 10px 12px;
		border-radius: var(--radius-sm);
		background: var(--hairline);
	}
	.results {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
	}
	.results > div {
		display: grid;
		gap: 2px;
		padding: 12px 14px;
		border-radius: var(--radius-sm);
		background: var(--hairline);
	}
	.big {
		font-size: 28px;
		font-weight: 650;
		letter-spacing: -0.01em;
	}
	.big small {
		margin-left: 4px;
		font-size: 13px;
		font-weight: 400;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	.progress {
		height: 5px;
		border-radius: 3px;
		background: var(--hairline);
		overflow: hidden;
	}
	.bar {
		height: 100%;
		width: 100%;
		background: var(--accent);
		transform-origin: left;
		animation: slide 1.1s ease-in-out infinite;
	}
	@keyframes slide {
		50% {
			transform: scaleX(0.35);
		}
	}
</style>
