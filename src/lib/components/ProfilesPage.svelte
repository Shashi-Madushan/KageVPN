<script lang="ts">
	import { api, blankProfile, type Profile, type Subscription } from '$lib/api';
	import { clickOutside, focusOnMount } from '$lib/actions';
	import { app } from '$lib/app.svelte';
	import { formatBytes, latencyLabel, latencyTone, profileFlag } from '$lib/format';
	import ImportDialog from './ImportDialog.svelte';
	import MiniSpark from './MiniSpark.svelte';
	import ProfileDialog from './ProfileDialog.svelte';
	import QrDialog from './QrDialog.svelte';

	let importing = $state(false);
	let importText = $state('');
	let editing = $state<Profile | null>(null);
	let sharing = $state<Profile | null>(null);
	let menuFor = $state<string | null>(null);
	let renaming = $state<string | null>(null);
	let query = $state('');
	let sortBy = $state<'default' | 'latency' | 'name'>('default');
	let deleteTarget = $state<Profile | null>(null);
	let deleteSubTarget = $state<Subscription | null>(null);
	let subName = $state('');
	let subUrl = $state('');
	let subError = $state<string | null>(null);
	let subBusy = $state(false);

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

<div class="page">
	<div class="toolbar">
		<input type="search" placeholder="Search profiles" bind:value={query} aria-label="Search profiles" />
		<select bind:value={sortBy} aria-label="Sort profiles" class="sort-select">
			<option value="default">Default</option>
			<option value="latency">⚡ Lowest Ping</option>
			<option value="name">Name</option>
		</select>
		<button disabled={app.profiles.length === 0 || testing} onclick={() => app.testAll()}>{testing ? 'Testing…' : 'Test all'}</button>
		<button onclick={openImport}>Import</button>
		<button class="primary" onclick={() => (editing = blankProfile())}>New</button>
	</div>

	<section class="card auto">
		<label>
			<span>
				<strong>Auto: fastest profile</strong>
				<small>Tests all profiles every few minutes and uses the quickest. Fails over if one stops working.</small>
			</span>
			<input type="checkbox" checked={app.auto.enabled} onchange={(e) => app.setAuto({ ...app.auto, enabled: e.currentTarget.checked })} />
		</label>
		{#if app.auto.enabled && app.groups.length}
			<label class="inline">
				<small>Only from group</small>
				<select value={app.auto.group ?? ''} onchange={(e) => app.setAuto({ ...app.auto, group: e.currentTarget.value || null })}>
					<option value="">All profiles</option>
					{#each app.groups as group (group)}<option value={group}>{group}</option>{/each}
				</select>
			</label>
		{/if}
	</section>

	{#if app.profiles.length === 0}
		<div class="empty card">
			<strong>No profiles yet</strong>
			<span>Paste <span class="mono">vless://</span> links, add a subscription, or create one by hand.</span>
			<button class="primary" onclick={openImport}>Paste links</button>
		</div>
	{/if}

	{#each sections as section (section.group)}
		<section>
			{#if section.group || sections.length > 1}<span class="caption">{section.group || 'Ungrouped'}</span>{/if}
			<ul>
				{#each section.items as profile (profile.id)}
					<li class="card" class:active={profile.id === app.activeId && !app.auto.enabled}>
						<button class="pick" onclick={() => app.select(profile.id)} title={app.connected ? 'Switch live' : 'Select'}>
							<span class="radio"></span>
							<span class="item-flag">{profileFlag(profile.name, profile.server)}</span>
							<span class="text">
								{#if renaming === profile.id}
									<input
										class="rename"
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
									<strong ondblclick={() => (renaming = profile.id)}>{profile.name}</strong>
								{/if}
								<small class="mono">{profile.server}:{profile.port}</small>
							</span>
						</button>
						<span class="badge">{profile.security === 'reality' ? 'Reality' : 'TLS'} · {profile.transport.kind}</span>
						<MiniSpark values={app.latencyHistory[profile.id] ?? []} />
						<button class="ping {latencyTone(app.latency[profile.id])}" onclick={() => app.ping(profile.id)} title="Test TCP latency">
							{latencyLabel(app.latency[profile.id])}
						</button>
						<button class="star" class:on={profile.favorite} onclick={() => app.toggleFavorite(profile.id)} aria-label="Favorite" aria-pressed={profile.favorite}>
							{profile.favorite ? '★' : '☆'}
						</button>
						<span class="menu-wrap" use:clickOutside={() => menuFor === profile.id && (menuFor = null)}>
							<button class="more" onclick={() => (menuFor = menuFor === profile.id ? null : profile.id)} aria-label="More actions">⋯</button>
							{#if menuFor === profile.id}
								<div class="menu">
									<button onclick={() => ((renaming = profile.id), (menuFor = null))}>Rename</button>
									<button onclick={() => ((editing = profile), (menuFor = null))}>Edit…</button>
									<button onclick={() => (app.duplicate(profile.id), (menuFor = null))}>Duplicate</button>
									<button onclick={() => copyLink(profile)}>Copy link</button>
									<button onclick={() => ((sharing = profile), (menuFor = null))}>Show QR code</button>
									<button class="danger" onclick={() => { deleteTarget = profile; menuFor = null; }}>Delete</button>
								</div>
							{/if}
						</span>
					</li>
				{/each}
			</ul>
		</section>
	{/each}

	<section class="card subs">
		<span class="caption">Subscriptions</span>
		{#each app.subscriptions as sub (sub.id)}
			{@const q = quota(sub)}
			<div class="sub">
				<div class="sub-head">
					<strong>{sub.name}</strong>
					<small class="muted">{app.profiles.filter((p) => p.subscriptionId === sub.id).length} profiles · updated {ago(sub.lastUpdated)}</small>
					<span class="spacer"></span>
					<button onclick={() => app.refreshSubscription(sub.id)}>Refresh</button>
					<button class="danger" onclick={() => (deleteSubTarget = sub)}>Remove</button>
				</div>
				{#if sub.lastError}<small class="error">{sub.lastError}</small>{/if}
				{#if q}
					<div class="meter"><div style:width="{q.percent}%"></div></div>
					<small class="muted">
						{formatBytes(q.used)} of {formatBytes(q.total)}{q.expire ? ` · expires ${new Date(q.expire * 1000).toLocaleDateString()}` : ''}
					</small>
				{/if}
			</div>
		{/each}
		<form class="sub-form" onsubmit={(e) => (e.preventDefault(), addSubscription())}>
			<input placeholder="Name" bind:value={subName} aria-label="Subscription name" />
			<input class="url" placeholder="https://…/subscription" bind:value={subUrl} aria-label="Subscription URL" spellcheck="false" />
			<button class="primary" disabled={!subUrl.trim() || subBusy}>{subBusy ? 'Adding…' : 'Add'}</button>
		</form>
		{#if subError}<small class="error">{subError}</small>{/if}
	</section>
</div>

{#if deleteTarget}
	<dialog class="confirm-dialog" open>
		<div class="confirm-content">
			<h3>Delete Profile</h3>
			<p>Are you sure you want to delete <strong>{deleteTarget.name}</strong>? This cannot be undone.</p>
			<div class="confirm-actions">
				<button onclick={() => (deleteTarget = null)}>Cancel</button>
				<button class="primary danger-btn" onclick={() => { app.remove(deleteTarget!.id); deleteTarget = null; }}>Delete</button>
			</div>
		</div>
	</dialog>
{/if}

{#if deleteSubTarget}
	<dialog class="confirm-dialog" open>
		<div class="confirm-content">
			<h3>Remove Subscription</h3>
			<p>Are you sure you want to remove <strong>{deleteSubTarget.name}</strong>? All profiles from this subscription will also be removed.</p>
			<div class="confirm-actions">
				<button onclick={() => (deleteSubTarget = null)}>Cancel</button>
				<button class="primary danger-btn" onclick={() => { app.removeSubscription(deleteSubTarget!.id); deleteSubTarget = null; }}>Remove</button>
			</div>
		</div>
	</dialog>
{/if}

<ImportDialog bind:open={importing} initial={importText} />
<ProfileDialog profile={editing} onclose={() => (editing = null)} />
<QrDialog profile={sharing} onclose={() => (sharing = null)} />

<style>
	.page {
		display: grid;
		gap: 14px;
		align-content: start;
	}
	.toolbar {
		display: flex;
		gap: 8px;
	}
	.toolbar input {
		flex: 1;
		min-width: 0;
	}
	input,
	select {
		padding: 6px 10px;
		border-radius: 8px;
		border: 1px solid var(--hairline);
		background: var(--surface);
		user-select: text;
	}
	button {
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		border-radius: 8px;
		padding: 5px 12px;
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
	.auto {
		padding: 12px 14px;
		display: grid;
		gap: 8px;
	}
	.auto label {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 16px;
	}
	.auto label.inline {
		justify-content: flex-start;
	}
	.auto span {
		display: grid;
	}
	small {
		color: var(--fg-muted);
	}
	input[type='checkbox'] {
		width: 34px;
		height: 20px;
		padding: 0;
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
	ul {
		list-style: none;
		margin: 6px 0 0;
		padding: 0;
		display: grid;
		gap: 8px;
	}
	li {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 6px 4px 4px;
		transition: border-color 0.15s;
	}
	li.active {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 9%, var(--surface));
	}
	.pick {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 10px;
		border: 0;
		background: none;
		text-align: left;
		padding: 7px 8px;
		min-width: 0;
	}
	.radio {
		width: 14px;
		height: 14px;
		border-radius: 50%;
		border: 2px solid var(--hairline);
		flex: none;
	}
	li.active .radio {
		border-color: var(--accent);
		background: radial-gradient(var(--accent) 38%, transparent 42%);
	}
	.text {
		display: grid;
		min-width: 0;
	}
	.text small {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.rename {
		font-weight: 600;
		border-color: var(--accent);
		padding: 1px 5px;
		width: 100%;
	}
	.badge {
		font-size: 10px;
		padding: 2px 6px;
		border-radius: 6px;
		background: var(--hairline);
		color: var(--fg-muted);
		white-space: nowrap;
	}
	.ping,
	.star,
	.more {
		border: 0;
		background: none;
		padding: 4px 8px;
	}
	.ping {
		font-size: 12px;
		min-width: 60px;
		color: var(--fg-muted);
		font-variant-numeric: tabular-nums;
	}
	.ping.ok {
		color: var(--ok);
	}
	.ping.warn {
		color: var(--warn);
	}
	.ping.bad {
		color: var(--danger);
	}
	.star {
		color: var(--fg-muted);
		font-size: 15px;
		padding: 4px 4px;
	}
	.star.on {
		color: var(--warn);
	}
	.more {
		font-size: 16px;
		line-height: 1;
	}
	.menu-wrap {
		position: relative;
	}
	.menu {
		position: absolute;
		right: 0;
		top: calc(100% + 4px);
		z-index: 20;
		min-width: 150px;
		display: grid;
		padding: 4px;
		border-radius: 10px;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		box-shadow: 0 10px 30px rgb(0 0 0 / 0.3);
	}
	.menu button {
		border: 0;
		background: none;
		text-align: left;
		padding: 6px 10px;
		border-radius: 6px;
	}
	.menu button:hover {
		background: var(--hairline);
	}
	.empty {
		display: grid;
		justify-items: center;
		gap: 6px;
		padding: 22px 16px;
		text-align: center;
		color: var(--fg-muted);
	}
	.empty strong {
		color: var(--fg);
		font-size: 14px;
	}
	.subs {
		padding: 14px 16px;
		display: grid;
		gap: 10px;
	}
	.sub {
		display: grid;
		gap: 6px;
		padding-top: 10px;
		border-top: 1px solid var(--hairline);
	}
	.sub-head {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.spacer {
		flex: 1;
	}
	.muted {
		color: var(--fg-muted);
	}
	.error {
		color: var(--danger);
	}
	.meter {
		height: 5px;
		border-radius: 3px;
		background: var(--hairline);
		overflow: hidden;
	}
	.meter div {
		height: 100%;
		background: var(--accent);
	}
	.sub-form {
		display: flex;
		gap: 8px;
		padding-top: 10px;
		border-top: 1px solid var(--hairline);
	}
	.sub-form input {
		width: 110px;
	}
	.sub-form .url {
		flex: 1;
		width: auto;
		min-width: 0;
	}
	.sort-select {
		padding: 5px 8px;
		font-size: 12px;
		border-radius: 8px;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
	}
	.item-flag {
		font-size: 15px;
		margin-right: 2px;
		flex: none;
	}
	.confirm-dialog {
		width: min(380px, 92vw);
		border: 1px solid var(--hairline);
		border-radius: var(--radius);
		background: var(--surface-strong);
		color: var(--fg);
		padding: 18px;
		box-shadow: 0 20px 60px rgb(0 0 0 / 0.4);
		position: fixed;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		z-index: 100;
	}
	.confirm-content {
		display: grid;
		gap: 8px;
	}
	.confirm-content h3 {
		margin: 0;
		font-size: 15px;
	}
	.confirm-content p {
		margin: 0;
		color: var(--fg-muted);
		font-size: 12px;
	}
	.confirm-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 10px;
	}
	.danger-btn {
		background: var(--danger) !important;
		border-color: var(--danger) !important;
		color: white !important;
	}
</style>
