<script lang="ts">
	import Select from './Select.svelte';
	import type { DnsProvider, RouteAction, RouteMode, RoutingSettings, RuleKind, RuleSetKind } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import Segmented from './Segmented.svelte';

	let draft = $state<RoutingSettings | null>(null);
	let kind = $state<RuleKind>('domainSuffix');
	let value = $state('');
	let action = $state<RouteAction>('direct');
	let setKind = $state<RuleSetKind>('geoip');
	let country = $state('');
	let setAction = $state<RouteAction>('direct');
	let probe = $state('');

	$effect(() => {
		if (app.routing && !draft) draft = structuredClone($state.snapshot(app.routing));
	});

	const dirty = $derived(draft !== null && JSON.stringify(draft) !== JSON.stringify(app.routing));
	const modes: { value: RouteMode; label: string; text: string }[] = [
		{ value: 'global', label: 'Global', text: 'Everything goes through the VPN.' },
		{ value: 'rule', label: 'Rules', text: 'Your rules decide; anything unmatched follows the default below.' },
		{ value: 'direct', label: 'Direct', text: 'The VPN stays on but nothing is routed through it. Useful for testing.' }
	];
	const kinds: { value: RuleKind; label: string }[] = [
		{ value: 'domain', label: 'Domain' },
		{ value: 'domainSuffix', label: 'Domain suffix' },
		{ value: 'domainKeyword', label: 'Domain keyword' },
		{ value: 'ipCidr', label: 'IP range (CIDR)' },
		{ value: 'processName', label: 'App (process name)' }
	];
	const actions: { value: RouteAction; label: string }[] = [
		{ value: 'proxy', label: 'VPN' },
		{ value: 'direct', label: 'Direct' },
		{ value: 'block', label: 'Block' }
	];
	const providers: { value: DnsProvider; label: string }[] = [
		{ value: 'cloudflare', label: 'Cloudflare (1.1.1.1)' },
		{ value: 'google', label: 'Google (8.8.8.8)' },
		{ value: 'quad9', label: 'Quad9 (9.9.9.9)' },
		{ value: 'custom', label: 'Custom DNS server' },
		{ value: 'system', label: 'System resolver' }
	];

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

	function addSet() {
		const code = country.trim().toLowerCase();
		if (!draft || !/^[a-z0-9-]+$/.test(code)) return;
		draft.ruleSets.push({ id: crypto.randomUUID(), enabled: true, kind: setKind, country: code, action: setAction });
		country = '';
	}

	const verdict = $derived.by(() => {
		if (!draft || !probe.trim()) return null;
		const host = probe.trim().toLowerCase();
		if (draft.mode === 'direct') return 'Direct (routing mode is Direct)';
		if (draft.mode === 'global') return 'VPN (routing mode is Global)';
		for (const rule of draft.rules) {
			const v = rule.value.toLowerCase();
			const hit =
				rule.enabled &&
				((rule.kind === 'domain' && host === v) ||
					(rule.kind === 'domainSuffix' && (host === v.replace(/^\./, '') || host.endsWith('.' + v.replace(/^\./, '')))) ||
					(rule.kind === 'domainKeyword' && host.includes(v)));
			if (hit) return `${actionLabel(rule.action)} (rule: ${kindLabel(rule.kind)} "${rule.value}")`;
		}
		return `${draft.finalAction === 'direct' ? 'Direct' : 'VPN'} (no rule matched, default)`;
	});
</script>

