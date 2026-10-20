<script lang="ts">
	import Select from './Select.svelte';
	import Icon, { type IconName } from './Icon.svelte';
	import AppPicker, { appLabel } from './AppPicker.svelte';
	import RuleSources from './RuleSources.svelte';
	import AppsTab from './AppsTab.svelte';
	import type { DnsProvider, RouteAction, RouteMode, RoutingSettings, RuleKind, RuleSetKind } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { CATEGORIES, COUNTRIES, describeList, findEntry } from '$lib/ruleCatalog';
	import { ruleLists } from '$lib/ruleLists.svelte';

	type Tab = 'rules' | 'apps' | 'lists' | 'test';

	let draft = $state<RoutingSettings | null>(null);
	let tab = $state<Tab>('rules');

	// Adding a rule
	let kind = $state<RuleKind>('domainSuffix');
	let value = $state('');
	let action = $state<RouteAction>('direct');

	// Adding a list
	let listGroup = $state<'country' | 'category'>('country');
	let listPick = $state('');
	let customKind = $state<RuleSetKind>('geosite');
	let customName = $state('');
	let setAction = $state<RouteAction>('direct');

	let probe = $state('');

	$effect(() => {
		if (app.routing && !draft) draft = structuredClone($state.snapshot(app.routing));
	});

	$effect(() => {
		void app.routing;
		ruleLists.reload();
	});

	const dirty = $derived(draft !== null && JSON.stringify(draft) !== JSON.stringify(app.routing));

	const modes: { value: RouteMode; label: string; text: string; icon: IconName }[] = [
		{ value: 'global', label: 'Everything through the VPN', text: 'The simplest choice. All your traffic is protected.', icon: 'shield' },
		{ value: 'rule', label: 'Choose by rules', text: 'Send some sites and apps through the VPN and let others go direct.', icon: 'routing' },
		{ value: 'direct', label: 'VPN idle', text: 'The VPN stays on but no traffic uses it. Useful for testing.', icon: 'bolt' }
	];
	const kinds: { value: RuleKind; label: string; help: string; example: string }[] = [
		{ value: 'domainSuffix', label: 'Website and its subdomains', help: 'example.com also covers www.example.com and mail.example.com.', example: 'example.com' },
		{ value: 'domain', label: 'One exact website', help: 'Only this exact address, not its subdomains.', example: 'www.example.com' },
		{ value: 'domainKeyword', label: 'Address containing a word', help: 'Any website whose address contains this word.', example: 'google' },
		{ value: 'app', label: 'An app', help: 'Everything the app sends, including its helper processes.', example: '' },
		{ value: 'ipCidr', label: 'IP address range', help: 'A range of numeric addresses, for example a company network.', example: '10.0.0.0/8' },
		{ value: 'processName', label: 'Process name', help: 'For command-line tools and other programs without an app icon.', example: 'curl' }
	];
	const actions: { value: RouteAction; label: string }[] = [
		{ value: 'proxy', label: 'Use the VPN' },
		{ value: 'direct', label: 'Go direct' },
		{ value: 'block', label: 'Block' }
	];
	const dnsProviders: { value: DnsProvider; label: string }[] = [
		{ value: 'cloudflare', label: 'Cloudflare (1.1.1.1)' },
		{ value: 'google', label: 'Google (8.8.8.8)' },
		{ value: 'quad9', label: 'Quad9 (9.9.9.9)' },
		{ value: 'custom', label: 'Custom DNS server' },
		{ value: 'system', label: 'System resolver' }
	];
	const presets: { id: 'streaming' | 'ai' | 'ads'; title: string; text: string }[] = [
		{ id: 'streaming', title: 'Streaming direct', text: 'Netflix, YouTube, Disney+ and Prime Video use your normal connection.' },
		{ id: 'ai', title: 'AI through the VPN', text: 'ChatGPT and Claude always use the VPN.' },
		{ id: 'ads', title: 'Block ads', text: 'Stops common advertising and tracking addresses.' }
	];
	const tabs = $derived([
		{ id: 'rules' as Tab, label: 'Your rules', count: draft?.rules.length ?? 0 },
		{ id: 'apps' as Tab, label: 'Apps', count: null },
		{ id: 'lists' as Tab, label: 'Ready-made lists', count: draft?.ruleSets.length ?? 0 },
		{ id: 'test' as Tab, label: 'Try a website', count: null }
	]);

	const kindInfo = $derived(kinds.find((k) => k.value === kind) ?? kinds[0]);
	const kindLabel = (k: RuleKind) => kinds.find((x) => x.value === k)?.label ?? k;
	const actionLabel = (a: RouteAction) => actions.find((x) => x.value === a)?.label ?? a;

	function addRule() {
		if (!draft || !value.trim()) return;
		draft.rules.push({ id: crypto.randomUUID(), enabled: true, kind, value: value.trim(), action });
		value = '';
	}

	function move(index: number, delta: number) {
		if (!draft) return;
		const target = index + delta;
		if (target < 0 || target >= draft.rules.length) return;
		[draft.rules[index], draft.rules[target]] = [draft.rules[target], draft.rules[index]];
	}

	function applyPreset(presetType: 'streaming' | 'ai' | 'ads') {
		if (!draft) return;
		const rulesToAdd: { kind: RuleKind; value: string; action: RouteAction }[] = [];
		if (presetType === 'streaming') {
			rulesToAdd.push(
				{ kind: 'domainSuffix', value: 'netflix.com', action: 'direct' },
				{ kind: 'domainSuffix', value: 'youtube.com', action: 'direct' },
				{ kind: 'domainSuffix', value: 'disneyplus.com', action: 'direct' },
				{ kind: 'domainSuffix', value: 'primevideo.com', action: 'direct' }
			);
		} else if (presetType === 'ai') {
			rulesToAdd.push(
				{ kind: 'domainSuffix', value: 'openai.com', action: 'proxy' },
				{ kind: 'domainSuffix', value: 'anthropic.com', action: 'proxy' },
				{ kind: 'domainSuffix', value: 'chatgpt.com', action: 'proxy' },
				{ kind: 'domainSuffix', value: 'claude.ai', action: 'proxy' }
			);
		} else if (presetType === 'ads') {
			rulesToAdd.push(
				{ kind: 'domainSuffix', value: 'doubleclick.net', action: 'block' },
				{ kind: 'domainSuffix', value: 'googleadservices.com', action: 'block' },
				{ kind: 'domainSuffix', value: 'adnxs.com', action: 'block' }
			);
		}
		for (const r of rulesToAdd) {
			if (!draft.rules.some((x) => x.value === r.value)) {
				draft.rules.push({ id: crypto.randomUUID(), enabled: true, ...r });
			}
		}
	}

	const presetAdded = (id: 'streaming' | 'ai' | 'ads') => {
		const probeValue = { streaming: 'netflix.com', ai: 'openai.com', ads: 'doubleclick.net' }[id];
		return !!draft?.rules.some((r) => r.value === probeValue);
	};

	// ---- Ready-made lists ----
	const CUSTOM = '__custom';
	const pool = $derived(listGroup === 'country' ? COUNTRIES : CATEGORIES);
	const pickable = $derived(pool.filter((e) => !draft?.ruleSets.some((s) => s.kind === e.kind && s.country.toLowerCase() === e.name)));
	const pickOptions = $derived([
		...pickable.map((e) => ({ value: e.kind + ':' + e.name, label: e.title, hint: e.hint })),
		{ value: CUSTOM, label: 'Other (enter a name)…' }
	]);
	const customValid = $derived(/^[a-z0-9!-]+$/.test(customName.trim().toLowerCase()) && customName.trim().length > 0);
	const canAddList = $derived(listPick === CUSTOM ? customValid : listPick !== '');

	function choosePick(v: string) {
		listPick = v;
		const entry = v === CUSTOM ? undefined : findEntry(v.split(':')[0] as RuleSetKind, v.split(':')[1]);
		if (entry) setAction = entry.suggested;
	}

	function addList() {
		if (!draft || !canAddList) return;
		let listKind: RuleSetKind;
		let name: string;
		if (listPick === CUSTOM) {
			listKind = customKind;
			name = customName.trim().toLowerCase();
		} else {
			[listKind, name] = listPick.split(':') as [RuleSetKind, string];
		}
		if (draft.ruleSets.some((s) => s.kind === listKind && s.country.toLowerCase() === name)) return;
		draft.ruleSets.push({ id: crypto.randomUUID(), enabled: true, kind: listKind, country: name, action: setAction });
		listPick = '';
		customName = '';
	}

	function listTag(kind: RuleSetKind, name: string) {
		return `${kind === 'geoip' ? 'geoip' : 'geosite'}-${name.trim().toLowerCase()}`;
	}

	function stateOf(kind: RuleSetKind, name: string, saved: boolean): { text: string; tone: 'ok' | 'warn' | 'info' } {
		if (!saved) return { text: 'Save to download', tone: 'info' };
		const present = ruleLists.present(listTag(kind, name));
		if (present === null) return { text: 'Not saved yet', tone: 'info' };
		return present ? { text: 'Ready', tone: 'ok' } : { text: 'Not downloaded', tone: 'warn' };
	}

	const isSaved = (id: string) => !!app.routing?.ruleSets.some((s) => s.id === id && s.enabled);

	const verdict = $derived.by(() => {
		if (!draft || !probe.trim()) return null;
		const host = probe.trim().toLowerCase();
		if (draft.mode === 'direct') return { text: 'Goes direct', why: 'The VPN is idle in this mode.', tone: 'direct' };
		if (draft.mode === 'global') return { text: 'Uses the VPN', why: 'Everything uses the VPN in this mode.', tone: 'proxy' };
		for (const rule of draft.rules) {
			const v = rule.value.toLowerCase();
			const hit =
				rule.enabled &&
				((rule.kind === 'domain' && host === v) ||
					(rule.kind === 'domainSuffix' && (host === v.replace(/^\./, '') || host.endsWith('.' + v.replace(/^\./, '')))) ||
					(rule.kind === 'domainKeyword' && host.includes(v)));
			if (hit) return { text: actionLabel(rule.action), why: `Matched your rule: ${kindLabel(rule.kind)} “${rule.value}”.`, tone: rule.action };
		}
		const fallback = draft.finalAction === 'direct' ? 'direct' : 'proxy';
		return {
			text: fallback === 'direct' ? 'Goes direct' : 'Uses the VPN',
			why: 'None of your rules match. The ready-made lists are checked when you connect, so a site covered only by a list is not shown here.',
			tone: fallback
		};
	});
