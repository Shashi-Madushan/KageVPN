<script lang="ts">
	import { app } from '$lib/app.svelte';
	import { releaseNotes, type NoteKind } from '$lib/releaseNotes';
	import { compareVersions } from '$lib/version';
	import type { Page } from '$lib/pages';

	let { onnavigate }: { onnavigate: (page: Page) => void } = $props();

	const labels: Record<NoteKind, string> = { new: 'New', improved: 'Improved', fixed: 'Fixed' };
	const date = (iso: string) => new Date(`${iso}T00:00:00`).toLocaleDateString(undefined, { day: 'numeric', month: 'long', year: 'numeric' });
	const isCurrent = (version: string) => compareVersions(version, app.info.version) === 0;
</script>

<div class="page">
	{#each releaseNotes as release (release.version)}
		<article class="card">
			<header>
				<div>
					<h2>Version {release.version}{#if isCurrent(release.version)}<span class="tag">Installed</span>{/if}</h2>
					<p>{release.headline}</p>
				</div>
				<time datetime={release.date}>{date(release.date)}</time>
			</header>
			<ul>
				{#each release.notes as note (note.title)}
					<li>
						<span class="kind {note.kind}">{labels[note.kind]}</span>
						<div class="body">
							<strong>{note.title}</strong>
							<span>{note.text}</span>
						</div>
						{#if note.page}
							<button class="go" aria-label="Open {note.title}" onclick={() => onnavigate(note.page!)}>Open</button>
						{/if}
					</li>
				{/each}
			</ul>
		</article>
	{/each}
	<p class="foot">The full history is in the Changelog of the documentation.</p>
</div>

<style>
	.page {
		display: grid;
		gap: 12px;
		align-content: start;
		max-width: 760px;
	}
	article {
		padding: 18px 20px;
		display: grid;
		gap: 12px;
	}
	header {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		gap: 12px;
		flex-wrap: wrap;
	}
	h2 {
		margin: 0;
		font-size: var(--text-xl);
		display: flex;
		align-items: center;
		gap: 8px;
	}
	header p {
		margin: 2px 0 0;
		color: var(--fg-muted);
		font-size: var(--text-md);
	}
	time,
	.foot {
		color: var(--fg-muted);
		font-size: var(--text-sm);
	}
	.tag {
		font-size: var(--text-xs);
		font-weight: 700;
		padding: 1px 8px;
		border-radius: 999px;
		background: color-mix(in srgb, var(--accent) 18%, transparent);
		color: var(--accent);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 10px;
	}
	li {
		display: flex;
		align-items: flex-start;
		gap: 10px;
	}
	.kind {
		flex: none;
		width: 66px;
		text-align: center;
		font-size: var(--text-xs);
		font-weight: 700;
		padding: 2px 0;
		border-radius: 6px;
		background: var(--hairline);
	}
	.kind.new {
		background: color-mix(in srgb, var(--accent) 18%, transparent);
		color: var(--accent);
	}
	.kind.improved {
		background: color-mix(in srgb, var(--ok) 18%, transparent);
		color: var(--ok-text);
	}
	.kind.fixed {
		background: color-mix(in srgb, var(--warn) 20%, transparent);
		color: var(--warn);
	}
	.body {
		flex: 1;
		display: grid;
		gap: 1px;
		font-size: var(--text-md);
	}
	.body span {
		color: var(--fg-muted);
	}
	.go {
		flex: none;
		font-size: var(--text-sm);
		padding: 2px 10px;
	}
	.foot {
		margin: 4px 2px;
	}
</style>
