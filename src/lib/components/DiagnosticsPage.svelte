<script lang="ts">
	import { onDestroy } from 'svelte';
	import { api, type Diagnostics, type IpInfo, type SpeedProgress, type SpeedResult } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { core } from '$lib/core.svelte';
	import HeaderActions from './HeaderActions.svelte';
	import Icon from './Icon.svelte';
	import SpeedGauge from './SpeedGauge.svelte';

	let result = $state<Diagnostics | null>(null);
	let speed = $state<SpeedResult | null>(null);
	let live = $state<SpeedProgress | null>(null);
	let samples = $state<number[]>([]);
	let partial = $state<{ ping: number | null; jitter: number | null; down: number | null; up: number | null }>({
		ping: null,
		jitter: null,
		down: null,
		up: null
	});
	let speedError = $state<string | null>(null);
	let unlisten: (() => void) | null = null;
	let checking = $state(false);
	let testing = $state(false);
	let checkedAt = $state<Date | null>(null);

	async function run() {
		checking = true;
		result = await api.runDiagnostics().catch((e) => (app.notify(String(e)), null));
		checking = false;
		checkedAt = new Date();
	}

	const mean = (values: number[]) => (values.length ? values.reduce((a, b) => a + b, 0) / values.length : null);

	function onProgress(event: SpeedProgress) {
		if (live && live.phase !== event.phase) {
			// A phase just finished: show its average right away, the exact figure arrives at the end.
			if (live.phase === 'download') partial.down = mean(samples.slice(Math.floor(samples.length * 0.2)));
			samples = [];
		}
		if (event.phase === 'download' || event.phase === 'upload') samples = [...samples, event.mbps];
		if (event.pingMs !== null) {
			partial.ping = event.pingMs;
			partial.jitter = event.jitterMs;
		}
		live = event;
	}

	async function runSpeed() {
		testing = true;
		speed = null;
		speedError = null;
		live = null;
		samples = [];
		partial = { ping: null, jitter: null, down: null, up: null };
		unlisten = await api.onSpeedProgress(onProgress);
		try {
			speed = await api.runSpeedTest();
			partial = { ping: speed.pingMs, jitter: speed.jitterMs, down: speed.downMbps, up: speed.upMbps };
		} catch (e) {
			const message = String(e);
			if (!message.includes('cancelled')) speedError = message;
		} finally {
			unlisten?.();
			unlisten = null;
			live = null;
			samples = [];
			testing = false;
		}
	}

	function cancelSpeed() {
		api.cancelSpeedTest();
	}

	onDestroy(() => {
		if (testing) api.cancelSpeedTest();
		unlisten?.();
	});

	const gaugePhase = $derived(live ? live.phase : speed ? 'done' : 'idle');

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
			speed ? `Speed: ${speed.downMbps.toFixed(1)} Mbps down, ${speed.upMbps.toFixed(1)} Mbps up, ping ${speed.pingMs.toFixed(0)} ms (jitter ${speed.jitterMs.toFixed(0)} ms)` : '',
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
			<div><strong>Speed test</strong><small>Measured through the tunnel with Cloudflare. About 8 seconds each way; uses roughly 30–100 MB.</small></div>
			{#if testing}
				<button class="btn" onclick={cancelSpeed}>Cancel</button>
			{:else}
				<button class="btn primary" disabled={!app.connected} onclick={runSpeed}>{speed ? 'Test again' : 'Start test'}</button>
			{/if}
		</div>

		<SpeedGauge phase={gaugePhase} mbps={live?.mbps ?? 0} {samples} pingMs={live?.pingMs ?? partial.ping} final={speed?.downMbps ?? null} />

		<div class="results" aria-live="polite">
			<div class="res" class:now={live?.phase === 'ping'}>
				<span class="caption">Ping</span>
				<strong class="tabular" class:empty={partial.ping === null}>{partial.ping === null ? '—' : partial.ping.toFixed(0)}<small> ms</small></strong>
				<span class="sub tabular">{partial.jitter === null ? 'Jitter —' : `Jitter ${partial.jitter.toFixed(0)} ms`}</span>
			</div>
			<div class="res down" class:now={live?.phase === 'download'}>
				<span class="caption">Download</span>
				<strong class="tabular" class:empty={partial.down === null}>{partial.down === null ? '—' : partial.down.toFixed(1)}<small> Mbps</small></strong>
			</div>
			<div class="res up" class:now={live?.phase === 'upload'}>
				<span class="caption">Upload</span>
				<strong class="tabular" class:empty={partial.up === null}>{partial.up === null ? '—' : partial.up.toFixed(1)}<small> Mbps</small></strong>
			</div>
		</div>

		{#if speedError}<p class="speed-error" role="alert">{speedError}</p>{/if}
		{#if !app.connected && !testing}
			<div class="hint"><small>Connect to the VPN to measure its speed.</small><button class="btn" disabled={app.busy || !app.activeProfile} onclick={() => app.toggle()}>Connect</button></div>
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
		padding: 14px 16px 16px;
		display: grid;
		gap: 12px;
	}
	.speed-head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 10px 16px;
	}
	.speed-head div {
		display: grid;
		flex: 1;
		min-width: 240px;
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
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 8px;
	}
	.res {
		display: grid;
		gap: 2px;
		padding: 10px 12px;
		border-radius: var(--radius-sm);
		border: 1px solid transparent;
		background: var(--hairline);
		transition: border-color 0.2s;
	}
	.res.now {
		border-color: var(--accent);
	}
	.res.down.now {
		border-color: var(--down);
	}
	.res.up.now {
		border-color: var(--up);
	}
	.res strong {
		font-size: 20px;
		font-weight: 650;
	}
	.res.down strong:not(.empty) {
		color: var(--down);
	}
	.res.up strong:not(.empty) {
		color: var(--up);
	}
	.res strong.empty {
		color: var(--fg-muted);
	}
	.res strong small {
		font-size: 11px;
		font-weight: 400;
		color: var(--fg-muted);
	}
	.res .sub {
		font-size: 11px;
		color: var(--fg-muted);
	}
	.speed-error {
		margin: 0;
		padding: 8px 10px;
		border-radius: 8px;
		font-size: 12px;
		color: var(--danger);
		background: color-mix(in srgb, var(--danger) 10%, transparent);
	}
</style>
