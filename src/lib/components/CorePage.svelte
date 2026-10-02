<script lang="ts">
	import { core } from '$lib/core.svelte';
	import { app } from '$lib/app.svelte';
	import { formatBytes } from '$lib/format';
	import { compareVersions } from '$lib/version';
	import HeaderActions from './HeaderActions.svelte';
	import Icon from './Icon.svelte';
	import Segmented from './Segmented.svelte';
	import type { Channel, Release } from '$lib/api';

	const channels = [
		{ value: 'stable' as const, label: 'Stable' },
		{ value: 'prerelease' as const, label: 'Pre-release' }
	];
	const STAGES = { download: 'Downloading', verify: 'Verifying checksum', extract: 'Extracting', check: 'Checking config', done: 'Done' };

	let confirming = $state<string | null>(null);
	let expanded = $state<string | null>(null);

	const status = $derived(core.status);
	const installedVersions = $derived(new Set(status?.installed.map((i) => i.version)));
	const managedActive = $derived(status?.settings.managedVersion ?? null);
	const percent = $derived(
		core.progress && core.progress.total > 0 ? Math.min(100, (core.progress.received / core.progress.total) * 100) : null
	);

	function relation(release: Release): 'active' | 'installed' | 'newer' | 'older' {
		if (release.version === core.activeVersion) return 'active';
		if (installedVersions.has(release.version)) return 'installed';
		return core.activeVersion && compareVersions(release.version, core.activeVersion) > 0 ? 'newer' : 'older';
	}

	function install(release: Release) {
		if (!release.sha256 && confirming !== release.version) {
			confirming = release.version;
			return;
		}
		confirming = null;
		core.install(release.version, !release.sha256);
	}

	const date = (iso: string | null) => (iso ? new Date(iso).toLocaleDateString() : '');

	const health = $derived(
		!status?.active ? 'bad' : status.active.missingTags.length || status.active.outdated ? 'warn' : 'ok'
	);
	const available = $derived(core.releases.filter((r) => relation(r) !== 'active' && !installedVersions.has(r.version)));
	const busy = $derived(core.checking || core.installing !== null);
</script>

<HeaderActions>
	<button class="btn primary" disabled={busy} onclick={() => core.check()}>{core.checking ? 'Checking…' : 'Check for updates'}</button>
</HeaderActions>

