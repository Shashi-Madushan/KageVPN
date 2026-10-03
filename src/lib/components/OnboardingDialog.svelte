<script lang="ts">
	import type { Sample } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { core } from '$lib/core.svelte';
	import { onboarding } from '$lib/onboarding.svelte';
	import Dialog from './Dialog.svelte';
	import PowerButton from './PowerButton.svelte';
	import Segmented from './Segmented.svelte';
	import Select from './Select.svelte';
	import Sparkline from './Sparkline.svelte';
	import type { Page } from './Sidebar.svelte';

	let { onnavigate }: { onnavigate: (page: Page) => void } = $props();

	const slides = $derived([
		{ title: `Welcome to ${app.info.name}`, text: 'A simple way to run a private, encrypted connection on your Mac. This quick tour takes about a minute.' },
		{ title: 'Add a server', text: 'Your VPN provider gives you a vless:// link or a subscription URL. Paste it once and the app saves and keeps it for you.' },
		{ title: 'Choose how to connect', text: 'Full system sends every app through the VPN. Proxy only protects just the apps you point at the local proxy.' },
		{ title: 'Connect in one click', text: 'Pick a server and press the power button. Turn on auto mode and the app picks the fastest server for you.' },
		{ title: 'Always within reach', text: 'Connect, switch servers and watch your speed from the menu bar, without opening the window.' },
		{ title: "You're all set", text: 'Here is where your setup stands.' }
	]);

	let step = $state(0);
	const last = $derived(slides.length - 1);
	const hasServer = $derived(app.profiles.length > 0);
	const hotkey = $derived(
		app.reliability.hotkey && app.hotkey ? app.hotkey.replace('CommandOrControl', '⌘').replace('Alt', '⌥').replace(/\+/g, ' ') : ''
	);

	// Static sample data so the demos look alive without touching the real connection.
	const demoSamples: Sample[] = Array.from({ length: 48 }, (_, i) => ({
		down: 900_000 + Math.sin(i / 4) * 600_000 + Math.sin(i / 1.7) * 150_000,
		up: 120_000 + Math.cos(i / 5) * 60_000
	}));
	const modes = [
		{ value: 'tun' as const, label: 'Full System (TUN)' },
		{ value: 'proxy' as const, label: 'Proxy Only' }
	];
	const demoServers = [{ value: 'fra', label: 'Frankfurt', icon: '🇩🇪', hint: '42 ms' }];

	$effect(() => {
		if (onboarding.open) step = 0;
	});

	const next = () => (step = Math.min(last, step + 1));
	const back = () => (step = Math.max(0, step - 1));

	function primary() {
		if (step < last) return next();
		if (!hasServer) {
			onboarding.addServerRequested = true;
			onnavigate('profiles');
		}
		onboarding.finish();
	}

	function installEngine() {
		core.promptInstall = true;
		onboarding.finish();
	}

	function onkeydown(e: KeyboardEvent) {
		if (!onboarding.open || e.metaKey || e.ctrlKey || e.altKey) return;
		if (e.key === 'ArrowRight') (e.preventDefault(), next());
		if (e.key === 'ArrowLeft') (e.preventDefault(), back());
	}
</script>

<svelte:window {onkeydown} />

