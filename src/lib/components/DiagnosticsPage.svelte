<script lang="ts">
	import { api, type Diagnostics, type IpInfo, type SpeedResult } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { core } from '$lib/core.svelte';

	let result = $state<Diagnostics | null>(null);
	let speed = $state<SpeedResult | null>(null);
	let checking = $state(false);
	let testing = $state(false);

	async function run() {
		checking = true;
		result = await api.runDiagnostics().catch((e) => (app.notify(String(e)), null));
		checking = false;
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

<div class="page">
	<section class="card block">
		<div class="head">
			<div><strong>Leak check</strong><small>Compares the tunnel IP, your system route and the DNS resolver the internet sees.</small></div>
			<button class="primary" disabled={checking} onclick={run}>{checking ? 'Checking…' : 'Run checks'}</button>
		</div>
		{#if result}
			<div class="grid">
				<div class="cell"><span class="caption">Tunnel</span><strong class="mono">{result.tunnel?.ip ?? '—'}</strong><small>{place(result.tunnel)}</small><small>{result.tunnel?.org}</small></div>
				<div class="cell"><span class="caption">System route</span><strong class="mono">{result.system?.ip ?? '—'}</strong><small>{place(result.system)}</small><small>{result.system?.org}</small></div>
				<div class="cell"><span class="caption">DNS resolver</span><strong class="mono">{result.dns?.ip ?? '—'}</strong><small>{result.dns?.geo}</small></div>
			</div>
			{#if verdict}
				<div class="verdict-box {verdict.tone}">
					<p class="verdict {verdict.tone}">{verdict.text}</p>
					{#if app.status.mode === 'proxy'}
						<div class="remediation">
							<small>Want all apps protected?</small>
							<button class="fix-btn" onclick={() => app.setMode('tun')}>Switch to TUN Mode</button>
						</div>
					{/if}
				</div>
			{/if}
		{/if}
	</section>

	<section class="card block">
		<div class="head">
			<div><strong>Speed test</strong><small>Measured through the tunnel with Cloudflare. Uses about 30 MB.</small></div>
			<button class="primary" disabled={testing || !app.connected} onclick={runSpeed}>{testing ? 'Testing…' : 'Start test'}</button>
		</div>
		{#if !app.connected}<small>Connect first.</small>{/if}
		{#if speed}
			<div class="grid two">
				<div class="cell"><span class="caption">Download</span><strong class="big down tabular">{speed.downMbps.toFixed(1)} <small>Mbps</small></strong></div>
				<div class="cell"><span class="caption">Upload</span><strong class="big up tabular">{speed.upMbps.toFixed(1)} <small>Mbps</small></strong></div>
			</div>
		{/if}
	</section>

	<section class="card block">
		<div class="head">
			<div><strong>Support report</strong><small>Copies versions, results and recent errors. Review it before sharing.</small></div>
			<button onclick={copyReport}>Copy diagnostics</button>
		</div>
	</section>
</div>

<style>
	.page {
		display: grid;
		gap: 14px;
		align-content: start;
	}
	.block {
		padding: 14px 16px;
		display: grid;
		gap: 12px;
	}
	.head {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 16px;
	}
	.head div {
		display: grid;
	}
	small {
		color: var(--fg-muted);
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
		gap: 10px;
	}
	.grid.two {
		grid-template-columns: 1fr 1fr;
	}
	.cell {
		display: grid;
		gap: 2px;
		padding: 10px 12px;
		border-radius: var(--radius-sm);
		background: var(--hairline);
	}
	.cell strong {
		user-select: text;
	}
	.big {
		font-size: 22px;
	}
	.down {
		color: var(--down);
	}
	.up {
		color: var(--up);
	}
	.verdict {
		margin: 0;
		padding: 9px 12px;
		border-radius: var(--radius-sm);
		font-weight: 600;
	}
	.verdict.ok {
		background: color-mix(in srgb, var(--ok) 16%, transparent);
		color: var(--ok);
	}
	.verdict.bad {
		background: color-mix(in srgb, var(--danger) 16%, transparent);
		color: var(--danger);
	}
	.verdict.idle {
		background: var(--hairline);
		color: var(--fg-muted);
	}
	.verdict-box {
		display: grid;
		gap: 6px;
	}
	.remediation {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 2px 4px;
	}
	.fix-btn {
		font-size: 11px;
		padding: 3px 10px;
		border-radius: 6px;
		background: var(--accent);
		color: white;
		border: 0;
		font-weight: 600;
		cursor: pointer;
	}
	button {
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		border-radius: 8px;
		padding: 6px 14px;
		white-space: nowrap;
	}
	button.primary {
		background: var(--accent);
		border-color: var(--accent);
		color: white;
		font-weight: 600;
	}
</style>
