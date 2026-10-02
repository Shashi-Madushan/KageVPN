<script lang="ts">
	import type { DnsProvider, RouteAction, RouteMode, RoutingSettings, RuleKind, RuleSetKind } from '$lib/api';
	import { app } from '$lib/app.svelte';

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
	const modes: { value: RouteMode; title: string; text: string }[] = [
		{ value: 'global', title: 'Global', text: 'Everything goes through the VPN.' },
		{ value: 'rule', title: 'Rules', text: 'Use your rules, send the rest to the default below.' },
		{ value: 'direct', title: 'Direct', text: 'VPN stays on but nothing is routed. For testing.' }
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
		<section class="modes">
			{#each modes as mode (mode.value)}
				<button class="card mode" class:active={draft.mode === mode.value} onclick={() => draft && (draft.mode = mode.value)}>
					<strong>{mode.title}</strong>
					<small>{mode.text}</small>
				</button>
			{/each}
		</section>

		<section class="card block">
			<label class="toggle">
				<span><strong>Bypass local network</strong><small>Keep printers, routers and other LAN devices reachable.</small></span>
				<input type="checkbox" bind:checked={draft.bypassLan} />
			</label>
			{#if draft.mode === 'rule'}
				<label class="toggle">
					<span><strong>Everything else goes through</strong><small>Used when no rule matches.</small></span>
					<select bind:value={draft.finalAction}>
						<option value="proxy">The VPN</option>
						<option value="direct">Direct (only listed traffic uses the VPN)</option>
					</select>
				</label>
			{/if}
		</section>

		{#if draft.mode === 'rule'}
			<section class="card block">
				<span class="caption">Rules, first match wins</span>
				{#each draft.rules as rule, i (rule.id)}
					<div class="rule" class:off={!rule.enabled}>
						<input type="checkbox" bind:checked={rule.enabled} aria-label="Enable rule" />
						<span class="kind">{kindLabel(rule.kind)}</span>
						<span class="mono val">{rule.value}</span>
						<select bind:value={rule.action} aria-label="Action">
							{#each actions as a (a.value)}<option value={a.value}>{a.label}</option>{/each}
						</select>
						<button onclick={() => move(i, -1)} disabled={i === 0} aria-label="Move up">↑</button>
						<button onclick={() => move(i, 1)} disabled={i === draft.rules.length - 1} aria-label="Move down">↓</button>
						<button class="danger" onclick={() => draft?.rules.splice(i, 1)} aria-label="Delete rule">✕</button>
					</div>
				{:else}
					<small class="muted">No rules yet.</small>
				{/each}
				<div class="presets-row">
					<span class="preset-label">Quick presets:</span>
					<button type="button" class="preset-btn" onclick={() => applyPreset('streaming')}>📺 Bypass Streaming</button>
					<button type="button" class="preset-btn" onclick={() => applyPreset('ai')}>🤖 Force AI Services</button>
					<button type="button" class="preset-btn" onclick={() => applyPreset('ads')}>🚫 Block Common Ads</button>
				</div>
				<form class="add" onsubmit={(e) => (e.preventDefault(), addRule())}>
					<select bind:value={kind}>{#each kinds as k (k.value)}<option value={k.value}>{k.label}</option>{/each}</select>
					<input class="grow" bind:value placeholder={kind === 'ipCidr' ? '10.0.0.0/8' : kind === 'processName' ? 'Slack' : 'example.com'} spellcheck="false" />
					<select bind:value={action}>{#each actions as a (a.value)}<option value={a.value}>{a.label}</option>{/each}</select>
					<button class="primary" disabled={!value.trim()}>Add</button>
				</form>
			</section>

			<section class="card block">
				<span class="caption">Country lists</span>
				<small class="muted">sing-box downloads these lists when it connects. Needs internet access to GitHub at that moment.</small>
				{#each draft.ruleSets as set, i (set.id)}
					<div class="rule" class:off={!set.enabled}>
						<input type="checkbox" bind:checked={set.enabled} aria-label="Enable list" />
						<span class="kind">{set.kind === 'geoip' ? 'IP list' : 'Site list'}</span>
						<span class="mono val">{set.country}</span>
						<select bind:value={set.action} aria-label="Action">
							{#each actions as a (a.value)}<option value={a.value}>{a.label}</option>{/each}
						</select>
						<button class="danger" onclick={() => draft?.ruleSets.splice(i, 1)} aria-label="Delete list">✕</button>
					</div>
				{/each}
				<div class="quick-countries">
					<span class="preset-label">Popular lists:</span>
					{#each ['cn', 'ru', 'ir', 'lk', 'us', 'hk', 'jp'] as code (code)}
						<button type="button" class="preset-btn" onclick={() => (country = code)}>{code.toUpperCase()}</button>
					{/each}
				</div>
				<form class="add" onsubmit={(e) => (e.preventDefault(), addSet())}>
					<select bind:value={setKind}><option value="geoip">IP list (geoip)</option><option value="geosite">Site list (geosite)</option></select>
					<input class="grow" bind:value={country} placeholder="Country or list, e.g. cn, ru, lk" maxlength="32" />
					<select bind:value={setAction}>{#each actions as a (a.value)}<option value={a.value}>{a.label}</option>{/each}</select>
					<button class="primary" disabled={!country.trim()}>Add</button>
				</form>
			</section>

			<section class="card block">
				<span class="caption">Try a domain</span>
				<input bind:value={probe} placeholder="Which rule matches example.com?" spellcheck="false" />
				{#if verdict}<p class="verdict">{verdict}</p>{/if}
			</section>
		{/if}

		<section class="card block">
			<span class="caption">DNS</span>
			<label class="toggle">
				<span><strong>Resolver</strong><small>Queries go through the tunnel unless you pick the system resolver.</small></span>
				<select bind:value={draft.dns.provider}>
					{#each providers as p (p.value)}<option value={p.value}>{p.label}</option>{/each}
				</select>
			</label>
			{#if draft.dns.provider === 'custom'}
				<input bind:value={draft.dns.custom} placeholder="dns.example.com" spellcheck="false" />
			{/if}
			{#if draft.dns.provider === 'system'}<small class="warn">The system resolver can reveal the sites you visit to your ISP.</small>{/if}
		</section>

		<div class="savebar" class:show={dirty}>
			<span>Unsaved changes. They apply the next time you connect.</span>
			<button onclick={() => (draft = structuredClone($state.snapshot(app.routing!)))}>Revert</button>
			<button class="primary" onclick={() => draft && app.saveRouting($state.snapshot(draft) as RoutingSettings)}>Save</button>
		</div>
	</div>
{/if}

<style>
	.page {
		display: grid;
		gap: 14px;
		align-content: start;
		padding-bottom: 56px;
	}
	.modes {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 10px;
	}
	.mode {
		text-align: left;
		padding: 12px 14px;
		display: grid;
		gap: 3px;
		color: var(--fg);
	}
	.mode.active {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 10%, var(--surface));
	}
	.mode small,
	small {
		color: var(--fg-muted);
	}
	.block {
		padding: 14px 16px;
		display: grid;
		gap: 10px;
	}
	.toggle {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 16px;
	}
	.toggle span {
		display: grid;
	}
	input,
	select {
		padding: 5px 8px;
		border-radius: 8px;
		border: 1px solid var(--hairline);
		background: var(--surface);
		user-select: text;
		min-width: 0;
	}
	input[type='checkbox'] {
		padding: 0;
		width: 34px;
		height: 20px;
		appearance: none;
		border-radius: 10px;
		background: var(--hairline);
		position: relative;
		flex: none;
		transition: background 0.2s;
	}
	input[type='checkbox']::after {
		content: '';
		position: absolute;
		top: 2px;
		left: 2px;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: white;
		box-shadow: 0 1px 2px rgb(0 0 0 / 0.3);
		transition: transform 0.2s;
	}
	input[type='checkbox']:checked {
		background: var(--ok);
	}
	input[type='checkbox']:checked::after {
		transform: translateX(14px);
	}
	.rule {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 0;
		border-top: 1px solid var(--hairline);
	}
	.rule.off .kind,
	.rule.off .val {
		opacity: 0.45;
	}
	.kind {
		font-size: 11px;
		color: var(--fg-muted);
		min-width: 110px;
	}
	.val {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		user-select: text;
	}
	.add {
		display: flex;
		gap: 8px;
		padding-top: 10px;
		border-top: 1px solid var(--hairline);
	}
	.grow {
		flex: 1;
	}
	button {
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		border-radius: 8px;
		padding: 4px 10px;
	}
	button:disabled {
		opacity: 0.4;
	}
	button.primary {
		background: var(--accent);
		border-color: var(--accent);
		color: white;
		font-weight: 600;
	}
	button.danger {
		color: var(--danger);
	}
	.verdict {
		margin: 0;
		font-weight: 600;
	}
	.warn {
		color: var(--warn);
	}
	.muted {
		color: var(--fg-muted);
	}
	.savebar {
		position: sticky;
		bottom: 0;
		display: none;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-radius: var(--radius);
		border: 1px solid var(--accent);
		background: var(--surface-strong);
		box-shadow: 0 8px 24px rgb(0 0 0 / 0.25);
	}
	.savebar.show {
		display: flex;
	}
	.savebar span {
		flex: 1;
	}
	.presets-row,
	.quick-countries {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
		margin: 4px 0;
	}
	.preset-label {
		font-size: 11px;
		color: var(--fg-muted);
		font-weight: 600;
	}
	.preset-btn {
		font-size: 11px;
		padding: 3px 8px;
		border-radius: 6px;
		border: 1px solid var(--hairline);
		background: var(--surface);
		cursor: pointer;
	}
	.preset-btn:hover {
		border-color: var(--accent);
		color: var(--accent);
	}
</style>
