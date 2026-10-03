<script lang="ts">
	import { core } from '$lib/core.svelte';
	import { STAGES } from '$lib/downloads.svelte';
	import { app } from '$lib/app.svelte';
	import { formatBytes } from '$lib/format';
	import { compareVersions } from '$lib/version';
	import HeaderActions from './HeaderActions.svelte';
	import Segmented from './Segmented.svelte';
	import Select from './Select.svelte';
	import type { Channel, Release, UpdatePolicy } from '$lib/api';

	const channels = [
		{ value: 'stable' as const, label: 'Stable' },
		{ value: 'prerelease' as const, label: 'Pre-release' }
	];
	const policies = [
		{ value: 'ask' as const, label: 'Ask me' },
		{ value: 'auto' as const, label: 'Install automatically' },
		{ value: 'off' as const, label: 'Do nothing' }
	];

	let confirming = $state<string | null>(null);
	let expanded = $state<string | null>(null);

	const status = $derived(core.status);
	const coreOptions = $derived(
		(status?.cores ?? []).map((c) => ({ value: c.id, label: c.runnable ? c.name : `${c.name} (download only)` }))
	);
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
		core.installBackground(release.version, !release.sha256);
	}

	const date = (iso: string | null) => (iso ? new Date(iso).toLocaleDateString() : '');

	const health = $derived(
		!status?.active ? 'bad' : status.active.missingTags.length || status.active.outdated ? 'warn' : 'ok'
	);
	const available = $derived(core.releases.filter((r) => relation(r) !== 'active' && !installedVersions.has(r.version)));
	const busy = $derived(core.checking || core.installing !== null);
</script>

<HeaderActions>
	<button class="btn" disabled={busy} onclick={() => core.check()}>{core.checking ? 'Checking…' : 'Check for updates'}</button>
</HeaderActions>

