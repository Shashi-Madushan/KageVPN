<script lang="ts" module>
	import { api, type InstalledApp } from '$lib/api';

	/** The installed apps, read once per window; the list barely changes while the app runs. */
	let cache: Promise<InstalledApp[]> | null = null;
	export function installedApps(): Promise<InstalledApp[]> {
		cache ??= api.installedApps().catch(() => ((cache = null), []));
		return cache;
	}

	/** "Slack" for `/Applications/Slack.app`. */
	export function appLabel(path: string): string {
		return path.replace(/\/+$/, '').split('/').pop()?.replace(/\.app$/i, '') || path;
	}
</script>

<script lang="ts">
	let { value = $bindable(''), onpick }: { value?: string; onpick?: (app: InstalledApp) => void } = $props();

	const MAX_SHOWN = 60;
	const listId = $props.id();

	let apps = $state<InstalledApp[]>([]);
	let loading = $state(true);
	let query = $state('');
	let open = $state(false);
	let focused = $state(0);
	let input = $state<HTMLInputElement>();
	let menu = $state<HTMLDivElement>();
	let pos = $state({ left: 0, top: 0, width: 0 });

	installedApps().then((list) => {
		apps = list;
		loading = false;
	});

	// Show the chosen app's name rather than its path.
	$effect(() => {
		if (!open) query = value ? (apps.find((a) => a.path === value)?.name ?? appLabel(value)) : '';
	});

	const matches = $derived.by(() => {
		const q = open ? query.trim().toLowerCase() : '';
		const hits = q ? apps.filter((a) => a.name.toLowerCase().includes(q) || a.path.toLowerCase().includes(q)) : apps;
		return hits.slice(0, MAX_SHOWN);
	});

	function show() {
		if (!input) return;
		const r = input.getBoundingClientRect();
		pos = { left: r.left, top: r.bottom + 4, width: Math.max(r.width, 260) };
		open = true;
		focused = 0;
	}

	function pick(app: InstalledApp) {
		value = app.path;
		open = false;
		onpick?.(app);
	}

	function move(delta: number) {
		if (!matches.length) return;
		focused = (focused + delta + matches.length) % matches.length;
		queueMicrotask(() => menu?.querySelector<HTMLElement>('[data-focused="true"]')?.scrollIntoView({ block: 'nearest' }));
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') (e.preventDefault(), open ? move(1) : show());
		else if (e.key === 'ArrowUp') (e.preventDefault(), move(-1));
		else if (e.key === 'Enter' && open && matches[focused]) (e.preventDefault(), pick(matches[focused]));
		else if (e.key === 'Escape' && open) (e.preventDefault(), e.stopPropagation(), (open = false));
	}

	function onwindowpointer(e: PointerEvent) {
		const t = e.target as Node;
		if (open && !input?.contains(t) && !menu?.contains(t)) open = false;
	}

	function where(path: string) {
		return path.replace(/\/[^/]+$/, '').replace(/^\/System/, '') || '/';
	}
</script>

<svelte:window onpointerdown={onwindowpointer} onresize={() => (open = false)} />

<input
	class="input grow picker"
	bind:this={input}
	bind:value={query}
	placeholder={loading ? 'Loading apps…' : 'Search your apps'}
	aria-label="App"
	role="combobox"
	aria-controls={listId}
	aria-expanded={open}
	aria-autocomplete="list"
	spellcheck="false"
	onfocus={show}
	oninput={() => (open ? (focused = 0) : show())}
	{onkeydown}
/>

{#if open}
	<div class="menu" id={listId} bind:this={menu} role="listbox" aria-label="Installed apps" style:left="{pos.left}px" style:top="{pos.top}px" style:width="{pos.width}px">
		{#each matches as app, i (app.path)}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<div class="option" class:focused={i === focused} data-focused={i === focused} role="option" tabindex="-1" aria-selected={app.path === value} onpointermove={() => (focused = i)} onclick={() => pick(app)}>
				<span class="name">{app.name}</span>
				<span class="where">{where(app.path)}</span>
			</div>
		{:else}
			<div class="none">{loading ? 'Loading…' : 'No app matches.'}</div>
		{/each}
	</div>
{/if}

<style>
	.picker {
		min-width: 120px;
	}
	.menu {
		position: fixed;
		z-index: 2000;
		max-height: 220px;
		max-width: calc(100vw - 16px);
		overflow-y: auto;
		padding: 4px;
		border: 1px solid var(--hairline);
		border-radius: 9px;
		background: color-mix(in srgb, var(--surface-strong) 96%, transparent);
		backdrop-filter: blur(20px);
		-webkit-backdrop-filter: blur(20px);
		box-shadow: 0 8px 24px rgb(0 0 0 / 0.28);
	}
	.option {
		display: flex;
		align-items: baseline;
		gap: 8px;
		padding: 5px 8px;
		border-radius: 6px;
		font-size: 12px;
		cursor: pointer;
	}
	.option.focused {
		background: var(--hairline);
	}
	.name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.where,
	.none {
		font-size: 11px;
		color: var(--fg-muted);
	}
	.none {
		padding: 6px 8px;
	}
</style>
