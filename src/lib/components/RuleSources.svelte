<script lang="ts">
	import { api, type ListStatus, type RuleProvider, type RoutingSettings } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { formatBytes } from '$lib/format';
	import { describeList } from '$lib/ruleCatalog';
	import { ruleLists } from '$lib/ruleLists.svelte';

	let { draft, dirty }: { draft: RoutingSettings; dirty: boolean } = $props();

	let builtin = $state<RuleProvider[]>([]);
	let busy = $state<string | null>(null);
	let errors = $state<Record<string, string>>({});
	let adding = $state(false);
	let name = $state('');
	let geoip = $state('');
	let geosite = $state('');

	const status = $derived(ruleLists.status);
	const providers = $derived([...builtin, ...draft.customProviders]);
	const saved = $derived(providers.find((p) => p.id === app.routing?.ruleProvider));
	const sourceChanged = $derived(!!status?.downloadedFrom && status.downloadedFrom !== app.routing?.ruleProvider);
	const canAdd = $derived(name.trim() !== '' && (geoip.trim() !== '' || geosite.trim() !== ''));
	const ready = $derived(status?.lists.filter((l) => l.present).length ?? 0);
	const total = $derived(status?.lists.length ?? 0);

	$effect(() => {
		api.ruleProviders().then((list) => (builtin = list)).catch(() => null);
	});

	function provides(p: RuleProvider) {
		if (p.geoip && p.geosite) return 'Countries and websites';
		return p.geoip ? 'Countries only' : 'Websites only';
	}

	function addProvider() {
		if (!canAdd) return;
		const id = crypto.randomUUID();
		draft.customProviders.push({ id, name: name.trim(), geoip: geoip.trim(), geosite: geosite.trim(), note: '' });
		draft.ruleProvider = id;
		name = geoip = geosite = '';
		adding = false;
	}

	function removeProvider(id: string) {
		draft.customProviders = draft.customProviders.filter((p) => p.id !== id);
		if (draft.ruleProvider === id) draft.ruleProvider = 'sagernet';
	}

	function age(modified: number | null) {
		if (modified === null) return '';
		const days = Math.floor((Date.now() / 1000 - modified) / 86400);
		return days <= 0 ? 'updated today' : days === 1 ? 'updated yesterday' : `updated ${days} days ago`;
	}

	const title = (list: ListStatus) => describeList(list.kind, list.name).title;

	async function download(tag: string | null) {
		busy = tag ?? 'all';
		try {
			const results = await api.downloadRuleSets(tag);
			for (const r of results) {
				if (r.error) errors[r.tag] = r.error;
				else delete errors[r.tag];
			}
			const failed = results.filter((r) => r.error).length;
			if (failed) app.notify(`${failed} list${failed === 1 ? '' : 's'} could not be downloaded. Try another source.`);
			else app.notify(results.length === 1 ? 'List downloaded. Reconnect to apply it.' : 'Lists downloaded. Reconnect to apply them.', 'info');
		} catch (e) {
			app.notify(String(e));
		} finally {
			busy = null;
			await ruleLists.reload();
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
			await ruleLists.reload();
		}
	}
</script>