<div class="page">
	{#if status && status.cores.length > 1}
		<div class="section">
			<div class="section-head"><span class="caption">Core</span></div>
			<div class="group">
				<div class="item">
					<span class="grow">Core used for connections<span class="sub">Each core keeps its own versions and update settings.</span></span>
					<Select options={coreOptions} value={status.core.id} label="Core" disabled={app.locked || busy} onchange={(id: string) => core.setActive(id)} />
				</div>
			</div>
		</div>
	{/if}

	<div class="section">
		<div class="section-head"><span class="caption">Active engine</span></div>
		<div class="group">
			{#if status?.active}
				{@const info = status.active}
				<div class="item">
					<span class="label">Version</span>
					<span class="grow mono tabular">{status.core.name} {info.version}</span>
					{#if info.missingTags.length || info.outdated}
						<span class="state warn">Needs a newer build</span>
					{:else if core.updateAvailable && core.newest}
						<span class="state info">{core.newest.version} available</span>
					{:else if core.checked}
						<span class="state ok">Up to date</span>
					{:else}
						<span class="state ok">Ready</span>
					{/if}
				</div>
				<div class="item">
					<span class="label">Source</span>
					<span class="grow">{info.source === 'managed' ? 'Managed by this app' : 'System install'}</span>
				</div>
				<div class="item">
					<span class="label">Location</span>
					<span class="grow mono path" title={info.path}>{info.path}</span>
				</div>
				<div class="item">
					<span class="label">Features</span>
					<span class="grow features">
						{#each status.core.requiredFeatures as tag (tag)}
							{@const missing = info.missingTags.includes(tag)}
							<span class="mono" class:missing>{missing ? '✕' : '✓'} {tag.replace('with_', '')}</span>
						{/each}
					</span>
				</div>
				{#if info.missingTags.length || info.outdated}
					<div class="item note"><span class="grow">This engine is missing required features or is older than {status.core.minVersion}. Install a managed version below.</span></div>
				{/if}
			{:else}
				<div class="item">
					<span class="grow">
						<strong>{status ? 'No engine found' : 'Loading…'}</strong>
						<span class="sub">{status?.error ?? 'The VPN needs the sing-box engine to connect.'}</span>
					</span>
					{#if status}
						<button class="btn primary" disabled={busy} onclick={() => (core.releases.length ? install(core.releases[0]) : core.check())}>
							{core.releases.length ? `Install ${core.releases[0].version}` : 'Find latest version'}
						</button>
					{/if}
				</div>
			{/if}
		</div>
	</div>

	<div class="section">
		<div class="section-head">
			<span class="caption">Versions</span>
			<Segmented options={channels} value={status?.settings.channel ?? 'stable'} label="Channel" disabled={busy} onchange={(c: Channel) => core.setChannel(c)} />
		</div>
		<div class="group table" role="table" aria-label="Engine versions">
			<div class="item th" role="row">
				<span class="grow" role="columnheader">Version</span>
				<span class="meta" role="columnheader">Released · size</span>
				<span class="st" role="columnheader">Status</span>
				<span class="act"></span>
			</div>

			<div class="item" role="row">
				<span class="grow">
					<span class="mono tabular">{status?.system ? status.system.version : 'System'}</span>
					<span class="sub">System · {status?.system?.path ?? 'not found'}</span>
				</span>
				<span class="meta"></span>
				<span class="st">{#if managedActive === null && status?.system}<span class="state ok">Active</span>{:else if !status?.system}<span class="muted">Not installed</span>{:else}<span class="muted">Installed</span>{/if}</span>
				<span class="act">
					{#if !(managedActive === null && status?.system)}
						<button class="btn" disabled={!status?.system || app.locked} onclick={() => core.use(null)}>Use</button>
					{/if}
				</span>
			</div>

			{#each status?.installed ?? [] as item (item.version)}
				<div class="item" role="row">
					<span class="grow">
						<span class="mono tabular">{item.version}</span>
						<span class="sub">Managed</span>
					</span>
					<span class="meta">{item.installedAt ? `Installed ${new Date(item.installedAt * 1000).toLocaleDateString()}` : ''}</span>
					<span class="st">{#if managedActive === item.version}<span class="state ok">Active</span>{:else}<span class="muted">Installed</span>{/if}</span>
					<span class="act">
						{#if managedActive !== item.version}
							<button class="btn" disabled={app.locked} onclick={() => core.use(item.version)}>Use</button>
							<button class="btn danger" onclick={() => core.remove(item.version)}>Remove</button>
						{/if}
					</span>
				</div>
			{/each}

			{#each available as release (release.version)}
				{@const rel = relation(release)}
				<div class="item avail" role="row">
					<span class="grow">
						<span class="mono tabular">{release.version}</span>
						{#if release.prerelease}<span class="sub">Pre-release</span>{/if}
					</span>
					<span class="meta">{date(release.publishedAt)} · {formatBytes(release.size)}</span>
					<span class="st">{#if rel === 'newer'}<span class="state info">Newer</span>{:else}<span class="muted">Older</span>{/if}</span>
					<span class="act">
						<button class="btn" class:primary={rel === 'newer'} disabled={core.installing !== null || app.locked} onclick={() => install(release)}>
							{confirming === release.version ? 'Install anyway' : 'Install'}
						</button>
					</span>
					{#if confirming === release.version || core.installing === release.version || release.notes}
						<div class="extra">
							{#if confirming === release.version}
								<p class="warn">This release has no published checksum, so its download cannot be verified.</p>
							{/if}
							{#if core.installing === release.version}
								<div class="progress" aria-label="Install progress">
									<div class="bar" style:width={percent === null ? '100%' : `${percent}%`} class:indeterminate={percent === null}></div>
								</div>
								<span class="sub">{core.progress ? STAGES[core.progress.stage] : 'Starting…'}{percent !== null ? ` ${percent.toFixed(0)}%` : ''}</span>
							{/if}
							{#if release.notes}
								<button class="linkbtn" onclick={() => (expanded = expanded === release.version ? null : release.version)}>
									{expanded === release.version ? 'Hide release notes' : 'Release notes'}
								</button>
								{#if expanded === release.version}<pre class="notes">{release.notes.slice(0, 1500)}</pre>{/if}
							{/if}
						</div>
					{/if}
				</div>
			{/each}
		</div>
		{#if app.locked}<p class="foot">Disconnect before installing or switching engines.</p>{:else if core.checked && !available.length}<p class="foot">No other versions on this channel.</p>{/if}
	</div>

	<div class="section">
		<div class="section-head"><span class="caption">Updates</span></div>
		<div class="group">
			<div class="item">
				<span class="grow">When an update is available<span class="sub">Checked once a day. Automatic installs only replace the engine this app downloaded, and take effect while disconnected.</span></span>
				<Select options={policies} value={status?.settings.updatePolicy ?? 'ask'} label="Update policy" onchange={(p: UpdatePolicy) => core.setPolicy(p)} />
			</div>
			<div class="item">
				<span class="grow">Upgrade with Homebrew<span class="sub mono">brew upgrade sing-box</span></span>
				<button class="btn" disabled={core.installing !== null || app.locked} onclick={() => core.brewUpgrade()}>
					{core.installing === 'brew' ? 'Upgrading…' : 'Run'}
				</button>
			</div>
		</div>
	</div>
</div>

<style>
	.page {
		container-type: inline-size;
		display: grid;
		gap: 18px;
		align-content: start;
	}
	.muted {
		color: var(--fg-muted);
	}
	p {
		margin: 0;
	}
	.path {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		user-select: text;
	}
	.features {
		display: flex;
		gap: 14px;
		flex-wrap: wrap;
	}
	.features .missing {
		color: var(--danger);
	}
	.note {
		color: var(--warn);
		background: color-mix(in srgb, var(--warn) 8%, transparent);
	}
	.table .th {
		min-height: 30px;
		padding-block: 4px;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--fg-muted);
		background: color-mix(in srgb, var(--hairline) 50%, transparent);
	}
	.table .item {
		flex-wrap: wrap;
	}
	.meta {
		width: 150px;
		flex: none;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.st {
		width: 96px;
		flex: none;
		font-size: 12px;
	}
	.act {
		width: 132px;
		flex: none;
		display: flex;
		justify-content: flex-end;
		gap: 6px;
	}
	.grow > .sub {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.extra .linkbtn {
		justify-self: start;
	}
	.extra {
		flex-basis: 100%;
		display: grid;
		gap: 6px;
		padding-bottom: 4px;
	}
	.warn {
		color: var(--warn);
		font-size: 12px;
	}
	.foot {
		padding: 0 4px;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.btn {
		min-height: 26px;
		padding: 3px 10px;
		font-size: 12px;
	}
	.btn:disabled {
		opacity: 0.45;
	}
	.notes {
		margin: 0;
		padding: 10px;
		border-radius: 8px;
		background: var(--hairline);
		font: 11px/1.45 ui-monospace, 'SF Mono', Menlo, monospace;
		white-space: pre-wrap;
		max-height: 180px;
		overflow: auto;
		user-select: text;
	}
	.progress {
		height: 4px;
		border-radius: 2px;
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
	@container (max-width: 500px) {
		.meta,
		.th .meta {
			display: none;
		}
		.item .label {
			width: 80px;
		}
	}
</style>