{#if draft}
	<div class="page">
		<div class="section">
			<div class="section-head"><span class="caption">Mode</span></div>
			<Segmented options={modes} value={draft.mode} label="Routing mode" onchange={(m) => draft && (draft.mode = m)} />
			<p class="mode-note">{modes.find((m) => m.value === draft!.mode)?.text}</p>
		</div>

		<div class="section">
			<div class="section-head"><span class="caption">General</span></div>
			<div class="group">
				<label class="item">
					<span class="grow">Bypass local network<span class="sub">Keep printers, routers and LAN devices reachable.</span></span>
					<input type="checkbox" bind:checked={draft.bypassLan} />
				</label>
				{#if draft.mode === 'rule'}
					<div class="item">
						<span class="grow">Unmatched traffic<span class="sub">Used when no rule matches.</span></span>
						<Select bind:value={draft.finalAction} label="Unmatched traffic" options={[{ value: 'proxy', label: 'Through the VPN' }, { value: 'direct', label: 'Direct' }]} />
					</div>
				{/if}
				<div class="item">
					<span class="grow">DNS resolver<span class="sub">{draft.dns.provider === 'system' ? 'Queries may reveal visited sites to your ISP.' : 'Queries go through the tunnel.'}</span></span>
					<Select bind:value={draft.dns.provider} label="Resolver" options={providers} />
				</div>
				{#if draft.dns.provider === 'custom'}
					<div class="item"><input class="input grow" bind:value={draft.dns.custom} placeholder="dns.example.com or https://dns.example.com/your-id" aria-label="Custom DNS server" spellcheck="false" /></div>
				{/if}
			</div>
		</div>

		{#if draft.mode === 'rule'}
			<div class="section">
				<div class="section-head">
					<span class="caption">Rules · first match wins</span>
					<span class="links">
						<button type="button" class="linkbtn" onclick={() => applyPreset('streaming')}>+ Streaming direct</button>
						<button type="button" class="linkbtn" onclick={() => applyPreset('ai')}>+ AI via VPN</button>
						<button type="button" class="linkbtn" onclick={() => applyPreset('ads')}>+ Block ads</button>
					</span>
				</div>
				<div class="group">
					{#each draft.rules as rule, i (rule.id)}
						<div class="item rule" class:off={!rule.enabled}>
							<input type="checkbox" bind:checked={rule.enabled} aria-label="Enable rule" />
							<span class="type">{kindLabel(rule.kind)}</span>
							<span class="grow mono val" title={rule.value}>{rule.value}</span>
							<Select bind:value={rule.action} label="Action" options={actions} />
							<span class="tools">
								<button class="icon-btn" onclick={() => move(i, -1)} disabled={i === 0} aria-label="Move up">↑</button>
								<button class="icon-btn" onclick={() => move(i, 1)} disabled={i === draft.rules.length - 1} aria-label="Move down">↓</button>
								<button class="icon-btn del" onclick={() => draft?.rules.splice(i, 1)} aria-label="Delete rule">✕</button>
							</span>
						</div>
					{:else}
						<div class="item empty"><span class="grow">No rules. Add one below or use a preset above.</span></div>
					{/each}
					<form class="item add" onsubmit={(e) => (e.preventDefault(), addRule())}>
						<Select bind:value={kind} label="Rule type" options={kinds} />
						<input class="input grow" bind:value placeholder={kind === 'ipCidr' ? '10.0.0.0/8' : kind === 'processName' ? 'Slack' : 'example.com'} aria-label="Rule value" spellcheck="false" />
						<Select bind:value={action} label="Action" options={actions} />
						<button class="btn" disabled={!value.trim()}>Add</button>
					</form>
				</div>
			</div>

			<div class="section">
				<div class="section-head">
					<span class="caption">Country lists</span>
					<span class="links">
						{#each ['cn', 'ru', 'ir', 'lk', 'us', 'hk', 'jp'] as code (code)}
							<button type="button" class="linkbtn mono" onclick={() => (country = code)}>{code.toUpperCase()}</button>
						{/each}
					</span>
				</div>
				<div class="group">
					{#each draft.ruleSets as set, i (set.id)}
						<div class="item rule" class:off={!set.enabled}>
							<input type="checkbox" bind:checked={set.enabled} aria-label="Enable list" />
							<span class="type">{set.kind === 'geoip' ? 'IP list' : 'Site list'}</span>
							<span class="grow mono val">{set.country.toUpperCase()}</span>
							<Select bind:value={set.action} label="Action" options={actions} />
							<span class="tools"><button class="icon-btn del" onclick={() => draft?.ruleSets.splice(i, 1)} aria-label="Delete list">✕</button></span>
						</div>
					{/each}
					<form class="item add" onsubmit={(e) => (e.preventDefault(), addSet())}>
						<Select bind:value={setKind} label="List type" options={[{ value: 'geoip', label: 'IP list' }, { value: 'geosite', label: 'Site list' }]} />
						<input class="input grow" bind:value={country} placeholder="Country code, e.g. cn" aria-label="Country or list" maxlength="32" />
						<Select bind:value={setAction} label="Action" options={actions} />
						<button class="btn" disabled={!country.trim()}>Add</button>
					</form>
				</div>
				<p class="foot">Downloaded by sing-box when it connects; needs access to GitHub.</p>
			</div>

			<div class="section">
				<div class="section-head"><span class="caption">Test a domain</span></div>
				<div class="group">
					<div class="item">
						<input class="input grow" bind:value={probe} placeholder="example.com" aria-label="Domain to test" spellcheck="false" />
						<span class="verdict" class:idle={!verdict}>{verdict ?? 'Type a domain to see which rule applies.'}</span>
					</div>
				</div>
			</div>
		{/if}

		{#if dirty}
			<div class="savebar" role="status">
				<span>Unsaved changes. They apply the next time you connect.</span>
				<button class="btn" onclick={() => (draft = structuredClone($state.snapshot(app.routing!)))}>Revert</button>
				<button class="btn primary" onclick={() => draft && app.saveRouting($state.snapshot(draft) as RoutingSettings)}>Save</button>
			</div>
		{/if}
	</div>
{/if}

<style>
	.page {
		display: grid;
		gap: 18px;
		align-content: start;
		container-type: inline-size;
	}
	p {
		margin: 0;
	}
	.mode-note,
	.foot {
		padding: 0 4px;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.rule.off .type,
	.rule.off .val {
		opacity: 0.45;
	}
	.type {
		width: 104px;
		flex: none;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.val {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		user-select: text;
	}
	.tools {
		display: flex;
		gap: 0;
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
	.empty {
		color: var(--fg-muted);
	}
	.add {
		flex-wrap: wrap;
		background: color-mix(in srgb, var(--hairline) 35%, transparent);
	}
	.add .grow {
		min-width: 120px;
	}
	.verdict {
		font-size: 12px;
		font-weight: 600;
	}
	.verdict.idle {
		font-weight: 400;
		color: var(--fg-muted);
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
	@container (max-width: 520px) {
		.type {
			display: none;
		}
		.tools {
			opacity: 1;
		}
	}
</style>