<details class="advanced">
	<summary>
		<span class="s-title">Downloads and sources</span>
		<span class="s-sub">
			{saved?.name ?? 'SagerNet'}{total ? ` · ${ready} of ${total} ready` : ''}
		</span>
	</summary>

	<div class="body">
		<div class="block">
			<h4>Where lists come from</h4>
			<p>All sources work the same way. If one is slow or blocked where you are, pick another.</p>
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
			</div>

			{#if adding}
				<form class="adder" onsubmit={(e) => (e.preventDefault(), addProvider())}>
					<h4>Add your own source</h4>
					<label class="field">
						<span class="field-label">Name</span>
						<input class="input" bind:value={name} placeholder="My lists" maxlength="40" />
					</label>
					<label class="field">
						<span class="field-label">Address for country lists</span>
						<input class="input" bind:value={geoip} placeholder="https://example.com/geoip-&#123;name&#125;.srs" spellcheck="false" />
					</label>
					<label class="field">
						<span class="field-label">Address for website lists <em>(optional)</em></span>
						<input class="input" bind:value={geosite} placeholder="https://example.com/geosite-&#123;name&#125;.srs" spellcheck="false" />
					</label>
					<small>Use <span class="mono">&#123;name&#125;</span> where the list name goes, for example <span class="mono">cn</span>. The files must be sing-box rule lists (<span class="mono">.srs</span>).</small>
					<div class="buttons">
						<button type="button" class="btn" onclick={() => (adding = false)}>Cancel</button>
						<button class="btn primary" disabled={!canAdd}>Add source</button>
					</div>
				</form>
			{:else}
				<button type="button" class="btn start" onclick={() => (adding = true)}>+ Add your own source</button>
			{/if}
		</div>

		<div class="block">
			<div class="line">
				<h4>Downloaded lists</h4>
				<span class="links">
					<button type="button" class="linkbtn" disabled={dirty || busy !== null || !total} onclick={() => download(null)}>{busy === 'all' ? 'Downloading…' : 'Download all'}</button>
					<button type="button" class="linkbtn" disabled={dirty || busy !== null || !ready} onclick={() => remove(null)}>Remove all</button>
				</span>
			</div>
			<div class="group">
				{#each status?.lists ?? [] as list (list.tag)}
					<div class="item">
						<span class="grow info">
							<strong>{title(list)}</strong>
							<small>{list.kind === 'geoip' ? 'Country' : 'Websites'}</small>
						</span>
						<span class="state" class:missing={!list.present}>
							{#if errors[list.tag]}
								<span class="bad" title={errors[list.tag]}>Could not download</span>
							{:else if list.present}
								{formatBytes(list.bytes)} · {age(list.modified)}
							{:else}
								Not downloaded
							{/if}
						</span>
						<button type="button" class="btn" disabled={dirty || busy !== null} onclick={() => download(list.tag)}>{busy === list.tag ? 'Working…' : list.present ? 'Update' : 'Download'}</button>
						<button type="button" class="icon-btn del" disabled={dirty || busy !== null || !list.present} onclick={() => remove(list.tag)} aria-label="Delete the downloaded file for {title(list)}">✕</button>
					</div>
				{:else}
					<div class="item empty"><span class="grow">Nothing to download yet. Add a list above and save.</span></div>
				{/each}
			</div>
			<p class="foot">
				{#if dirty}
					Save your changes first, then you can download or delete lists here.
				{:else if sourceChanged}
					These files came from another source. Choose “Download all” to switch to {saved?.name ?? 'the selected one'}.
				{:else}
					{status?.throughTunnel ? 'Connected: lists are fetched through the VPN first, then directly.' : 'Lists are fetched directly now, and through the VPN when you are connected.'}
					Updated lists apply the next time you connect. Lists refresh by themselves every week.
				{/if}
			</p>
		</div>
	</div>
</details>

<style>
	p,
	h4 {
		margin: 0;
	}
	h4 {
		font-size: 13px;
		font-weight: 650;
	}
	p,
	small,
	.foot {
		font-size: 12px;
		color: var(--fg-muted);
	}
	.advanced {
		border: 1px solid var(--hairline);
		border-radius: 12px;
		background: var(--surface);
	}
	summary {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		padding: 12px 14px;
		cursor: pointer;
		list-style: none;
	}
	summary::-webkit-details-marker {
		display: none;
	}
	summary::before {
		content: '›';
		display: inline-block;
		width: 12px;
		margin-right: 4px;
		color: var(--fg-muted);
		transition: transform 0.15s;
	}
	.advanced[open] summary::before {
		transform: rotate(90deg);
	}
	.s-title {
		flex: 1;
		font-weight: 600;
	}
	.s-sub {
		font-size: 12px;
		color: var(--fg-muted);
	}
	.body {
		display: grid;
		gap: 20px;
		padding: 4px 14px 14px;
		border-top: 1px solid var(--hairline);
		padding-top: 14px;
	}
	.block {
		display: grid;
		gap: 8px;
	}
	.line {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}
	.links {
		display: flex;
		gap: 12px;
	}
	.linkbtn:disabled {
		opacity: 0.45;
		text-decoration: none;
	}
	.source {
		cursor: pointer;
	}
	.info {
		display: grid;
		gap: 1px;
		min-width: 0;
	}
	small {
		overflow-wrap: anywhere;
	}
	.state {
		font-size: 12px;
		color: var(--fg-muted);
		white-space: nowrap;
	}
	.state.missing {
		opacity: 0.75;
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
	.start {
		justify-self: start;
	}
	.adder {
		display: grid;
		gap: 10px;
		padding: 14px;
		border: 1px solid var(--hairline);
		border-radius: 10px;
		background: color-mix(in srgb, var(--hairline) 25%, transparent);
	}
	.field {
		display: grid;
		gap: 5px;
	}
	.field-label {
		font-size: 11.5px;
		font-weight: 600;
		color: var(--fg);
	}
	.field-label em {
		font-weight: 400;
		font-style: normal;
		color: var(--fg-muted);
	}
	.buttons {
		display: flex;
		gap: 8px;
		justify-content: flex-end;
	}
</style>
