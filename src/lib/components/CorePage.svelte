<script lang="ts">
	import { core } from '$lib/core.svelte';
	import { app } from '$lib/app.svelte';
	import { formatBytes } from '$lib/format';
	import { compareVersions } from '$lib/version';
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
</script>

<div class="page">
	<section class="card block">
		<span class="caption">Active core</span>
		{#if status?.active}
			{@const info = status.active}
			<div class="active-row">
				<strong class="version tabular">{info.version}</strong>
				<span class="pill">{info.source === 'managed' ? 'Managed by app' : 'System install'}</span>
			</div>
			<span class="mono path">{info.path}</span>
			<div class="tags">
				{#each ['with_clash_api', 'with_utls'] as tag (tag)}
					<span class="tag" class:bad={info.missingTags.includes(tag)}>{info.missingTags.includes(tag) ? '✕' : '✓'} {tag.replace('with_', '')}</span>
				{/each}
			</div>
			{#if info.missingTags.length || info.outdated}
				<p class="warn">This core is missing required features or is older than 1.12. Install a managed version below.</p>
			{/if}
		{:else}
			<p class="warn">{status?.error ?? 'Loading…'}</p>
			{#if status}<p class="muted">No sing-box core was found. Check for updates below and install the latest stable version.</p>{/if}
		{/if}
	</section>

	<section class="card block">
		<div class="row">
			<span class="caption">Updates</span>
			<Segmented options={channels} value={status?.settings.channel ?? 'stable'} label="Channel" disabled={core.checking || core.installing !== null} onchange={(c: Channel) => core.setChannel(c)} />
		</div>
		<div class="row">
			<span class="muted">
				{#if core.updateAvailable && core.newest}Version {core.newest.version} is available.
				{:else if core.checked}You are up to date.
				{:else}Check GitHub for new sing-box releases.{/if}
			</span>
			<button class="primary" disabled={core.checking || core.installing !== null} onclick={() => core.check()}>
				{core.checking ? 'Checking…' : 'Check for updates'}
			</button>
		</div>

		{#if core.releases.length}
			<ul class="releases">
				{#each core.releases as release (release.version)}
					{@const rel = relation(release)}
					<li>
						<div class="release-head">
							<strong class="tabular">{release.version}</strong>
							{#if release.prerelease}<span class="pill warnpill">pre-release</span>{/if}
							{#if rel === 'active'}<span class="pill okpill">Active</span>{/if}
							<span class="muted small">{date(release.publishedAt)} · {formatBytes(release.size)}</span>
							<span class="spacer"></span>
							{#if rel === 'active'}
								<span class="muted small">In use</span>
							{:else if rel === 'installed'}
								<button onclick={() => core.use(release.version)}>Use</button>
							{:else}
								<button class="primary" disabled={core.installing !== null || app.locked} onclick={() => install(release)}>
									{confirming === release.version ? 'Install anyway' : rel === 'older' ? 'Install older' : 'Install'}
								</button>
							{/if}
						</div>
						{#if confirming === release.version}
							<p class="warn small">This release has no published checksum, so its download cannot be verified.</p>
						{/if}
						{#if core.installing === release.version}
							<div class="progress" aria-label="Install progress">
								<div class="bar" style:width={percent === null ? '100%' : `${percent}%`} class:indeterminate={percent === null}></div>
							</div>
							<span class="muted small">{core.progress ? STAGES[core.progress.stage] : 'Starting…'}{percent !== null ? ` ${percent.toFixed(0)}%` : ''}</span>
						{/if}
						{#if release.notes}
							<button class="link" onclick={() => (expanded = expanded === release.version ? null : release.version)}>
								{expanded === release.version ? 'Hide notes' : 'Release notes'}
							</button>
							{#if expanded === release.version}<pre class="notes">{release.notes.slice(0, 1500)}</pre>{/if}
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
		{#if app.locked}<p class="muted small">Disconnect before installing a new core.</p>{/if}
	</section>

	<section class="card block">
		<span class="caption">Installed cores</span>
		<ul class="installed">
			<li>
				<div>
					<strong>System{status?.system ? ` ${status.system.version}` : ''}</strong>
					<span class="muted small mono">{status?.system?.path ?? 'Not found'}</span>
				</div>
				{#if managedActive === null && status?.system}
					<span class="pill okpill">Active</span>
				{:else}
					<button disabled={!status?.system} onclick={() => core.use(null)}>Use system</button>
				{/if}
			</li>
			{#each status?.installed ?? [] as item (item.version)}
				<li>
					<div>
						<strong>Managed {item.version}</strong>
						<span class="muted small">{item.installedAt ? `Installed ${new Date(item.installedAt * 1000).toLocaleDateString()}` : ''}</span>
					</div>
					{#if managedActive === item.version}
						<span class="pill okpill">Active</span>
					{:else}
						<span class="actions">
							<button onclick={() => core.use(item.version)}>Use</button>
							<button class="danger" onclick={() => core.remove(item.version)}>Remove</button>
						</span>
					{/if}
				</li>
			{/each}
		</ul>
	</section>

	<section class="card block">
		<span class="caption">Preferences</span>
		<label class="toggle-row">
			<span>
				<strong>Check automatically</strong>
				<small class="muted">Look for a new release once a day and notify you. Nothing installs by itself.</small>
			</span>
			<input type="checkbox" checked={status?.settings.autoCheck ?? true} onchange={(e) => core.setAutoCheck(e.currentTarget.checked)} />
		</label>
		<div class="row">
			<span class="muted small">Using Homebrew? <span class="mono">brew upgrade sing-box</span></span>
			<button disabled={core.installing !== null || app.locked} onclick={() => core.brewUpgrade()}>
				{core.installing === 'brew' ? 'Upgrading…' : 'Run upgrade'}
			</button>
		</div>
	</section>
</div>

<style>
	.page {
		display: grid;
		gap: 14px;
		align-content: start;
	}
	.block {
		padding: 14px 16px;
		display: grid;
		gap: 10px;
	}
	.row,
	.active-row,
	.release-head {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}
	.row {
		justify-content: space-between;
	}
	.spacer {
		flex: 1;
	}
	.version {
		font-size: 24px;
	}
	.path {
		font-size: 11px;
		color: var(--fg-muted);
		word-break: break-all;
		user-select: text;
	}
	.pill {
		font-size: 11px;
		padding: 2px 8px;
		border-radius: 999px;
		background: var(--hairline);
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
	.tags {
		display: flex;
		gap: 6px;
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
		margin: 0;
	}
	.muted {
		color: var(--fg-muted);
	}
	.small {
		font-size: 11px;
	}
	p {
		margin: 0;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
	}
	.releases li,
	.installed li {
		padding: 10px 0;
		border-top: 1px solid var(--hairline);
		display: grid;
		gap: 6px;
	}
	.installed li {
		grid-template-columns: 1fr auto;
		align-items: center;
	}
	.installed li div {
		display: grid;
	}
	.actions {
		display: flex;
		gap: 6px;
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
	.toggle-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 16px;
	}
	.toggle-row span {
		display: grid;
	}
</style>
