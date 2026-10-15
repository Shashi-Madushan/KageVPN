<script lang="ts">
	import { api, type ListStatus, type RuleProvider, type RoutingSettings, type RuleSetsStatus } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { formatBytes } from '$lib/format';

	let { draft, dirty }: { draft: RoutingSettings; dirty: boolean } = $props();

	let builtin = $state<RuleProvider[]>([]);
	let status = $state<RuleSetsStatus | null>(null);
	let busy = $state<string | null>(null);
	let errors = $state<Record<string, string>>({});
	let name = $state('');
	let geoip = $state('');
	let geosite = $state('');

	const providers = $derived([...builtin, ...draft.customProviders]);
	const saved = $derived(providers.find((p) => p.id === app.routing?.ruleProvider));
	const sourceChanged = $derived(!!status?.downloadedFrom && status.downloadedFrom !== app.routing?.ruleProvider);
	const canAdd = $derived(name.trim() !== '' && (geoip.trim() !== '' || geosite.trim() !== ''));

	async function reload() {
		status = await api.ruleSetsStatus().catch(() => null);
	}

	$effect(() => {
		api.ruleProviders().then((list) => (builtin = list)).catch(() => null);
		reload();
	});

	// The saved lists change when settings are saved; look again.
	$effect(() => {
		void app.routing;
		reload();
	});

	function provides(p: RuleProvider) {
		if (p.geoip && p.geosite) return 'IP lists and site lists';
		return p.geoip ? 'IP lists only' : 'Site lists only';
	}

	function addProvider() {
		if (!canAdd) return;
		const id = crypto.randomUUID();
		draft.customProviders.push({ id, name: name.trim(), geoip: geoip.trim(), geosite: geosite.trim(), note: '' });
		draft.ruleProvider = id;
		name = geoip = geosite = '';
	}

	function removeProvider(id: string) {
		draft.customProviders = draft.customProviders.filter((p) => p.id !== id);
		if (draft.ruleProvider === id) draft.ruleProvider = 'sagernet';
	}

	function age(modified: number | null) {
		if (modified === null) return '';
		const days = Math.floor((Date.now() / 1000 - modified) / 86400);
		return days <= 0 ? 'today' : days === 1 ? 'yesterday' : `${days} days ago`;
	}

	function label(list: ListStatus) {
		return `${list.kind === 'geoip' ? 'IP' : 'Site'} · ${list.name.toUpperCase()}`;
	}

	async function download(tag: string | null) {
		busy = tag ?? 'all';
		try {
			const results = await api.downloadRuleSets(tag);
			for (const r of results) {
				if (r.error) errors[r.tag] = r.error;
				else delete errors[r.tag];
			}
			const failed = results.filter((r) => r.error).length;
			if (failed) app.notify(`${failed} list${failed === 1 ? '' : 's'} could not be downloaded.`);
			else app.notify(results.length === 1 ? 'List downloaded. Reconnect to apply it.' : 'Lists downloaded. Reconnect to apply them.', 'info');
		} catch (e) {
			app.notify(String(e));
		} finally {
			busy = null;
			await reload();
		}
	}

	async function remove(tag: string | null) {
		busy = tag ?? 'all';
		try {
			await api.deleteRuleSets(tag);
		} catch (e) {
			app.notify(String(e));
		} finally {
			busy = null;
			await reload();
		}
	}
</script>

<div class="section">
	<div class="section-head"><span class="caption">List sources</span></div>
	<div class="group">
		{#each providers as p (p.id)}
			<label class="item source">
				<input type="radio" name="rule-provider" value={p.id} checked={draft.ruleProvider === p.id} onchange={() => (draft.ruleProvider = p.id)} />
				<span class="grow info">
					<strong>{p.name}</strong>
					<small>{provides(p)}{p.note ? ` · ${p.note}` : ''}</small>
				</span>
				{#if !builtin.some((b) => b.id === p.id)}
					<button type="button" class="icon-btn del" onclick={() => removeProvider(p.id)} aria-label="Remove source {p.name}">✕</button>
				{/if}
			</label>
		{/each}
		<form class="item add" onsubmit={(e) => (e.preventDefault(), addProvider())}>
			<input class="input" bind:value={name} placeholder="Name" aria-label="Source name" maxlength="40" />
			<input class="input grow" bind:value={geoip} placeholder="IP list address, https://…/geoip-&#123;name&#125;.srs" aria-label="IP list address" spellcheck="false" />
			<input class="input grow" bind:value={geosite} placeholder="Site list address (optional)" aria-label="Site list address" spellcheck="false" />
			<button class="btn" disabled={!canAdd}>Add source</button>
		</form>
	</div>
	<p class="foot">
		Pick where lists are downloaded from. Every source must publish sing-box rule-set (<span class="mono">.srs</span>) files; use
		<span class="mono">&#123;name&#125;</span> where the country or category goes. GitHub addresses also get a jsDelivr mirror automatically.
	</p>
</div>

<div class="section">
	<div class="section-head">
		<span class="caption">Downloaded lists</span>
		<span class="links">
			<button type="button" class="linkbtn" disabled={dirty || busy !== null || !status?.lists.length} onclick={() => download(null)}>
				{busy === 'all' ? 'Downloading…' : 'Download all'}
			</button>
			<button type="button" class="linkbtn" disabled={dirty || busy !== null || !status?.lists.some((l) => l.present)} onclick={() => remove(null)}>Remove all</button>
		</span>
	</div>
	<div class="group">
		{#each status?.lists ?? [] as list (list.tag)}
			<div class="item">
				<span class="grow mono val">{label(list)}</span>
				<span class="state" class:missing={!list.present}>
					{#if errors[list.tag]}
						<span class="bad" title={errors[list.tag]}>Failed</span>
					{:else if list.present}
						{formatBytes(list.bytes)} · {age(list.modified)}
					{:else}
						Not downloaded
					{/if}
				</span>
				<button type="button" class="btn" disabled={dirty || busy !== null} onclick={() => download(list.tag)}>{busy === list.tag ? '…' : list.present ? 'Update' : 'Download'}</button>
				<button type="button" class="icon-btn del" disabled={dirty || busy !== null || !list.present} onclick={() => remove(list.tag)} aria-label="Remove {label(list)}">✕</button>
			</div>
		{:else}
			<div class="item empty"><span class="grow">No lists yet. Add a country list above and save.</span></div>
		{/each}
	</div>
	<p class="foot">
		{#if dirty}
			Save your changes to download or remove lists.
		{:else if sourceChanged}
			These files came from another source. Download all to switch to {saved?.name ?? 'the selected one'}.
		{:else}
			{status?.throughTunnel ? 'Connected: downloads go through the VPN first, then directly.' : 'Not connected: downloads go directly, then through the VPN once you are connected.'}
			New or updated lists apply the next time you connect.
		{/if}
	</p>
</div>

<style>
	p {
		margin: 0;
	}
	.foot {
		padding: 0 4px;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.source {
		cursor: pointer;
	}
	.info {
		display: grid;
		gap: 2px;
		min-width: 0;
	}
	small {
		color: var(--fg-muted);
		overflow-wrap: anywhere;
	}
	.val {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.state {
		font-size: 12px;
		color: var(--fg-muted);
		white-space: nowrap;
	}
	.state.missing {
		opacity: 0.7;
	}
	.bad {
		color: var(--danger);
		font-weight: 600;
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
	.icon-btn.del:hover:not(:disabled) {
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
		min-width: 160px;
	}
</style>