</script>

{#if draft}
	<div class="page">
		<!-- 1. How traffic is handled -->
		<section class="block">
			<header class="head">
				<h3>How should traffic be handled?</h3>
				<p>Pick one. You can change it any time.</p>
			</header>
			<div class="modes" role="radiogroup" aria-label="Routing mode">
				{#each modes as m (m.value)}
					<button type="button" role="radio" aria-checked={draft.mode === m.value} class="mode" class:active={draft.mode === m.value} onclick={() => draft && (draft.mode = m.value)}>
						<span class="mode-icon"><Icon name={m.icon} size={18} /></span>
						<strong>{m.label}</strong>
						<small>{m.text}</small>
					</button>
				{/each}
			</div>
		</section>

		{#if draft.mode === 'rule'}
			<!-- 2. The default -->
			<section class="block">
				<div class="row-card">
					<div class="grow">
						<h4>Everything else</h4>
						<p>What happens to traffic that none of your rules or lists match.</p>
					</div>
					<div class="pill-choice" role="radiogroup" aria-label="Unmatched traffic">
						{#each [{ v: 'proxy', l: 'Use the VPN' }, { v: 'direct', l: 'Go direct' }] as o (o.v)}
							<button type="button" role="radio" aria-checked={draft.finalAction === o.v} class:active={draft.finalAction === o.v} onclick={() => draft && (draft.finalAction = o.v as RouteAction)}>{o.l}</button>
						{/each}
					</div>
				</div>
			</section>

			<!-- 3. The rules -->
			<section class="block">
				<div class="tabs" role="tablist" aria-label="Routing rules">
					{#each tabs as t (t.id)}
						<button type="button" role="tab" aria-selected={tab === t.id} class:active={tab === t.id} onclick={() => (tab = t.id)}>
							{t.label}{#if t.count}<span class="count">{t.count}</span>{/if}
						</button>
					{/each}
				</div>

				{#if tab === 'apps'}
					<AppsTab {draft} />
				{:else if tab === 'rules'}
					<div class="panel" role="tabpanel">
						<p class="lead">Rules are checked from top to bottom. The first one that matches decides.</p>

						{#if draft.rules.length === 0}
							<div class="starter">
								<h4>Start with a ready-made set</h4>
								<div class="presets">
									{#each presets as p (p.id)}
										<div class="preset">
											<strong>{p.title}</strong>
											<small>{p.text}</small>
											<button type="button" class="btn" onclick={() => applyPreset(p.id)}>Add</button>
										</div>
									{/each}
								</div>
							</div>
						{:else}
							<div class="group">
								{#each draft.rules as rule, i (rule.id)}
									<div class="item rule" class:off={!rule.enabled}>
										<span class="order">{i + 1}</span>
										<input type="checkbox" bind:checked={rule.enabled} aria-label="Turn this rule on or off" />
										<span class="grow what">
											<span class="val" title={rule.value}>{rule.kind === 'app' ? appLabel(rule.value) : rule.value}</span>
											<small>{kindLabel(rule.kind)}</small>
										</span>
										<span class="act"><Select bind:value={rule.action} label="What to do" options={actions} grow /></span>
										<span class="tools">
											<button class="icon-btn" onclick={() => move(i, -1)} disabled={i === 0} aria-label="Move up">↑</button>
											<button class="icon-btn" onclick={() => move(i, 1)} disabled={i === draft.rules.length - 1} aria-label="Move down">↓</button>
											<button class="icon-btn del" onclick={() => draft?.rules.splice(i, 1)} aria-label="Delete rule">✕</button>
										</span>
									</div>
								{/each}
							</div>
						{/if}

						<form class="adder" onsubmit={(e) => (e.preventDefault(), addRule())}>
							<h4>Add a rule</h4>
							<div class="fields">
								<label class="field">
									<span class="field-label">Match</span>
									<Select bind:value={kind} label="What to match" options={kinds} onchange={() => (value = '')} />
								</label>
								<label class="field grow">
									<span class="field-label">{kind === 'app' ? 'App' : kind === 'ipCidr' ? 'IP range' : kind === 'processName' ? 'Process' : 'Address'}</span>
									{#if kind === 'app'}
										<AppPicker bind:value />
									{:else}
										<input class="input" bind:value placeholder={kindInfo.example} aria-label="Value to match" spellcheck="false" />
									{/if}
								</label>
								<label class="field">
									<span class="field-label">Then</span>
									<Select bind:value={action} label="What to do" options={actions} />
								</label>
								<button class="btn primary" disabled={!value.trim()}>Add rule</button>
							</div>
							<small class="help">{kindInfo.help}</small>
						</form>

						{#if draft.rules.length > 0}
							<div class="more">
								<span>More ready-made sets:</span>
								{#each presets as p (p.id)}
									<button type="button" class="chip" disabled={presetAdded(p.id)} onclick={() => applyPreset(p.id)} title={p.text}>{presetAdded(p.id) ? '✓ ' : '+ '}{p.title}</button>
								{/each}
							</div>
						{/if}
						{#if kind === 'app' || draft.rules.some((r) => r.kind === 'app')}
							<p class="note">Safari and apps built on its engine send their traffic from a shared system process, so they cannot be routed by app. Use a website rule for them.</p>
						{/if}
					</div>
				{:else if tab === 'lists'}
					<div class="panel" role="tabpanel">
						<p class="lead">Lists are kept up to date for you: whole countries, ad networks, and popular services. Turn on what you need and choose what to do with it.</p>

						{#if draft.ruleSets.length > 0}
							<div class="group">
								{#each draft.ruleSets as set, i (set.id)}
									{@const info = describeList(set.kind, set.country)}
									{@const st = stateOf(set.kind, set.country, isSaved(set.id))}
									<div class="item rule" class:off={!set.enabled}>
										<input type="checkbox" bind:checked={set.enabled} aria-label="Turn this list on or off" />
										<span class="grow what">
											<span class="val">{info.title}<span class="tag">{set.kind === 'geoip' ? 'Country' : 'Websites'}</span></span>
											<small>{info.hint}</small>
										</span>
										{#if set.enabled}<span class="status {st.tone}">{st.text}</span>{/if}
										<span class="act"><Select bind:value={set.action} label="What to do" options={actions} grow /></span>
										<span class="tools"><button class="icon-btn del" onclick={() => draft?.ruleSets.splice(i, 1)} aria-label="Remove list">✕</button></span>
									</div>
								{/each}
							</div>
						{/if}

						<form class="adder" onsubmit={(e) => (e.preventDefault(), addList())}>
							<h4>Add a list</h4>
							<div class="fields">
								<div class="field">
									<span class="field-label">Type</span>
									<div class="pill-choice" role="radiogroup" aria-label="List type">
										<button type="button" role="radio" aria-checked={listGroup === 'country'} class:active={listGroup === 'country'} onclick={() => ((listGroup = 'country'), (listPick = ''))}>Countries</button>
										<button type="button" role="radio" aria-checked={listGroup === 'category'} class:active={listGroup === 'category'} onclick={() => ((listGroup = 'category'), (listPick = ''))}>Websites &amp; services</button>
									</div>
								</div>
								<label class="field grow">
									<span class="field-label">{listGroup === 'country' ? 'Country' : 'Service or category'}</span>
									<Select value={listPick} label="List" options={pickOptions} placeholder="Choose…" onchange={choosePick} />
								</label>
								<label class="field">
									<span class="field-label">Then</span>
									<Select bind:value={setAction} label="What to do" options={actions} />
								</label>
								<button class="btn primary" disabled={!canAddList}>Add list</button>
							</div>
							{#if listPick === CUSTOM}
								<div class="fields custom">
									<div class="field">
										<span class="field-label">Kind</span>
										<Select bind:value={customKind} label="Kind of list" options={[{ value: 'geosite', label: 'Websites' }, { value: 'geoip', label: 'Countries (addresses)' }]} />
									</div>
									<label class="field grow">
										<span class="field-label">List name</span>
										<input class="input" bind:value={customName} placeholder={customKind === 'geoip' ? 'a country code such as de' : 'a category such as category-ads-all'} spellcheck="false" maxlength="48" />
									</label>
								</div>
							{/if}
							<small class="help">
								{#if listGroup === 'country'}
									A country list matches network addresses registered in that country. “Go direct” is common for your own country.
								{:else}
									A service list matches the websites a service uses, for example every Netflix address.
								{/if}
							</small>
						</form>

						<RuleSources {draft} {dirty} />
					</div>
				{:else}
					<div class="panel" role="tabpanel">
						<p class="lead">Type a website to see what your rules would do with it.</p>
						<input class="input big" bind:value={probe} placeholder="example.com" aria-label="Website to test" spellcheck="false" />
						{#if verdict}
							<div class="result {verdict.tone}">
								<strong>{verdict.text}</strong>
								<small>{verdict.why}</small>
							</div>
						{/if}
					</div>
				{/if}
			</section>
		{/if}

		<!-- 4. Network -->
		<section class="block">
			<header class="head">
				<h3>Network</h3>
				<p>Applies in every mode.</p>
			</header>
			<div class="group">
				<label class="item">
					<span class="grow">Keep my local network reachable<span class="sub">Printers, routers and other devices at home or work keep working.</span></span>
					<input type="checkbox" bind:checked={draft.bypassLan} />
				</label>
				<label class="item">
					<span class="grow">Allow IPv6<span class="sub">Turn off if a server has no IPv6 and some apps are slow to connect. IPv6 traffic is then blocked, never sent around the VPN.</span></span>
					<input type="checkbox" bind:checked={draft.ipv6} />
				</label>
				<div class="item">
					<span class="grow">DNS provider<span class="sub">{draft.dns.provider === 'system' ? 'Your internet provider can see which sites you look up.' : 'Lookups go through the VPN, so they stay private.'}</span></span>
					<Select bind:value={draft.dns.provider} label="DNS provider" options={dnsProviders} />
				</div>
				{#if draft.dns.provider === 'custom'}
					<div class="item"><input class="input grow" bind:value={draft.dns.custom} placeholder="dns.example.com or https://dns.example.com/your-id" aria-label="Custom DNS server" spellcheck="false" /></div>
				{/if}
				<label class="item">
					<span class="grow">Route by site name (FakeIP)<span class="sub">Full system mode only. Sites are routed by their name, which keeps domain rules exact and avoids wrong answers from your local resolver. Turn off if an app that remembers addresses misbehaves.</span></span>
					<input type="checkbox" bind:checked={draft.dns.fakeIp} />
				</label>
			</div>
		</section>

		{#if dirty}
			<div class="savebar" role="status">
				<span>You have unsaved changes. They apply the next time you connect.</span>
				<button class="btn" onclick={() => (draft = structuredClone($state.snapshot(app.routing!)))}>Discard</button>
				<button class="btn primary" onclick={() => draft && app.saveRouting($state.snapshot(draft) as RoutingSettings)}>Save changes</button>
			</div>
		{/if}
	</div>
{/if}

<style>
	.page {
		display: grid;
		gap: 22px;
		align-content: start;
		container-type: inline-size;
	}
	h3,
	h4,
	p {
		margin: 0;
	}
	h3 {
		font-size: 15px;
		font-weight: 650;
	}
	h4 {
		font-size: 13px;
		font-weight: 650;
	}
	p,
	.lead,
	.help,
	small {
		color: var(--fg-muted);
		font-size: 12px;
	}
	.block {
		display: grid;
		gap: 10px;
	}
	.head {
		display: grid;
		gap: 2px;
		padding: 0 2px;
	}

	/* Modes */
	.modes {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 10px;
	}
	.mode {
		display: grid;
		gap: 4px;
		justify-items: start;
		text-align: left;
		padding: 14px;
		border: 1px solid var(--hairline);
		border-radius: 12px;
		background: var(--surface);
		transition: border-color 0.15s, background 0.15s;
	}
	.mode:hover {
		border-color: color-mix(in srgb, var(--accent) 50%, var(--hairline));
	}
	.mode.active {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 9%, var(--surface));
	}
	.mode-icon {
		display: grid;
		place-items: center;
		width: 32px;
		height: 32px;
		border-radius: 9px;
		background: var(--hairline);
		color: var(--fg-muted);
		margin-bottom: 4px;
	}
	.mode.active .mode-icon {
		background: var(--accent);
		color: #fff;
	}
	.mode strong {
		font-size: 13px;
	}

	/* A single setting as a card */
	.row-card {
		display: flex;
		align-items: center;
		gap: 12px;
		flex-wrap: wrap;
		padding: 12px 14px;
		border: 1px solid var(--hairline);
		border-radius: 12px;
		background: var(--surface);
	}
	.grow {
		flex: 1;
		min-width: 0;
	}

	.pill-choice {
		display: inline-flex;
		gap: 2px;
		padding: 2px;
		border-radius: 9px;
		background: var(--hairline);
	}
	.pill-choice button {
		border: 0;
		background: none;
		padding: 5px 12px;
		border-radius: 7px;
		font-size: 12px;
		color: var(--fg-muted);
		white-space: nowrap;
	}
	.pill-choice button.active {
		background: var(--surface-strong);
		color: var(--fg);
		font-weight: 600;
		box-shadow: 0 1px 2px rgb(0 0 0 / 0.15);
	}

	/* Tabs */
	.tabs {
		display: flex;
		gap: 4px;
		border-bottom: 1px solid var(--hairline);
	}
	.tabs button {
		position: relative;
		border: 0;
		background: none;
		padding: 8px 12px;
		font-size: 13px;
		color: var(--fg-muted);
		display: inline-flex;
		gap: 6px;
		align-items: center;
	}
	.tabs button.active {
		color: var(--fg);
		font-weight: 600;
	}
	.tabs button.active::after {
		content: '';
		position: absolute;
		left: 8px;
		right: 8px;
		bottom: -1px;
		height: 2px;
		border-radius: 2px;
		background: var(--accent);
	}
	.count {
		min-width: 18px;
		padding: 0 6px;
		border-radius: 9px;
		background: var(--hairline);
		font-size: 11px;
		line-height: 18px;
		text-align: center;
		color: var(--fg-muted);
	}
	.panel {
		display: grid;
		gap: 14px;
		padding-top: 6px;
	}

	/* Rule and list rows */
	.rule.off .what,
	.rule.off .status {
		opacity: 0.45;
	}
	.order {
		width: 18px;
		flex: none;
		font-size: 11px;
		color: var(--fg-muted);
		text-align: center;
	}
	.what {
		display: grid;
		gap: 1px;
	}
	.val {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-weight: 550;
		user-select: text;
	}
	.tag {
		margin-left: 8px;
		padding: 1px 7px;
		border-radius: 8px;
		background: var(--hairline);
		font-size: 10.5px;
		font-weight: 500;
		color: var(--fg-muted);
	}
	.act {
		width: 128px;
		flex: none;
	}
	.status {
		width: 112px;
		flex: none;
		font-size: 11.5px;
		white-space: nowrap;
		display: inline-flex;
		align-items: center;
		gap: 5px;
	}
	.status::before {
		content: '';
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--fg-muted);
	}
	.status.ok {
		color: var(--ok-text);
	}
	.status.ok::before {
		background: var(--ok);
	}
	.status.warn {
		color: var(--warn);
	}
	.status.warn::before {
		background: var(--warn);
	}
	.status.info::before {
		background: var(--accent);
	}
	.tools {
		display: flex;
		opacity: 0;
		transition: opacity 0.12s;
	}
	.rule:hover .tools,
	.rule:focus-within .tools {
		opacity: 1;
	}
	.icon-btn {
		min-width: 26px;
		min-height: 26px;
		border: 0;
		background: none;
		border-radius: 6px;
		color: var(--fg-muted);
	}
	.icon-btn:hover:not(:disabled) {
		background: var(--hairline);
		color: var(--fg);
	}
	.icon-btn.del:hover {
		color: var(--danger);
	}

	/* Starter, presets and chips */
	.starter {
		display: grid;
		gap: 10px;
		padding: 14px;
		border: 1px dashed var(--hairline);
		border-radius: 12px;
	}
	.presets {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 10px;
	}
	.preset {
		display: grid;
		gap: 4px;
		align-content: start;
		justify-items: start;
		padding: 12px;
		border-radius: 10px;
		background: var(--surface);
		border: 1px solid var(--hairline);
	}
	.preset strong {
		font-size: 13px;
	}
	.preset .btn {
		margin-top: 6px;
	}
	.more {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		align-items: center;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.chip {
		border: 1px solid var(--hairline);
		background: var(--surface);
		border-radius: 14px;
		padding: 3px 10px;
		font-size: 12px;
	}
	.chip:hover:not(:disabled) {
		border-color: var(--accent);
	}
	.chip:disabled {
		opacity: 0.6;
	}

	/* Add forms */
	.adder {
		display: grid;
		gap: 10px;
		padding: 14px;
		border: 1px solid var(--hairline);
		border-radius: 12px;
		background: color-mix(in srgb, var(--hairline) 25%, transparent);
	}
	.fields {
		display: flex;
		flex-wrap: wrap;
		gap: 12px;
		align-items: flex-end;
	}
	.fields.custom {
		padding-top: 2px;
	}
	.field {
		display: grid;
		gap: 5px;
		min-width: 0;
	}
	.field.grow {
		flex: 1;
		min-width: 160px;
	}
	.field-label {
		font-size: 11.5px;
		font-weight: 600;
		color: var(--fg);
	}
	.note {
		font-size: 12px;
		color: var(--fg-muted);
	}

	/* Try a website */
	.big {
		min-height: 36px;
		font-size: 14px;
	}
	.result {
		display: grid;
		gap: 3px;
		padding: 12px 14px;
		border-radius: 10px;
		border: 1px solid var(--hairline);
		border-left-width: 4px;
	}
	.result strong {
		font-size: 14px;
	}
	.result.proxy {
		border-left-color: var(--ok);
	}
	.result.direct {
		border-left-color: var(--accent);
	}
	.result.block {
		border-left-color: var(--danger);
	}

	.savebar {
		position: sticky;
		bottom: 0;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-radius: 10px;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		box-shadow: 0 8px 24px rgb(0 0 0 / 0.25);
	}
	.savebar span {
		flex: 1;
	}
	@container (max-width: 620px) {
		.modes,
		.presets {
			grid-template-columns: 1fr;
		}
		.tools {
			opacity: 1;
		}
		.status {
			display: none;
		}
	}
</style>
