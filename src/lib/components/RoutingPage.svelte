<script lang="ts">
	import Select from './Select.svelte';
	import type { DnsProvider, RouteAction, RouteMode, RoutingSettings, RuleKind, RuleSetKind } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import Icon from './Icon.svelte';

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
	const modes: { value: RouteMode; title: string; text: string; icon: 'shield' | 'routing' | 'bolt' }[] = [
		{ value: 'global', title: 'Global', text: 'Everything goes through the VPN.', icon: 'shield' },
		{ value: 'rule', title: 'Rules', text: 'Use your rules; the rest follows the default.', icon: 'routing' },
		{ value: 'direct', title: 'Direct', text: 'VPN stays on but nothing is routed. For testing.', icon: 'bolt' }
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
		{ value: 'custom', label: 'Custom DoH server' },
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
		<div class="modes" role="radiogroup" aria-label="Routing mode">
			{#each modes as mode (mode.value)}
				<button
					class="card mode"
					class:active={draft.mode === mode.value}
					role="radio"
					aria-checked={draft.mode === mode.value}
					onclick={() => draft && (draft.mode = mode.value)}
				>
					<span class="mode-icon"><Icon name={mode.icon} size={18} /></span>
					<strong>{mode.title}</strong>
					<small>{mode.text}</small>
				</button>
			{/each}
		</div>

		<section class="card list">
			<label class="row">
				<span class="text"><strong>Bypass local network</strong><small>Keep printers, routers and other LAN devices reachable.</small></span>
				<input type="checkbox" bind:checked={draft.bypassLan} />
			</label>
			{#if draft.mode === 'rule'}
				<label class="row">
					<span class="text"><strong>Everything else goes through</strong><small>Used when no rule matches.</small></span>
					<Select
						bind:value={draft.finalAction}
						label="Everything else goes through"
						options={[
							{ value: 'proxy', label: 'The VPN' },
							{ value: 'direct', label: 'Direct' }
						]}
					/>
				</label>
			{/if}
			<label class="row">
				<span class="text"><strong>DNS resolver</strong><small>Queries go through the tunnel unless you pick the system resolver.</small></span>
				<Select bind:value={draft.dns.provider} label="Resolver" options={providers} />
			</label>
			{#if draft.dns.provider === 'custom'}
				<div class="row"><input class="input grow" bind:value={draft.dns.custom} placeholder="dns.example.com" aria-label="Custom DoH server" spellcheck="false" /></div>
			{/if}
			{#if draft.dns.provider === 'system'}
				<div class="row"><small class="warn">The system resolver can reveal the sites you visit to your ISP.</small></div>
			{/if}
		</section>

		{#if draft.mode === 'rule'}
			<section class="card list">
				<div class="section-head">
					<div><strong>Rules</strong><small>First match wins · {draft.rules.length} rule{draft.rules.length === 1 ? '' : 's'}</small></div>
					<div class="chips" aria-label="Quick presets">
						<button type="button" class="chip" onclick={() => applyPreset('streaming')}>+ Streaming direct</button>
						<button type="button" class="chip" onclick={() => applyPreset('ai')}>+ AI via VPN</button>
						<button type="button" class="chip" onclick={() => applyPreset('ads')}>+ Block ads</button>
					</div>
				</div>
				{#each draft.rules as rule, i (rule.id)}
					<div class="rule" class:off={!rule.enabled}>
						<input type="checkbox" bind:checked={rule.enabled} aria-label="Enable rule" />
						<span class="rule-text"><span class="mono val">{rule.value}</span><small>{kindLabel(rule.kind)}</small></span>
						<Select bind:value={rule.action} label="Action" options={actions} />
						<span class="tools">
							<button class="icon-btn" onclick={() => move(i, -1)} disabled={i === 0} aria-label="Move up">↑</button>
							<button class="icon-btn" onclick={() => move(i, 1)} disabled={i === draft.rules.length - 1} aria-label="Move down">↓</button>
							<button class="icon-btn danger" onclick={() => draft?.rules.splice(i, 1)} aria-label="Delete rule">✕</button>
						</span>
					</div>
				{:else}
					<div class="empty"><strong>No rules yet</strong><small>Add one below, or start from a preset above.</small></div>
				{/each}
				<form class="add" onsubmit={(e) => (e.preventDefault(), addRule())}>
					<Select bind:value={kind} label="Rule type" options={kinds} />
					<input class="input grow" bind:value placeholder={kind === 'ipCidr' ? '10.0.0.0/8' : kind === 'processName' ? 'Slack' : 'example.com'} aria-label="Rule value" spellcheck="false" />
					<Select bind:value={action} label="Action" options={actions} />
					<button class="btn primary" disabled={!value.trim()}>Add rule</button>
				</form>
			</section>

			<section class="card list">
				<div class="section-head">
					<div><strong>Country lists</strong><small>sing-box downloads these when it connects (needs access to GitHub).</small></div>
					<div class="chips" aria-label="Popular lists">
						{#each ['cn', 'ru', 'ir', 'lk', 'us', 'hk', 'jp'] as code (code)}
							<button type="button" class="chip" onclick={() => (country = code)}>{code.toUpperCase()}</button>
						{/each}
					</div>
				</div>
				{#each draft.ruleSets as set, i (set.id)}
					<div class="rule" class:off={!set.enabled}>
						<input type="checkbox" bind:checked={set.enabled} aria-label="Enable list" />
						<span class="rule-text"><span class="mono val">{set.country.toUpperCase()}</span><small>{set.kind === 'geoip' ? 'IP list' : 'Site list'}</small></span>
						<Select bind:value={set.action} label="Action" options={actions} />
						<span class="tools"><button class="icon-btn danger" onclick={() => draft?.ruleSets.splice(i, 1)} aria-label="Delete list">✕</button></span>
					</div>
				{/each}
				<form class="add" onsubmit={(e) => (e.preventDefault(), addSet())}>
					<Select bind:value={setKind} label="List type" options={[{ value: 'geoip', label: 'IP list' }, { value: 'geosite', label: 'Site list' }]} />
					<input class="input grow" bind:value={country} placeholder="Country code, e.g. cn, ru, lk" aria-label="Country or list" maxlength="32" />
					<Select bind:value={setAction} label="Action" options={actions} />
					<button class="btn primary" disabled={!country.trim()}>Add list</button>
				</form>
			</section>

			<section class="card list">
				<div class="section-head"><div><strong>Test a domain</strong><small>See which rule would handle it.</small></div></div>
				<div class="row">
					<input class="input grow" bind:value={probe} placeholder="example.com" aria-label="Domain to test" spellcheck="false" />
					{#if verdict}<span class="verdict">{verdict}</span>{/if}
				</div>
			</section>
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
		gap: 12px;
		align-content: start;
		padding-bottom: 8px;
	}
	small {
		color: var(--fg-muted);
	}
	.warn {
		color: var(--warn);
	}
	.modes {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 10px;
	}
	.mode {
		text-align: left;
		padding: 14px;
		display: grid;
		gap: 3px;
		color: var(--fg);
		transition: border-color 0.15s, background 0.15s;
	}
	.mode:hover {
		border-color: color-mix(in srgb, var(--accent) 50%, var(--hairline));
	}
	.mode.active {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 10%, var(--surface));
	}
	.mode-icon {
		display: grid;
		place-items: center;
		width: 30px;
		height: 30px;
		border-radius: 8px;
		margin-bottom: 6px;
		background: var(--hairline);
		color: var(--fg-muted);
	}
	.mode.active .mode-icon {
		background: var(--accent);
		color: white;
	}
	.list {
		display: grid;
	}
	.row,
	.rule,
	.add,
	.empty {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 16px;
	}
	.row + .row,
	.section-head + .rule,
	.rule + .rule,
	.rule + .add,
	.empty,
	.section-head + .row {
		border-top: 1px solid var(--hairline);
	}
	.add {
		border-top: 1px solid var(--hairline);
		flex-wrap: wrap;
		background: color-mix(in srgb, var(--hairline) 40%, transparent);
		border-radius: 0 0 var(--radius) var(--radius);
	}
	.row {
		justify-content: space-between;
	}
	.text {
		display: grid;
		min-width: 0;
	}
	.section-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 8px 12px;
		padding: 12px 16px;
	}
	.section-head div:first-child {
		display: grid;
	}
	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.chip {
		border: 1px solid var(--hairline);
		background: var(--surface);
		border-radius: 999px;
		padding: 3px 10px;
		font-size: 11px;
		font-weight: 600;
		color: var(--fg-muted);
	}
	.chip:hover {
		border-color: var(--accent);
		color: var(--accent);
	}
	.rule.off .rule-text {
		opacity: 0.45;
	}
	.rule-text {
		flex: 1;
		min-width: 0;
		display: grid;
	}
	.val {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		user-select: text;
	}
	.tools {
		display: flex;
		gap: 2px;
	}
	.icon-btn {
		min-width: 28px;
		min-height: 28px;
		border: 0;
		background: none;
		border-radius: 6px;
		color: var(--fg-muted);
	}
	.icon-btn:hover:not(:disabled) {
		background: var(--hairline);
		color: var(--fg);
	}
	.icon-btn.danger:hover:not(:disabled) {
		color: var(--danger);
	}
	.empty {
		flex-direction: column;
		padding: 22px 16px;
		gap: 2px;
		text-align: center;
	}
	.grow {
		flex: 1;
		min-width: 120px;
	}
	.verdict {
		font-weight: 600;
		font-size: 12px;
	}
	.savebar {
		position: sticky;
		bottom: 0;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-radius: var(--radius);
		border: 1px solid var(--accent);
		background: var(--surface-strong);
		box-shadow: 0 8px 24px rgb(0 0 0 / 0.25);
	}
	.savebar span {
		flex: 1;
	}
</style>
