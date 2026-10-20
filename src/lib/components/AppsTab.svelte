<script lang="ts">
	import { onMount } from 'svelte';
	import { api, usageApi, type InstalledApp, type RouteAction, type RoutingSettings } from '$lib/api';
	import { appChoice, applyPreset, blockedApps, otherAppRules, setAppChoice, type AppKind } from '$lib/appRules';
	import { caps } from '$lib/caps.svelte';
	import { barTone, periodSpan } from '$lib/capView';
	import { formatBytes } from '$lib/format';
	import type { Page } from '$lib/pages';
	import { installedApps } from './AppPicker.svelte';
	import CapEditor from './CapEditor.svelte';
	import Segmented from './Segmented.svelte';

	let { draft, onnavigate }: { draft: RoutingSettings; onnavigate?: (page: Page) => void } = $props();

	const PAGE = 80;
	const options: { value: RouteAction; label: string }[] = [
		{ value: 'proxy', label: 'VPN' },
		{ value: 'direct', label: 'Direct' },
		{ value: 'block', label: 'Block' }
	];
	let apps = $state<InstalledApp[]>([]);
	let loading = $state(true);
	let query = $state('');
	let limit = $state(PAGE);
	let icons = $state<Record<string, string | null>>({});
	/** The app whose cap editor is open. */
	let capOpen = $state<string | null>(null);

	onMount(() => caps.use());
	/** Bytes through the VPN this billing period, by app name in lower case. Null while recording is off. */
	let usage = $state<Map<string, number> | null>(null);

	/** "Slack Helper (Renderer)" belongs to "Slack"; the Connections page groups the same way. */
	const appName = (process: string) => process.replace(/\s+Helper.*$/, '').trim().toLowerCase();

	onMount(async () => {
		apps = await installedApps();
		loading = false;
		try {
			const [settings, quota] = await Promise.all([usageApi.settings(), usageApi.quota()]);
			if (!settings.recordApps) return;
			const rows = await usageApi.top({ kind: 'app', from: quota.cycleStart, to: Math.floor(Date.now() / 1000), limit: 500, filters: {} });
			const byApp = new Map<string, number>();
			for (const r of rows) byApp.set(appName(r.key), (byApp.get(appName(r.key)) ?? 0) + r.up + r.down);
			usage = byApp;
		} catch {
			usage = null;
		}
	});

	/** Loads an icon when its row scrolls into view. */
	function visible(node: HTMLElement, path: string) {
		const observer = new IntersectionObserver((entries) => {
			if (!entries.some((e) => e.isIntersecting)) return;
			observer.disconnect();
			if (path in icons) return;
			icons[path] = null;
			api.appIcon(path).then((url) => (icons[path] = url)).catch(() => {});
		});
		observer.observe(node);
		return { destroy: () => observer.disconnect() };
	}

	const installedPaths = $derived(new Set(apps.map((a) => a.path)));
	const blocked = $derived(blockedApps(draft.rules));
	const others = $derived(otherAppRules(draft.rules, installedPaths));
	const missingCaps = $derived(caps.list.filter((c) => !c.installed));

	const rows = $derived.by(() => {
		const q = query.trim().toLowerCase();
		const hits = q ? apps.filter((a) => a.name.toLowerCase().includes(q)) : apps;
		// Capped apps first (closest to their limit on top), then apps with a choice, then the rest.
		const rank = (a: InstalledApp) => (caps.byPath(a.path) ? 0 : appChoice(draft.rules, 'app', a.path) ? 1 : 2);
		const percent = (a: InstalledApp) => caps.byPath(a.path)?.percent ?? 0;
		return [...hits].sort((a, b) => rank(a) - rank(b) || percent(b) - percent(a) || a.name.localeCompare(b.name));
	});

	function choose(kind: AppKind, value: string, action: RouteAction | null) {
		draft.rules = setAppChoice($state.snapshot(draft.rules) as typeof draft.rules, kind, value, action, () => crypto.randomUUID());
		// A per-app choice only means something in Rules mode; the rest keeps the way it was routed.
		if (action !== null && draft.mode !== 'rule') {
			const next = applyPreset($state.snapshot(draft) as RoutingSettings, draft.mode === 'direct' ? 'only' : 'except');
			draft.mode = next.mode;
			draft.finalAction = next.finalAction;
		}
	}

	const labelOf = (kind: AppKind, value: string) => (kind === 'app' ? (apps.find((a) => a.path === value)?.name ?? value.replace(/\/+$/, '').split('/').pop()?.replace(/\.app$/i, '') ?? value) : value);
