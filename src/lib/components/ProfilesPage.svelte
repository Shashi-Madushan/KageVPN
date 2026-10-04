<script lang="ts">
	import Select from './Select.svelte';
	import { api, type Profile, type Subscription } from '$lib/api';
	import { clickOutside, focusOnMount } from '$lib/actions';
	import { app } from '$lib/app.svelte';
	import { protocols } from '$lib/protocols.svelte';
	import Sensitive from './Sensitive.svelte';
	import { onboarding } from '$lib/onboarding.svelte';
	import { formatBytes, latencyLabel, latencyTone, profileFlag, stripFlag } from '$lib/format';
	import HeaderActions from './HeaderActions.svelte';
	import ConfirmDialog from './ConfirmDialog.svelte';
	import ImportDialog from './ImportDialog.svelte';
	import MiniSpark from './MiniSpark.svelte';
	import ProfileDialog from './ProfileDialog.svelte';
	import QrDialog from './QrDialog.svelte';

	let importing = $state(false);
	let importText = $state('');
	let editing = $state<Profile | null>(null);
	let sharing = $state<Profile | null>(null);
	let menuFor = $state<string | null>(null);
	let menuPos = $state({ top: 0, right: 0 });
	let renaming = $state<string | null>(null);
	let query = $state('');
	let compact = $state(typeof localStorage !== 'undefined' && localStorage.getItem('serverDensity') === 'compact');
	$effect(() => {
		try {
			localStorage.setItem('serverDensity', compact ? 'compact' : 'comfortable');
		} catch {}
	});

	function listKeydown(e: KeyboardEvent) {
		if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return;
		const picks = [...(e.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>('button.pick')];
		const i = picks.indexOf(document.activeElement as HTMLButtonElement);
		if (i < 0) return;
		e.preventDefault();
		picks[Math.max(0, Math.min(picks.length - 1, i + (e.key === 'ArrowDown' ? 1 : -1)))]?.focus();
	}
	let sortBy = $state<'default' | 'latency' | 'name'>('default');
	let deleteTarget = $state<Profile | null>(null);
	let deleteSubTarget = $state<Subscription | null>(null);
	let subName = $state('');
	let subUrl = $state('');
	let subError = $state<string | null>(null);
	let subBusy = $state(false);
	let showSubForm = $state(false);

	$effect(() => {
		if (onboarding.addServerRequested) {
			onboarding.addServerRequested = false;
			openImport();
		}
	});

	$effect(() => {
		if (app.pendingImport) {
			importText = app.pendingImport;
			app.pendingImport = null;
			importing = true;
		}
	});

	const filtered = $derived(
		app.profiles.filter((p) => {
			const q = query.trim().toLowerCase();
			return !q || p.name.toLowerCase().includes(q) || p.server.toLowerCase().includes(q) || (p.group ?? '').toLowerCase().includes(q);
		})
	);
	const sections = $derived.by(() => {
		const byGroup = new Map<string, Profile[]>();
		for (const profile of filtered) {
			const key = profile.group ?? '';
			byGroup.set(key, [...(byGroup.get(key) ?? []), profile]);
		}
		const getMs = (id: string) => {
			const l = app.latency[id];
			if (!l || 'pending' in l) return 999998;
			if ('failed' in l) return 999999;
			return l.ms;
		};

		return [...byGroup.entries()]
			.sort(([a], [b]) => (a === '' ? -1 : b === '' ? 1 : a.localeCompare(b)))
			.map(([group, items]) => {
				let sorted = [...items];
				if (sortBy === 'latency') {
					sorted.sort((a, b) => getMs(a.id) - getMs(b.id));
				} else if (sortBy === 'name') {
					sorted.sort((a, b) => a.name.localeCompare(b.name));
				} else {
					sorted.sort((a, b) => Number(b.favorite) - Number(a.favorite));
				}
				return { group, items: sorted };
			});
	});
	const testing = $derived(Object.values(app.latency).some((l) => 'pending' in l));

	async function copyLink(profile: Profile) {
		menuFor = null;
		try {
			await navigator.clipboard.writeText(await api.profileLink(profile.id));
			app.notify('Link copied.', 'info');
		} catch (e) {
			app.notify(String(e));
		}
	}

	function openImport() {
		importText = '';
		importing = true;
	}

	async function addSubscription() {
		subBusy = true;
		subError = await app.addSubscription(subName, subUrl);
		subBusy = false;
		if (!subError) {
			subName = '';
			subUrl = '';
			showSubForm = false;
		}
	}

	function ago(seconds: number): string {
		if (!seconds) return 'never';
		const diff = Math.max(0, Date.now() / 1000 - seconds);
		if (diff < 90) return 'just now';
		if (diff < 5400) return `${Math.round(diff / 60)} min ago`;
		if (diff < 129600) return `${Math.round(diff / 3600)} h ago`;
		return `${Math.round(diff / 86400)} d ago`;
	}

	function quota(sub: Subscription) {
		const info = sub.userInfo;
		if (!info || !info.total) return null;
		const used = info.upload + info.download;
		return { used, total: info.total, percent: Math.min(100, (used / info.total) * 100), expire: info.expire };
	}

	function commitRename(id: string, event: Event) {
		app.rename(id, (event.currentTarget as HTMLInputElement).value);
		renaming = null;
	}
</script>

<HeaderActions>
	<button class="btn" disabled={app.profiles.length === 0 || testing} onclick={() => app.testAll()}>{testing ? 'Testing…' : 'Test all'}</button>
	<button class="btn primary" onclick={openImport}>Add server</button>
</HeaderActions>

<div class="page">
	{#if app.profiles.length === 0}
		<div class="empty">
			<h2>No servers yet</h2>
			<p>Paste a server link (VLESS, Trojan, Shadowsocks or Hysteria2) to get started, or add a subscription from your provider.</p>
			<div class="empty-actions">
				<button class="btn primary" onclick={openImport}>Add server</button>
				<button class="btn" onclick={() => (showSubForm = true)}>Add subscription</button>
			</div>
		</div>
	{:else}
		<div class="toolbar">
			<input class="input" type="search" placeholder="Search servers" bind:value={query} aria-label="Search servers" />
			<Select
				bind:value={sortBy}
				label="Sort servers"
				options={[
					{ value: 'default', label: 'Sort: Default' },
					{ value: 'latency', label: 'Sort: Lowest ping' },
					{ value: 'name', label: 'Sort: Name' }
				]}
			/>
			<button class="btn" aria-pressed={compact} title="Toggle compact rows" onclick={() => (compact = !compact)}>{compact ? 'Comfortable' : 'Compact'}</button>
		</div>

		<div class="group">
			<label class="item">
				<span class="grow">Pick the fastest server automatically<span class="sub">Tests servers every few minutes and fails over if one stops working.</span></span>
				<input type="checkbox" checked={app.auto.enabled} onchange={(e) => app.setAuto({ ...app.auto, enabled: e.currentTarget.checked })} />
			</label>
			{#if app.auto.enabled && app.groups.length}
				<div class="item">
					<span class="grow">Only use servers from</span>
					<Select
						value={app.auto.group ?? ''}
						label="Only use servers from"
						options={[{ value: '', label: 'All servers' }, ...app.groups.map((g) => ({ value: g, label: g }))]}
						onchange={(g) => app.setAuto({ ...app.auto, group: g || null })}
					/>
				</div>
			{/if}
		</div>

		{#each sections as section (section.group)}
			<div class="section">
				<div class="section-head">
					<span class="caption">{section.group || (sections.length > 1 ? 'Ungrouped' : 'Servers')}</span>
					<span class="count tabular">{section.items.length}</span>
				</div>
				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<ul class="group list" class:compact onkeydown={listKeydown}>
					{#each section.items as profile (profile.id)}
						{@const selected = profile.id === app.activeId && !app.auto.enabled}
						<li class="item row" class:selected>
							<button class="pick grow" onclick={() => app.select(profile.id)} title={app.connected ? 'Switch to this server' : 'Use this server'} aria-pressed={selected}>
								<span class="radio" aria-hidden="true"></span>
								<span class="flag">{profileFlag(profile.name, profile.server)}</span>
								<span class="text">
									{#if renaming === profile.id}
										<input
											class="input rename"
											value={profile.name}
											use:focusOnMount
											onblur={(e) => commitRename(profile.id, e)}
											onkeydown={(e) => {
												if (e.key === 'Enter') e.currentTarget.blur();
												if (e.key === 'Escape') renaming = null;
											}}
											onclick={(e) => e.stopPropagation()}
										/>
									{:else}
										<strong ondblclick={() => (renaming = profile.id)}>{stripFlag(profile.name)}</strong>
									{/if}
									<small class="mono"><Sensitive>{profile.server}:{profile.port}</Sensitive></small>
								</span>
							</button>
							<span class="proto">{profile.summary ?? ''}</span>
							<span class="spark"><MiniSpark values={app.latencyHistory[profile.id] ?? []} /></span>
							<button class="ping" onclick={() => app.ping(profile.id)} title="Test latency">
								{#if latencyTone(app.latency[profile.id]) === 'idle' && !app.latency[profile.id]}
									<span class="muted">Test</span>
								{:else}
									<span class="state {latencyTone(app.latency[profile.id])}">{latencyLabel(app.latency[profile.id])}</span>
								{/if}
							</button>
							<button class="icon-btn star" class:on={profile.favorite} onclick={() => app.toggleFavorite(profile.id)} aria-label={profile.favorite ? 'Remove favorite' : 'Add favorite'} aria-pressed={profile.favorite}>
								{profile.favorite ? '★' : '☆'}
							</button>
							<span class="menu-wrap" use:clickOutside={() => menuFor === profile.id && (menuFor = null)}>
								<button
									class="icon-btn"
									onclick={(e) => {
										if (menuFor === profile.id) {
											menuFor = null;
										} else {
											const rect = e.currentTarget.getBoundingClientRect();
											const spaceBelow = window.innerHeight - rect.bottom;
											menuPos = {
												top: spaceBelow > 230 ? rect.bottom + 4 : Math.max(10, rect.top - 230),
												right: Math.max(10, window.innerWidth - rect.right)
											};
											menuFor = profile.id;
										}
									}}
									aria-label="More actions for {profile.name}"
									aria-haspopup="menu"
									aria-expanded={menuFor === profile.id}
								>
									⋯
								</button>
								{#if menuFor === profile.id}
									<div class="menu" role="menu" style="top: {menuPos.top}px; right: {menuPos.right}px;">
										<button role="menuitem" onclick={() => ((editing = profile), (menuFor = null))}>Edit…</button>
										<button role="menuitem" onclick={() => ((renaming = profile.id), (menuFor = null))}>Rename</button>
										<button role="menuitem" onclick={() => (app.duplicate(profile.id), (menuFor = null))}>Duplicate</button>
										<hr />
										<button role="menuitem" onclick={() => copyLink(profile)}>Copy link</button>
										<button role="menuitem" onclick={() => ((sharing = profile), (menuFor = null))}>Show QR code</button>
										<hr />
										<button role="menuitem" class="danger" onclick={() => ((deleteTarget = profile), (menuFor = null))}>Delete…</button>
									</div>
								{/if}
							</span>
						</li>
					{/each}
					{#if section.items.length === 0}<li class="item"><span class="muted">No matches.</span></li>{/if}
				</ul>
			</div>
		{/each}
		{#if query.trim() && sections.length === 0}
			<p class="no-match">No servers match “{query.trim()}”.</p>
		{/if}
	{/if}

	{#if app.profiles.length || app.subscriptions.length || showSubForm}
	<div class="section">
		<div class="section-head"><span class="caption">Subscriptions</span></div>
		<div class="group">
			{#each app.subscriptions as sub (sub.id)}
				{@const q = quota(sub)}
				<div class="item sub-row">
					<div class="sub-top">
						<span class="grow">
							<strong>{sub.name}</strong>
							<span class="sub">{app.profiles.filter((p) => p.subscriptionId === sub.id).length} servers · updated {ago(sub.lastUpdated)}</span>
						</span>
						<button class="btn small" onclick={() => app.refreshSubscription(sub.id)}>Refresh</button>
						<button class="btn small danger" onclick={() => (deleteSubTarget = sub)}>Remove</button>
					</div>
					{#if sub.lastError}<span class="error">{sub.lastError}</span>{/if}
					{#if q}
						<div class="quota">
							<div class="meter"><div style:width="{q.percent}%"></div></div>
							<span class="sub">{formatBytes(q.used)} of {formatBytes(q.total)}{q.expire ? ` · expires ${new Date(q.expire * 1000).toLocaleDateString()}` : ''}</span>
						</div>
					{/if}
				</div>
			{/each}
			{#if showSubForm}
				<form class="item sub-form" onsubmit={(e) => (e.preventDefault(), addSubscription())}>
					<input class="input name" placeholder="Name (optional)" bind:value={subName} aria-label="Subscription name" />
					<input class="input url mono" placeholder="https://…/subscription" bind:value={subUrl} aria-label="Subscription URL" spellcheck="false" use:focusOnMount />
					<button class="btn primary" disabled={!subUrl.trim() || subBusy}>{subBusy ? 'Adding…' : 'Add'}</button>
					<button type="button" class="btn" onclick={() => ((showSubForm = false), (subError = null))}>Cancel</button>
				</form>
				{#if subError}<div class="item"><span class="error">{subError}</span></div>{/if}
			{:else}
				<div class="item">
					<button class="linkbtn" onclick={() => (showSubForm = true)}>+ Add subscription…</button>
					{#if app.subscriptions.length === 0}<span class="sub">Get a list of servers from a URL and keep it up to date.</span>{/if}
				</div>
			{/if}
		</div>
	</div>
	{/if}
</div>

<ConfirmDialog
	open={deleteTarget !== null}
	title="Delete server?"
	message={`“${stripFlag(deleteTarget?.name ?? '')}” will be removed from this app. This cannot be undone.`}
	onconfirm={() => (deleteTarget && app.remove(deleteTarget.id), (deleteTarget = null))}
	onclose={() => (deleteTarget = null)}
/>
<ConfirmDialog
	open={deleteSubTarget !== null}
	title="Remove subscription?"
	message={`“${deleteSubTarget?.name ?? ''}” and all the servers that came from it will be removed.`}
	confirmLabel="Remove"
	onconfirm={() => (deleteSubTarget && app.removeSubscription(deleteSubTarget.id), (deleteSubTarget = null))}
	onclose={() => (deleteSubTarget = null)}
/>

<ImportDialog bind:open={importing} initial={importText} onmanual={async () => (editing = await api.newProfile(protocols.list[0]?.id ?? 'vless'))} />
<ProfileDialog profile={editing} onclose={() => (editing = null)} />
<QrDialog profile={sharing} onclose={() => (sharing = null)} />

<style>
	.page {
		container-type: inline-size;
		display: grid;
		gap: 18px;
		align-content: start;
	}
	.toolbar {
		display: flex;
		gap: 8px;
		align-items: center;
	}
	.toolbar .input {
		flex: 1;
		min-width: 0;
	}
	.count {
		font-size: 11px;
		color: var(--fg-muted);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.row {
		position: relative;
		gap: 8px;
		padding-block: 4px;
		padding-right: 8px;
	}
	.row.selected {
		background: color-mix(in srgb, var(--accent) 9%, transparent);
	}
	.row.selected::before {
		content: '';
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		width: 3px;
		background: var(--accent);
	}
	.list.compact .row {
		padding-block: 0;
	}
	.list.compact .proto,
	.list.compact .spark {
		display: none;
	}
	.pick {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 4px;
		border: 0;
		background: none;
		text-align: left;
	}
	.radio {
		flex: none;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		border: 2px solid var(--hairline);
	}
	.selected .radio {
		border-color: var(--accent);
		background: radial-gradient(var(--accent) 38%, transparent 42%);
	}
	.flag {
		flex: none;
		font-size: 16px;
	}
	.text {
		display: grid;
		min-width: 0;
	}
	.text strong {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-weight: 600;
	}
	.text small {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.rename {
		font-weight: 600;
		min-height: 24px;
		padding: 1px 6px;
		width: 100%;
	}
	.proto {
		flex: none;
		width: 110px;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.spark {
		flex: none;
		width: 56px;
	}
	.ping {
		flex: none;
		width: 84px;
		min-height: 28px;
		border: 0;
		background: none;
		border-radius: 6px;
		text-align: left;
		font-size: 12px;
		font-variant-numeric: tabular-nums;
	}
	.ping:hover {
		background: var(--hairline);
	}
	.ping .muted {
		color: var(--fg-muted);
	}
	.icon-btn {
		flex: none;
		min-width: 28px;
		min-height: 28px;
		border: 0;
		background: none;
		border-radius: 6px;
		color: var(--fg-muted);
		font-size: 15px;
		line-height: 1;
	}
	.icon-btn:hover {
		background: var(--hairline);
		color: var(--fg);
	}
	.star.on {
		color: var(--warn);
	}
	.menu-wrap {
		position: relative;
		flex: none;
	}
	.menu {
		position: fixed;
		z-index: 999;
		min-width: 168px;
		display: grid;
		padding: 4px;
		border-radius: 10px;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		box-shadow: 0 12px 32px rgb(0 0 0 / 0.3);
	}
	.menu button {
		border: 0;
		background: none;
		text-align: left;
		padding: 6px 10px;
		border-radius: 6px;
		font-size: 13px;
	}
	.menu button:hover {
		background: var(--accent);
		color: white;
	}
	.menu button.danger {
		color: var(--danger);
	}
	.menu button.danger:hover {
		background: var(--danger);
		color: white;
	}
	.menu hr {
		width: 100%;
		margin: 4px 0;
		border: 0;
		border-top: 1px solid var(--hairline);
	}
	.empty {
		display: grid;
		justify-items: center;
		gap: 6px;
		padding: 56px 24px 40px;
		text-align: center;
	}
	.empty h2 {
		margin: 0;
		font-size: 17px;
	}
	.empty p {
		margin: 0;
		max-width: 360px;
		color: var(--fg-muted);
		line-height: 1.5;
	}
	.empty-actions {
		display: flex;
		gap: 8px;
		margin-top: 12px;
	}
	.no-match {
		margin: 0;
		text-align: center;
		color: var(--fg-muted);
	}
	.muted {
		color: var(--fg-muted);
	}
	.error {
		font-size: 12px;
		color: var(--danger);
	}
	.sub-row {
		display: grid;
		gap: 6px;
		align-items: stretch;
	}
	.sub-top {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.sub-top .grow {
		display: grid;
	}
	.btn.small {
		min-height: 26px;
		padding: 3px 10px;
		font-size: 12px;
	}
	.quota {
		display: grid;
		gap: 4px;
	}
	.meter {
		height: 4px;
		border-radius: 2px;
		background: var(--hairline);
		overflow: hidden;
	}
	.meter div {
		height: 100%;
		background: var(--accent);
	}
	.sub-form {
		flex-wrap: wrap;
	}
	.sub-form .name {
		width: 150px;
	}
	.sub-form .url {
		flex: 1;
		min-width: 160px;
	}
	@container (max-width: 560px) {
		.proto,
		.spark {
			display: none;
		}
	}
</style>