<div class="page">
	<section class="card hero {health}">
		<span class="badge"><Icon name="core" size={22} /></span>
		{#if status?.active}
			{@const info = status.active}
			<div class="hero-main">
				<div class="line">
					<strong class="version tabular">sing-box {info.version}</strong>
					<span class="pill">{info.source === 'managed' ? 'Managed by app' : 'System install'}</span>
				</div>
				<span class="mono path">{info.path}</span>
				<div class="tags">
					{#each ['with_clash_api', 'with_utls'] as tag (tag)}
						<span class="tag" class:bad={info.missingTags.includes(tag)}>{info.missingTags.includes(tag) ? '✕' : '✓'} {tag.replace('with_', '')}</span>
					{/each}
				</div>
				{#if info.missingTags.length || info.outdated}
					<p class="warn">This engine is missing required features or is older than 1.12. Install a managed version below.</p>
				{:else if core.updateAvailable && core.newest}
					<p class="note">Version {core.newest.version} is available.</p>
				{:else if core.checked}
					<p class="muted">You are up to date.</p>
				{/if}
			</div>
		{:else}
			<div class="hero-main">
				<strong class="version">{status ? 'No engine found' : 'Loading…'}</strong>
				<p class="muted">{status?.error ?? 'The VPN needs the sing-box engine to connect.'}</p>
				{#if status}
					<button class="btn primary install" disabled={busy} onclick={() => (core.releases.length ? install(core.releases[0]) : core.check())}>
						{core.releases.length ? `Install ${core.releases[0].version}` : 'Find latest version'}
					</button>
				{/if}
			</div>
		{/if}
	</section>

	<section class="card list">
		<div class="section-head">
			<div><strong>Versions</strong><small>Switch between installed engines or add a new one.</small></div>
			<Segmented options={channels} value={status?.settings.channel ?? 'stable'} label="Channel" disabled={busy} onchange={(c: Channel) => core.setChannel(c)} />
		</div>
		{#if app.locked}<p class="locked">Disconnect before installing or switching engines.</p>{/if}

		<div class="item">
			<div class="item-main">
				<strong>System{status?.system ? ` ${status.system.version}` : ''}</strong>
				<small class="mono">{status?.system?.path ?? 'Not found'}</small>
			</div>
			{#if managedActive === null && status?.system}
				<span class="pill okpill">Active</span>
			{:else}
				<button class="btn" disabled={!status?.system || app.locked} onclick={() => core.use(null)}>Use</button>
			{/if}
		</div>
		{#each status?.installed ?? [] as item (item.version)}
			<div class="item">
				<div class="item-main">
					<strong class="tabular">Managed {item.version}</strong>
					<small>{item.installedAt ? `Installed ${new Date(item.installedAt * 1000).toLocaleDateString()}` : 'Installed'}</small>
				</div>
				{#if managedActive === item.version}
					<span class="pill okpill">Active</span>
				{:else}
					<span class="actions">
						<button class="btn" disabled={app.locked} onclick={() => core.use(item.version)}>Use</button>
						<button class="btn danger" onclick={() => core.remove(item.version)}>Remove</button>
					</span>
				{/if}
			</div>
		{/each}

		{#if available.length}
			<div class="subhead caption">Available to install</div>
			{#each available as release (release.version)}
				{@const rel = relation(release)}
				<div class="item stack">
					<div class="item-row">
						<div class="item-main">
							<span class="title">
								<strong class="tabular">{release.version}</strong>
								{#if release.prerelease}<span class="pill warnpill">pre-release</span>{/if}
								{#if rel === 'newer'}<span class="pill newpill">New</span>{/if}
							</span>
							<small>{date(release.publishedAt)} · {formatBytes(release.size)}</small>
						</div>
						<button class="btn" class:primary={rel === 'newer'} disabled={core.installing !== null || app.locked} onclick={() => install(release)}>
							{confirming === release.version ? 'Install anyway' : rel === 'older' ? 'Install older' : 'Install'}
						</button>
					</div>
					{#if confirming === release.version}
						<p class="warn small">This release has no published checksum, so its download cannot be verified.</p>
					{/if}
					{#if core.installing === release.version}
						<div class="progress" aria-label="Install progress">
							<div class="bar" style:width={percent === null ? '100%' : `${percent}%`} class:indeterminate={percent === null}></div>
						</div>
						<small>{core.progress ? STAGES[core.progress.stage] : 'Starting…'}{percent !== null ? ` ${percent.toFixed(0)}%` : ''}</small>
					{/if}
					{#if release.notes}
						<button class="link" onclick={() => (expanded = expanded === release.version ? null : release.version)}>
							{expanded === release.version ? 'Hide notes' : 'Release notes'}
						</button>
						{#if expanded === release.version}<pre class="notes">{release.notes.slice(0, 1500)}</pre>{/if}
					{/if}
				</div>
			{/each}
		{:else if core.checked}
			<div class="subhead muted">No other versions on this channel.</div>
		{/if}
	</section>

	<section class="card list">
		<label class="item">
			<span class="item-main"><strong>Check automatically</strong><small>Look for a new release once a day and notify you. Nothing installs by itself.</small></span>
			<input type="checkbox" checked={status?.settings.autoCheck ?? true} onchange={(e) => core.setAutoCheck(e.currentTarget.checked)} />
		</label>
		<div class="item">
			<span class="item-main"><strong>Using Homebrew?</strong><small class="mono">brew upgrade sing-box</small></span>
			<button class="btn" disabled={core.installing !== null || app.locked} onclick={() => core.brewUpgrade()}>
				{core.installing === 'brew' ? 'Upgrading…' : 'Run upgrade'}
			</button>
		</div>
	</section>
</div>

<style>
	.page {
		display: grid;
		gap: 12px;
		align-content: start;
	}
	small,
	.muted {
		color: var(--fg-muted);
	}
	p {
		margin: 0;
	}
	.small {
		font-size: 11px;
	}
	.hero {
		display: flex;
		gap: 16px;
		padding: 18px 20px;
		align-items: flex-start;
	}
	.hero.warn {
		border-color: var(--warn);
	}
	.hero.bad {
		border-color: var(--danger);
	}
	.badge {
		display: grid;
		place-items: center;
		width: 44px;
		height: 44px;
		border-radius: 12px;
		flex: none;
		background: color-mix(in srgb, var(--ok) 18%, transparent);
		color: var(--ok);
	}
	.hero.warn .badge {
		background: color-mix(in srgb, var(--warn) 20%, transparent);
		color: var(--warn);
	}
	.hero.bad .badge {
		background: color-mix(in srgb, var(--danger) 18%, transparent);
		color: var(--danger);
	}
	.hero-main {
		display: grid;
		gap: 6px;
		min-width: 0;
		flex: 1;
	}
	.line {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}
	.version {
		font-size: 20px;
		font-weight: 650;
	}
	.path {
		font-size: 11px;
		color: var(--fg-muted);
		word-break: break-all;
		user-select: text;
	}
	.install {
		justify-self: start;
		margin-top: 4px;
	}
	.pill {
		font-size: 11px;
		padding: 2px 8px;
		border-radius: 999px;
		background: var(--hairline);
		color: var(--fg-muted);
		white-space: nowrap;
	}
	.okpill {
		background: color-mix(in srgb, var(--ok) 20%, transparent);
		color: var(--ok);
		font-weight: 600;
	}
	.warnpill {
		background: color-mix(in srgb, var(--warn) 22%, transparent);
		color: var(--warn);
	}
	.newpill {
		background: color-mix(in srgb, var(--accent) 20%, transparent);
		color: var(--accent);
		font-weight: 600;
	}
	.tags {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}
	.tag {
		font-size: 11px;
		padding: 2px 8px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--ok) 15%, transparent);
		color: var(--ok);
	}
	.tag.bad {
		background: color-mix(in srgb, var(--danger) 15%, transparent);
		color: var(--danger);
	}
	.warn {
		color: var(--warn);
	}
	.note {
		color: var(--accent);
		font-weight: 600;
	}
	.list {
		display: grid;
	}
	.section-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 8px 12px;
		padding: 12px 16px;
	}
	.section-head div {
		display: grid;
	}
	.locked {
		padding: 8px 16px;
		font-size: 12px;
		color: var(--warn);
		background: color-mix(in srgb, var(--warn) 10%, transparent);
	}
	.item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 11px 16px;
		border-top: 1px solid var(--hairline);
	}
	.item.stack {
		display: grid;
		gap: 6px;
	}
	.item-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.item-main {
		display: grid;
		min-width: 0;
	}
	.item-main small {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.title {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.subhead {
		padding: 10px 16px 6px;
		border-top: 1px solid var(--hairline);
	}
	.actions {
		display: flex;
		gap: 6px;
	}
	.btn:disabled {
		opacity: 0.45;
	}
	button.link {
		justify-self: start;
		border: 0;
		background: none;
		padding: 0;
		color: var(--accent);
		font-size: 11px;
	}
	.notes {
		margin: 0;
		padding: 10px;
		border-radius: var(--radius-sm);
		background: var(--hairline);
		font: 11px/1.45 ui-monospace, 'SF Mono', Menlo, monospace;
		white-space: pre-wrap;
		max-height: 180px;
		overflow: auto;
		user-select: text;
	}
	.progress {
		height: 5px;
		border-radius: 3px;
		background: var(--hairline);
		overflow: hidden;
	}
	.bar {
		height: 100%;
		background: var(--accent);
		transition: width 0.2s;
	}
	.bar.indeterminate {
		animation: slide 1.1s ease-in-out infinite;
		transform-origin: left;
	}
	@keyframes slide {
		50% {
			transform: scaleX(0.35);
		}
	}
</style>