</script>

<div class="panel" role="tabpanel">
	<p class="lead">
		Choices (VPN, Direct, Block) apply when you press Save changes; data caps apply at once. An app with no choice follows
		{#if onnavigate}<button type="button" class="link" onclick={() => onnavigate('routing')}>the default set in Routing</button>{:else}the default set in Routing{/if}.
	</p>

	{#if blocked.length}
		<div class="blocked" aria-label="Blocked apps">
			<strong>Blocked ({blocked.length})</strong>
			{#each blocked as b (b.kind + b.value)}
				<span class="chip">
					{labelOf(b.kind, b.value)}{b.origin === 'cap' ? ' · data cap' : ''}
					{#if b.origin === 'manual'}
						<button type="button" onclick={() => choose(b.kind, b.value, null)} aria-label="Unblock {labelOf(b.kind, b.value)}">Unblock</button>
					{:else}
						<button type="button" onclick={() => caps.unblock(b.value)} aria-label="Unblock {labelOf(b.kind, b.value)}">Unblock</button>
					{/if}
				</span>
			{/each}
		</div>
	{/if}

	<input class="input" type="search" placeholder="Search your apps" bind:value={query} aria-label="Search apps" oninput={() => (limit = PAGE)} />

	{#if loading}
		<p class="empty">Reading your apps…</p>
	{:else if rows.length === 0}
		<p class="empty">No app matches “{query}”.</p>
	{:else}
		<div class="list">
			{#each rows.slice(0, limit) as a (a.path)}
				{@const choice = appChoice(draft.rules, 'app', a.path)}
				{@const used = usage?.get(a.name.toLowerCase())}
				{@const cap = caps.byPath(a.path)}
				<div class="row" use:visible={a.path}>
					<span class="icon" aria-hidden="true">
						{#if icons[a.path]}<img src={icons[a.path]} alt="" />{:else}{a.name.charAt(0).toUpperCase()}{/if}
					</span>
					<span class="what">
						<span class="name" title={a.path}>{a.name}</span>
						<small>
							{#if choice?.origin === 'cap'}Blocked by a data cap{:else if choice}Your choice{:else}Follows the default ({draft.mode === 'rule' && draft.finalAction === 'direct' ? 'direct' : draft.mode === 'direct' ? 'idle' : 'VPN'}){/if}
							{#if used && !cap}· {formatBytes(used)} this period{/if}
						</small>
						{#if cap}
							<span class="capline {barTone(cap.state)}">
								<span class="capbar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.min(100, Math.round(cap.percent))} aria-label="{a.name} data cap used"><i style="width:{Math.max(cap.percent > 0 ? 3 : 0, Math.min(100, cap.percent))}%"></i></span>
								<small>{formatBytes(cap.used)} of {formatBytes(cap.limitBytes)} {periodSpan(cap.period)}</small>
							</span>
						{/if}
					</span>
					<Segmented {options} value={(choice?.action ?? '') as RouteAction} label="What {a.name} does" disabled={choice?.origin === 'cap'} onchange={(v) => choose('app', a.path, v)} />
					{#if choice?.origin === 'manual'}
						<button type="button" class="reset" onclick={() => choose('app', a.path, null)} aria-label="Use the default for {a.name}" title="Use the default">↺</button>
					{:else}
						<span class="reset-gap"></span>
					{/if}
					<button type="button" class="capbtn {cap ? barTone(cap.state) : ''}" class:set={cap} aria-expanded={capOpen === a.path} aria-label="Data cap for {a.name}" onclick={() => (capOpen = capOpen === a.path ? null : a.path)}>
						{#if cap}{Math.round(cap.percent)}%{:else}Cap{/if}
					</button>
				</div>
				{#if capOpen === a.path}
					<CapEditor path={a.path} name={a.name} onclose={() => (capOpen = null)} />
				{/if}
			{/each}
		</div>
		{#if rows.length > limit}
			<button type="button" class="btn" onclick={() => (limit += PAGE)}>Show more ({rows.length - limit} left)</button>
		{/if}
	{/if}

	{#if others.length || missingCaps.length}
		<h4>Other app rules</h4>
		<p class="lead">
			Rules for programs without an app icon, or apps that are no longer installed. Edit them under
			{#if onnavigate}<button type="button" class="link" onclick={() => onnavigate('routing')}>Routing, Your rules</button>{:else}Routing, Your rules{/if}.
		</p>
		<div class="list">
			{#each others as r (r.id)}
				<div class="row">
					<span class="icon" aria-hidden="true">{labelOf(r.kind as AppKind, r.value).charAt(0).toUpperCase()}</span>
					<span class="what">
						<span class="name">{labelOf(r.kind as AppKind, r.value)}</span>
						<small>{r.kind === 'processName' ? 'Process' : 'Not installed'} · {options.find((o) => o.value === r.action)?.label}{r.enabled ? '' : ' (off)'}</small>
					</span>
				</div>
			{/each}
			{#each missingCaps as c (c.path)}
				<div class="row">
					<span class="icon" aria-hidden="true">{c.name.charAt(0).toUpperCase()}</span>
					<span class="what">
						<span class="name">{c.name}</span>
						<small>Data cap · not installed</small>
					</span>
					<button type="button" class="btn small" onclick={() => caps.remove(c.path)}>Remove cap</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.panel {
		display: grid;
		gap: 12px;
	}
	.lead {
		color: var(--fg-muted);
		font-size: 12px;
		margin: 0;
	}
	h4 {
		margin: 8px 0 0;
		font-size: 13px;
	}
	.blocked {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		font-size: 12px;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 2px 4px 2px 10px;
		border-radius: 999px;
		background: color-mix(in srgb, var(--danger) 14%, transparent);
	}
	.chip button {
		border: 0;
		border-radius: 999px;
		padding: 2px 8px;
		font-size: 11px;
		background: var(--hairline);
		color: inherit;
	}
	.list {
		border: 1px solid var(--hairline);
		border-radius: 12px;
		overflow: hidden;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 12px;
	}
	.row + .row {
		border-top: 1px solid var(--hairline);
	}
	.icon {
		flex: none;
		display: grid;
		place-items: center;
		width: 28px;
		height: 28px;
		border-radius: 7px;
		background: var(--hairline);
		color: var(--fg-muted);
		font-size: 13px;
		font-weight: 700;
		overflow: hidden;
	}
	.icon img {
		width: 100%;
		height: 100%;
		object-fit: contain;
	}
	.what {
		flex: 1;
		min-width: 0;
		display: grid;
	}
	.name {
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	small {
		color: var(--fg-muted);
		font-size: 11px;
	}
	.reset,
	.reset-gap {
		flex: none;
		width: 24px;
	}
	.reset {
		border: 0;
		background: none;
		color: var(--fg-muted);
		font-size: 14px;
	}
	.capbtn {
		flex: none;
		width: 64px;
		padding: 3px 0;
		border-radius: 999px;
		border: 1px solid var(--hairline);
		background: none;
		color: var(--fg-muted);
		font-size: 12px;
		text-align: center;
	}
	.capbtn:hover {
		color: var(--fg);
		border-color: var(--fg-muted);
	}
	.capline {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 3px;
	}
	.capbar {
		flex: 0 1 140px;
		height: 6px;
		min-width: 60px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--fg) 16%, transparent);
		overflow: hidden;
	}
	.capbar i {
		display: block;
		height: 100%;
		border-radius: 3px;
		background: var(--accent);
		transition: width 0.4s ease;
	}
	.capline.warn .capbar i {
		background: var(--warn);
	}
	.capline.over .capbar i {
		background: var(--danger);
	}
	.capline.off .capbar i {
		background: var(--fg-muted);
	}
	.capbtn.set {
		color: var(--fg);
	}
	.capbtn.warn {
		border-color: var(--warn);
	}
	.capbtn.over {
		border-color: var(--danger);
		color: var(--danger);
	}
	.btn.small {
		min-height: 24px;
		padding: 2px 10px;
		font-size: 12px;
	}
	.link {
		border: 0;
		background: none;
		padding: 0;
		color: var(--accent);
		font: inherit;
		text-decoration: underline;
		cursor: pointer;
	}
	.empty {
		color: var(--fg-muted);
		text-align: center;
		padding: 16px;
		margin: 0;
	}
	@media (max-width: 560px) {
		.row {
			flex-wrap: wrap;
		}
		.what {
			flex-basis: calc(100% - 50px);
		}
	}
</style>
