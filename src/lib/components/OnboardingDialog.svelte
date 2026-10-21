<script lang="ts">
	import { app } from '$lib/app.svelte';
	import { core } from '$lib/core.svelte';
	import { onboarding } from '$lib/onboarding.svelte';
	import Dialog from './Dialog.svelte';
	import PoweredBy from './PoweredBy.svelte';
	import Icon from './Icon.svelte';
	import Mascot from './Mascot.svelte';
	import AppIcon from './AppIcon.svelte';
	import CapBar from './CapBar.svelte';
	import Segmented from './Segmented.svelte';
	import Select from './Select.svelte';
	import { installedApps } from './AppPicker.svelte';
	import { importMessage } from '$lib/format';
	import type { InstalledApp, RouteAction } from '$lib/api';
	import type { Page } from './Sidebar.svelte';

	let { onnavigate }: { onnavigate: (page: Page) => void } = $props();

	let step = $state(0);

	const slides = $derived([
		{ id: 'welcome', title: `Welcome to ${app.info.name}`, text: 'A simple way to run a private, encrypted connection on your Mac. Meet the Kage mascot: it is your connect button and your status light. Three quick steps get you set up.' },
		{ id: 'engine', title: 'Get the engine', text: 'The engine is the program that carries your traffic. It is downloaded once from its official release page and checked before use.' },
		{ id: 'add', title: 'Add a server', text: 'Your VPN provider gives you a server link (vless://, vmess://, trojan://, ss://, hysteria://, hysteria2://, tuic://, ssh:// or wireguard://, or a WireGuard .conf file) or a subscription URL. Paste it here and the app saves and keeps it for you.' },
		{ id: 'mode', title: 'Choose how to connect', text: 'Full system sends every app through the VPN and asks for your password once to set it up. Proxy only protects just the apps you point at the local proxy. You can change this later.' },
		{ id: 'tap', title: 'Tap the mascot to connect', text: 'Pick a server, then tap the mascot to start the VPN. Tap it again any time to disconnect. A sleeping mascot means you are not connected; a smiling one means you are protected.' },
		{ id: 'face', title: 'Read the mascot’s face', text: 'The mascot shows what is happening. “Protected” appears only after a real request has gone through the tunnel. If it looks worried, you are connected but no internet is getting through. Open Home to see why and use the one-click fix.' },
		{ id: 'tray', title: 'Always within reach', text: 'Click the menu bar icon to open the popover. Tap the mascot there too, check your latency, switch mode and watch live traffic, without opening the window.' },
		{ id: 'usage', title: 'See what uses your data', text: 'Data usage tells you in plain words how much you used this month. Connections shows which apps are sending and receiving data right now.' },
		{ id: 'rules', title: 'Decide what uses the VPN', text: 'Under Routing you can send some sites through the VPN, let others go direct, or block them. Add your own rules, pick ready-made lists (a country, streaming, ads), or type a website to see what would happen to it.' },
		{ id: 'apps', title: 'Control each app', text: 'App controls lists the apps on your Mac. For each one choose VPN, Direct or Block, and see what it does right now. The Connections page has the same Route button on every app that is sending data.' },
		{ id: 'caps', title: 'Set a data limit for an app', text: 'Give any app a daily, weekly or monthly limit. You are told at 80% and 100%, and can choose to block the app until the period ends. Nothing is blocked unless you say so.' },
		{ id: 'done', title: "You're all set", text: 'Here is where your setup stands.' }
	]);
	const id = $derived(slides[step].id);

	let pasted = $state('');
	let adding = $state(false);
	let addError = $state<string | null>(null);
	const isSubscription = $derived(/^https?:\/\/\S+$/i.test(pasted.trim()));

	async function addServer() {
		adding = true;
		addError = null;
		if (isSubscription) {
			addError = await app.addSubscription('My subscription', pasted.trim(), { userAgent: null, sendDeviceId: false });
		} else {
			const { summary, error } = await app.importText(pasted);
			addError = error;
			if (summary) app.notify(importMessage(summary), 'info');
		}
		adding = false;
		if (!addError) pasted = '';
	}


	// The app steps show three of this Mac's own apps with their real icons.
	const FALLBACK_APPS: InstalledApp[] = ['Safari', 'Mail', 'Notes'].map((name) => ({ name, path: `/Applications/${name}.app`, bundleId: null }));
	const PREFERRED = ['Safari', 'Mail', 'Notes', 'Messages', 'Calendar', 'Maps', 'Music', 'Photos'];
	let sample = $state<InstalledApp[]>([]);
	const demoApps = $derived(sample.length === 3 ? sample : FALLBACK_APPS);

	$effect(() => {
		if (!onboarding.open) return;
		installedApps().then((list) => {
			const known = PREFERRED.map((name) => list.find((a) => a.name === name)).filter((a): a is InstalledApp => !!a);
			const rest = list.filter((a) => !known.includes(a));
			sample = [...known, ...rest].slice(0, 3);
		});
	});

	const appChoices: { value: RouteAction; label: string }[] = [
		{ value: 'proxy', label: 'VPN' },
		{ value: 'direct', label: 'Direct' },
		{ value: 'block', label: 'Block' }
	];
	const demoChoice: RouteAction[] = ['proxy', 'direct', 'block'];
	const ruleActions: { value: RouteAction; label: string }[] = [
		{ value: 'proxy', label: 'Use the VPN' },
		{ value: 'direct', label: 'Go direct' },
		{ value: 'block', label: 'Block' }
	];
	let demoRules = $state<{ value: string; action: RouteAction }[]>([
		{ value: 'netflix.com', action: 'direct' },
		{ value: 'company.internal', action: 'proxy' },
		{ value: 'ads.example.com', action: 'block' }
	]);

	const last = $derived(slides.length - 1);
	const hasServer = $derived(app.profiles.length > 0);
	const hotkey = $derived(
		app.reliability.hotkey && app.hotkey ? app.hotkey.replace('CommandOrControl', '⌘').replace('Alt', '⌥').replace(/\+/g, ' ') : ''
	);

	const modes = [
		{ value: 'tun' as const, label: 'Full System (TUN)' },
		{ value: 'proxy' as const, label: 'Proxy Only' }
	];

	$effect(() => {
		if (onboarding.open) step = 0;
	});

	const next = () => (step = Math.min(last, step + 1));
	const back = () => (step = Math.max(0, step - 1));

	function primary() {
		if (step < last) return next();
		if (!hasServer) onnavigate('profiles');
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

<Dialog open={onboarding.open} title="Set up" width={620} bare onclose={() => onboarding.finish()}>
	<div class="stage" aria-hidden="true">
		{#key step}
			<div class="demo" inert>
					{#if id === 'welcome'}
						<div class="d-hero">
							<Mascot mood="happy" size={118} />
							<strong class="d-headline">You're protected</strong>
							<span class="d-chips"><i>Kill switch</i><i>Auto-reconnect</i></span>
						</div>
					{:else if id === 'add'}
						<div class="d-card d-add">
							<div class="d-title">Add server</div>
							<div class="d-textarea mono">vless://3f9a1c2e-…@vpn.example.com:443<br />?security=reality&amp;sni=…#Frankfurt</div>
							<div class="d-row">
								<span class="d-ok">1 link ready to add</span>
								<span class="d-btn primary">Add server</span>
							</div>
						</div>
					{:else if id === 'mode'}
						<div class="d-modes">
							<Segmented options={modes} value="tun" label="Connection mode" onchange={() => {}} />
							<div class="d-mode-cards">
								<div class="d-card on"><strong>Full system</strong><span>Every app and background service goes through the VPN.</span></div>
								<div class="d-card"><strong>Proxy only</strong><span>Only apps set to use the local proxy are protected.</span></div>
							</div>
						</div>
					{:else if id === 'tap'}
						<div class="d-tap">
							<div class="d-tap-side">
								<span class="d-tap-btn"><i class="ripple"></i><Mascot mood="sleep" size={104} /><span class="finger"><Icon name="bolt" size={14} /> Tap</span></span>
								<span class="d-cap">Not connected</span>
							</div>
							<span class="d-arrow" aria-hidden="true">→</span>
							<div class="d-tap-side">
								<span class="d-tap-btn"><Mascot mood="happy" size={104} /></span>
								<span class="d-cap ok">Protected · tap to disconnect</span>
							</div>
						</div>
					{:else if id === 'face'}
						<div class="d-moods">
							<div><Mascot mood="sleep" size={84} /><b>Off</b><small>Tap to connect</small></div>
							<div><Mascot mood="busy" size={84} /><b class="warn">Checking</b><small>Verifying it works</small></div>
							<div><Mascot mood="happy" size={84} /><b class="ok">Protected</b><small>Traffic is flowing</small></div>
							<div><Mascot mood="worried" size={84} /><b class="bad">No internet</b><small>Open Home to fix</small></div>
						</div>
					{:else if id === 'tray'}
						<div class="d-tray">
							<div class="d-menubar">
								<span class="mb-icon"><svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" aria-hidden="true"><path d="M12 3v8" /><path d="M6.6 6.6a8 8 0 1 0 10.8 0" /></svg></span>
								<span class="mb-speed mono"><i>↑ 178K</i><i>↓ 2.3M</i></span>
							</div>
							<div class="d-tray-card">
								<Mascot mood="happy" size={64} />
								<strong class="d-headline small">YOU'RE PROTECTED</strong>
								<span class="d-pill"><Icon name="bolt" size={11} /> 38 ms</span>
								<div class="d-tray-speeds"><span><em>Download</em><b class="down">2.3 MB/s</b></span><span><em>Upload</em><b class="up">178 KB/s</b></span></div>
							</div>
						</div>
					{:else if id === 'usage'}
						<div class="d-card d-usage">
							<span class="d-sub">You used</span>
							<strong class="d-big">37 GB <small>this month</small></strong>
							<div class="d-bar"><i style="width: 84%"></i></div>
							<div class="d-apps">
								<div><span>Safari</span><i style="width: 90%"></i><b>5.4 GB</b></div>
								<div><span>Slack</span><i style="width: 46%"></i><b>2.5 GB</b></div>
								<div><span>Spotify</span><i style="width: 30%"></i><b>1.6 GB</b></div>
							</div>
						</div>
					{:else if id === 'rules'}
						<div class="d-card d-rules">
							<div class="d-title">Your rules</div>
							{#each demoRules as rule, i (rule.value)}
								<div class="d-rule">
									<span class="d-num">{i + 1}</span>
									<span class="mono grow">{rule.value}</span>
									<Select bind:value={rule.action} label="What to do with {rule.value}" options={ruleActions} />
								</div>
							{/each}
							<div class="d-rule">
								<span class="d-num">·</span>
								<span class="grow">Everything else</span>
								<span class="d-pill-choice"><i class="on">Use the VPN</i><i>Go direct</i></span>
							</div>
						</div>
					{:else if id === 'apps'}
						<div class="d-card d-appctl">
							<div class="d-title">App controls</div>
							{#each demoApps as a, i (a.path)}
								<div class="d-app">
									<AppIcon path={a.path} name={a.name} />
									<span class="grow d-name">{a.name}</span>
									<Segmented options={appChoices} value={demoChoice[i]} label="What {a.name} does" onchange={() => {}} />
								</div>
							{/each}
						</div>
					{:else if id === 'caps'}
						<div class="d-card d-capdemo">
							<div class="d-app">
								<AppIcon path={demoApps[0].path} name={demoApps[0].name} />
								<span class="grow d-name">{demoApps[0].name}<small>No rule, follows the default (the VPN)</small></span>
								<span class="d-edit">Edit cap</span>
							</div>
							<CapBar percent={100} tone="over" label="Data cap used" height={8} />
							<small>2 GB of 2 GB today · resets at midnight</small>
							<div class="d-notice">
								<span>{demoApps[0].name} reached its cap. Blocking reconnects the VPN once.</span>
								<span class="d-pair"><span class="btn small danger">Block now</span><span class="btn small">Not now</span></span>
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
				{#if id === 'tray' && hotkey}<p class="tip">Tip: press <kbd>{hotkey}</kbd> from anywhere to connect or disconnect.</p>{/if}
				{#if id === 'welcome'}<div class="brand"><PoweredBy align="start" /></div>{/if}
				{#if id === 'engine'}
					{#if core.missing}
						{#if core.installing}
							<p class="tip">Downloading{core.percent !== null ? ` ${Math.round(core.percent)}%` : '…'}</p>
						{:else}
							<button class="btn primary" onclick={() => core.installBackground()}>Download the engine</button>
							{#if core.failure}<p class="tip bad">{core.failure}</p>{/if}
						{/if}
					{:else}
						<p class="tip ok">Engine ready{core.activeVersion ? ` (${core.activeVersion})` : ''}.</p>
					{/if}
				{/if}
				{#if id === 'add'}
					{#if hasServer}
						<p class="tip ok">{app.profiles.length} server{app.profiles.length === 1 ? '' : 's'} added. Add more or continue.</p>
					{/if}
					<textarea class="input mono paste" bind:value={pasted} rows="3" spellcheck="false" placeholder="vless://…  or  https://provider.example/subscription" aria-label="Server link or subscription URL"></textarea>
					<button class="btn" disabled={!pasted.trim() || adding} onclick={addServer}>{adding ? 'Adding…' : isSubscription ? 'Add subscription' : 'Add server'}</button>
					{#if addError}<p class="tip bad" role="alert">{addError}</p>{/if}
				{/if}
				{#if id === 'mode'}
					<Segmented options={modes} value={app.mode} label="Connection mode" onchange={(m) => app.setMode(m)} />
				{/if}
				{#if id === 'done'}<p class="tip">You can run this setup again any time from Settings.</p>{/if}
				{#if step === last && core.missing}
					<button class="btn" onclick={installEngine}>Download the engine</button>
				{/if}
			</div>
		{/key}
	</div>

	{#snippet footer()}
		<div class="dots" role="tablist" aria-label="Setup steps">
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
		min-height: 196px;
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
	.copy .tip.ok {
		color: var(--ok, var(--accent));
	}
	.copy .tip.bad {
		color: var(--danger, #d33);
	}
	.paste {
		width: 100%;
		max-width: 420px;
		font-size: 11.5px;
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

	.d-tap {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 28px;
	}
	.d-tap-side {
		display: grid;
		justify-items: center;
		gap: 8px;
	}
	.d-tap-btn {
		position: relative;
		display: grid;
		place-items: center;
	}
	.ripple {
		position: absolute;
		inset: 8px;
		border-radius: 50%;
		border: 2px solid var(--accent);
		animation: ripple 1.8s ease-out infinite;
	}
	@keyframes ripple {
		from {
			transform: scale(0.7);
			opacity: 0.9;
		}
		to {
			transform: scale(1.25);
			opacity: 0;
		}
	}
	.finger {
		position: absolute;
		right: -14px;
		bottom: 8px;
		display: inline-flex;
		align-items: center;
		gap: 3px;
		padding: 3px 9px;
		border-radius: 999px;
		background: var(--accent);
		color: #fff;
		font-size: 11px;
		font-weight: 700;
		box-shadow: 0 4px 12px rgb(0 0 0 / 0.2);
	}
	.d-arrow {
		font-size: 26px;
		color: var(--fg-muted);
	}
	.d-cap {
		font-size: 12px;
		font-weight: 600;
		color: var(--fg-muted);
	}
	.d-cap.ok {
		color: var(--ok-text);
	}
	.d-moods {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 10px;
		width: min(100%, 520px);
	}
	.d-moods div {
		display: grid;
		justify-items: center;
		gap: 2px;
		padding: 10px 4px 8px;
		border-radius: 12px;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
	}
	.d-moods b {
		font-size: 12px;
		margin-top: 4px;
	}
	.d-moods b.ok {
		color: var(--ok-text);
	}
	.d-moods b.warn {
		color: var(--warn);
	}
	.d-moods b.bad {
		color: var(--danger);
	}
	.d-moods small {
		font-size: 10.5px;
		color: var(--fg-muted);
	}
	.d-pill {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 10px;
		border-radius: 999px;
		border: 1px solid color-mix(in srgb, var(--ok) 45%, transparent);
		background: color-mix(in srgb, var(--ok) 12%, var(--surface));
		color: var(--ok-text);
		font-size: 11.5px;
		font-weight: 700;
	}
	.d-usage {
		width: min(100%, 380px);
		gap: 4px;
	}
	.d-big {
		font-size: 30px;
		letter-spacing: -0.02em;
	}
	.d-big small {
		font-size: 13px;
		font-weight: 500;
		color: var(--fg-muted);
		letter-spacing: 0;
	}
	.d-bar {
		height: 8px;
		border-radius: 4px;
		background: var(--up);
		overflow: hidden;
		margin: 4px 0 6px;
	}
	.d-bar i {
		display: block;
		height: 100%;
		background: var(--down);
	}
	.d-apps {
		display: grid;
		gap: 7px;
	}
	.d-apps div {
		display: grid;
		grid-template-columns: 64px 1fr auto;
		align-items: center;
		gap: 8px;
		font-size: 11.5px;
	}
	.d-apps i {
		display: block;
		height: 6px;
		border-radius: 3px;
		background: var(--accent);
	}
	.d-apps b {
		font-variant-numeric: tabular-nums;
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
	}
	.brand {
		margin-top: 14px;
	}
	.d-rules,
	.d-appctl,
	.d-capdemo {
		width: min(100%, 420px);
		gap: 9px;
		text-align: left;
	}
	.d-rule,
	.d-app {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 12.5px;
	}
	.grow {
		flex: 1;
		min-width: 0;
	}
	.d-num {
		width: 16px;
		text-align: center;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.d-name {
		display: grid;
		font-weight: 600;
	}
	.d-name small,
	.d-capdemo small {
		font-weight: 400;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.d-pill-choice {
		display: inline-flex;
		gap: 2px;
		padding: 2px;
		border-radius: 999px;
		background: var(--hairline);
	}
	.d-pill-choice i {
		font-style: normal;
		font-size: 11px;
		padding: 2px 9px;
		border-radius: 999px;
		color: var(--fg-muted);
	}
	.d-pill-choice i.on {
		border: 1px solid var(--accent);
		color: var(--accent);
	}
	.d-edit {
		padding: 3px 12px;
		border-radius: 999px;
		border: 1px solid var(--hairline);
		font-size: 12px;
		color: var(--fg-muted);
	}
	.d-notice {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 8px 10px;
		border-radius: 8px;
		background: color-mix(in srgb, var(--danger) 14%, transparent);
		font-size: 12px;
	}
	.d-pair {
		display: flex;
		gap: 6px;
		flex: none;
	}
	.d-notice .btn.small {
		min-height: 24px;
		padding: 2px 10px;
		font-size: 12px;
		margin: 0;
	}
</style>