<Dialog open={onboarding.open} title="Welcome tour" width={620} bare onclose={() => onboarding.finish()}>
	<div class="stage" aria-hidden="true">
		{#key step}
			<div class="demo" inert>
				{#if step === 0}
					<div class="d-hero">
						<PowerButton state="connected" size={84} onclick={() => {}} />
						<strong class="d-headline">You're protected</strong>
						<span class="d-chips"><i>Kill switch</i><i>Auto-reconnect</i></span>
					</div>
				{:else if step === 1}
					<div class="d-card d-add">
						<div class="d-title">Add server</div>
						<div class="d-textarea mono">vless://3f9a1c2e-…@vpn.example.com:443<br />?security=reality&amp;sni=…#Frankfurt</div>
						<div class="d-row">
							<span class="d-ok">1 link ready to add</span>
							<span class="d-btn primary">Add server</span>
						</div>
					</div>
				{:else if step === 2}
					<div class="d-modes">
						<Segmented options={modes} value="tun" label="Connection mode" onchange={() => {}} />
						<div class="d-mode-cards">
							<div class="d-card on"><strong>Full system</strong><span>Every app and background service goes through the VPN.</span></div>
							<div class="d-card"><strong>Proxy only</strong><span>Only apps set to use the local proxy are protected.</span></div>
						</div>
					</div>
				{:else if step === 3}
					<div class="d-connect">
						<div class="d-field"><span class="caption">Server</span><Select grow size="lg" value="fra" label="Server" options={demoServers} /></div>
						<div class="d-power">
							<PowerButton state="connected" size={68} onclick={() => {}} />
							<span class="d-ok">Protected · 00:12:36</span>
						</div>
						<div class="d-spark"><Sparkline samples={demoSamples} height={48} /></div>
					</div>
				{:else if step === 4}
					<div class="d-tray">
						<div class="d-menubar">
							<span class="mb-icon"><svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" aria-hidden="true"><path d="M12 3v8" /><path d="M6.6 6.6a8 8 0 1 0 10.8 0" /></svg></span>
							<span class="mb-speed mono"><i>↑ 178K</i><i>↓ 2.3M</i></span>
						</div>
						<div class="d-tray-card">
							<PowerButton state="connected" size={46} onclick={() => {}} />
							<strong class="d-headline small">You're protected</strong>
							<span class="d-sub">Frankfurt · 00:12:36</span>
							<div class="d-tray-speeds"><span><em>Download</em><b class="down">2.3 MB/s</b></span><span><em>Upload</em><b class="up">178 KB/s</b></span></div>
						</div>
					</div>
				{:else}
					<div class="d-check">
						<div class="d-card">
							<div class="d-line"><span>Engine</span><span class="state" class:ok={!core.missing} class:bad={core.missing}>{core.missing ? 'Not installed' : 'Ready'}</span></div>
							<div class="d-line"><span>Server</span><span class="state" class:ok={hasServer} class:warn={!hasServer}>{hasServer ? `${app.profiles.length} added` : 'None yet'}</span></div>
						</div>
					</div>
				{/if}
			</div>
		{/key}
	</div>

	<div class="copy">
		{#key step}
			<div class="copy-in">
				<h2>{slides[step].title}</h2>
				<p>{slides[step].text}</p>
				{#if step === 4 && hotkey}<p class="tip">Tip: press <kbd>{hotkey}</kbd> from anywhere to connect or disconnect.</p>{/if}
				{#if step === 1}<p class="tip">You can also drop a text file, or use <em>Enter details manually</em> if you have the values.</p>{/if}
				{#if step === last}<p class="tip">You can replay this tour any time from Settings.</p>{/if}
				{#if step === last && core.missing}
					<button class="btn" onclick={installEngine}>Download the engine</button>
				{/if}
			</div>
		{/key}
	</div>

	{#snippet footer()}
		<div class="dots" role="tablist" aria-label="Tour steps">
			{#each slides as slide, i (slide.title)}
				<button type="button" role="tab" class="dot" class:on={i === step} aria-selected={i === step} aria-label="Step {i + 1}: {slide.title}" onclick={() => (step = i)}></button>
			{/each}
		</div>
		{#if step < last}<button class="linkbtn skip" onclick={() => onboarding.finish()}>Skip</button>{/if}
		{#if step > 0}<button class="btn" onclick={back}>Back</button>{/if}
		<button class="btn primary" onclick={primary}>
			{step < last ? 'Next' : hasServer ? 'Get started' : 'Add a server'}
		</button>
	{/snippet}
</Dialog>

<style>
	.stage {
		position: relative;
		display: grid;
		place-items: center;
		height: 252px;
		overflow: hidden;
		border-bottom: 1px solid var(--hairline);
		background:
			radial-gradient(120% 90% at 50% 0%, color-mix(in srgb, var(--accent) 14%, transparent), transparent 70%),
			color-mix(in srgb, var(--hairline) 35%, transparent);
	}
	.demo {
		width: 100%;
		display: grid;
		place-items: center;
		padding: 0 28px;
		animation: rise 0.28s ease-out;
	}
	.copy-in {
		display: grid;
		justify-items: center;
		gap: 8px;
		animation: rise 0.28s ease-out;
	}
	@keyframes rise {
		from {
			opacity: 0;
			transform: translateY(6px);
		}
	}
	.copy {
		min-height: 168px;
		padding: 20px 40px 16px;
		text-align: center;
	}
	h2 {
		margin: 0;
		font-size: 19px;
		font-weight: 650;
		letter-spacing: -0.01em;
	}
	.copy p {
		margin: 0;
		max-width: 440px;
		font-size: 13.5px;
		line-height: 1.55;
		color: var(--fg-muted);
	}
	.copy .tip {
		font-size: 12.5px;
	}
	kbd {
		padding: 1px 6px;
		border-radius: 5px;
		border: 1px solid var(--hairline);
		background: var(--surface);
		font: inherit;
		color: var(--fg);
	}
	.copy .btn {
		margin-top: 4px;
	}

	/* Demo pieces, drawn with the same tokens as the real screens */
	.d-hero,
	.d-check,
	.d-tray {
		display: grid;
		justify-items: center;
		gap: 10px;
	}
	.d-headline {
		font-size: 18px;
		font-weight: 650;
		color: var(--ok-text);
	}
	.d-headline.small {
		font-size: 14px;
	}
	.d-chips {
		display: flex;
		gap: 6px;
	}
	.d-chips i {
		font-style: normal;
		font-size: 11px;
		font-weight: 600;
		padding: 3px 9px;
		border-radius: 999px;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		color: var(--fg-muted);
	}
	.d-card {
		display: grid;
		gap: 6px;
		padding: 14px;
		border-radius: 10px;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		box-shadow: 0 8px 24px rgb(0 0 0 / 0.12);
	}
	.d-add {
		width: min(100%, 400px);
	}
	.d-title {
		font-weight: 650;
	}
	.d-textarea {
		padding: 9px 10px;
		border-radius: 8px;
		border: 1px solid var(--hairline);
		background: var(--surface);
		font-size: 11px;
		line-height: 1.55;
		color: var(--fg-muted);
		word-break: break-all;
	}
	.d-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.d-ok {
		font-size: 12px;
		font-weight: 600;
		color: var(--ok-text);
	}
	.d-btn {
		padding: 5px 12px;
		border-radius: 8px;
		font-size: 12px;
		font-weight: 600;
		border: 1px solid var(--hairline);
	}
	.d-btn.primary {
		background: var(--accent);
		border-color: var(--accent);
		color: white;
	}
	.d-modes {
		display: grid;
		gap: 12px;
		width: min(100%, 460px);
	}
	.d-mode-cards {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
	}
	.d-mode-cards .d-card {
		box-shadow: none;
		text-align: left;
		font-size: 12px;
	}
	.d-mode-cards .d-card span {
		color: var(--fg-muted);
		line-height: 1.45;
	}
	.d-mode-cards .d-card.on {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 10%, var(--surface-strong));
	}
	.d-connect {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: center;
		gap: 14px 24px;
		width: min(100%, 470px);
	}
	.d-field {
		display: grid;
		gap: 6px;
	}
	.d-power {
		display: grid;
		justify-items: center;
		gap: 6px;
	}
	.d-spark {
		grid-column: 1 / -1;
		padding: 8px 10px;
		border-radius: 10px;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
	}
	.d-spark :global(svg) {
		display: block;
		width: 100%;
	}
	.d-tray {
		gap: 6px;
		justify-items: end;
		width: 230px;
	}
	.d-menubar {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 2px 10px;
		border-radius: 6px;
		background: rgb(20 20 22 / 0.78);
		color: white;
	}
	.mb-icon {
		display: grid;
		place-items: center;
	}
	.mb-speed {
		display: grid;
		font-size: 9px;
		line-height: 1.15;
	}
	.mb-speed i {
		font-style: normal;
	}
	.d-tray-card {
		justify-self: center;
		display: grid;
		justify-items: center;
		gap: 4px;
		width: 190px;
		padding: 16px 14px 12px;
		border-radius: 12px;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		box-shadow: 0 10px 30px rgb(0 0 0 / 0.18);
	}
	.d-sub {
		font-size: 11px;
		color: var(--fg-muted);
	}
	.d-tray-speeds {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
		margin-top: 8px;
		padding-top: 8px;
		border-top: 1px solid var(--hairline);
		font-size: 11px;
	}
	.d-tray-speeds span {
		display: grid;
	}
	.d-tray-speeds em {
		font-style: normal;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--fg-muted);
	}
	.d-tray-speeds .down {
		color: var(--down);
	}
	.d-tray-speeds .up {
		color: var(--up);
	}
	.d-check .d-card {
		width: 300px;
		gap: 0;
		padding: 4px 14px;
	}
	.d-line {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 9px 0;
		font-size: 13px;
	}
	.d-line + .d-line {
		border-top: 1px solid var(--hairline);
	}

	.dots {
		display: flex;
		gap: 2px;
		margin-right: auto;
	}
	.dot {
		width: 20px;
		height: 28px;
		padding: 0;
		border: 0;
		background: none;
		display: grid;
		place-items: center;
	}
	.dot::before {
		content: '';
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--hairline);
		transition:
			width 0.2s,
			background 0.2s;
	}
	.dot:hover::before {
		background: var(--fg-muted);
	}
	.dot.on::before {
		width: 18px;
		border-radius: 4px;
		background: var(--accent);
	}
	.skip {
		margin-right: 6px;
		color: var(--fg-muted);
	}
	@media (max-width: 560px) {
		.copy {
			padding-inline: 20px;
		}
		.d-connect {
			grid-template-columns: 1fr;
		}
	}
</style>
