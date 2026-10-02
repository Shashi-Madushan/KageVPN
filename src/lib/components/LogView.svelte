<script lang="ts">
	import { tick } from 'svelte';
	import type { Level } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import HeaderActions from './HeaderActions.svelte';
	import Segmented from './Segmented.svelte';

	const levels: Level[] = ['debug', 'info', 'warn', 'error'];
	const tags: Record<Level, string> = { debug: 'DBG', info: 'INFO', warn: 'WARN', error: 'ERR' };
	const options = levels.map((l) => ({ value: l, label: l[0].toUpperCase() + l.slice(1) }));

	let minLevel = $state<Level>('info');
	let query = $state('');
	let box: HTMLDivElement;
	let follow = $state(true);

	const visible = $derived(
		app.logs.filter(
			(l) =>
				levels.indexOf(l.level) >= levels.indexOf(minLevel) &&
				l.text.toLowerCase().includes(query.trim().toLowerCase())
		)
	);
	const problems = $derived(app.logs.filter((l) => l.level === 'warn' || l.level === 'error').length);

	$effect(() => {
		visible.length;
		if (follow) tick().then(() => box && (box.scrollTop = box.scrollHeight));
	});

	function onscroll() {
		follow = box.scrollHeight - box.scrollTop - box.clientHeight < 24;
	}

	function jumpToLatest() {
		follow = true;
		box.scrollTop = box.scrollHeight;
	}

	async function copy() {
		try {
			await navigator.clipboard.writeText(visible.map((l) => `[${tags[l.level]}] ${l.text}`).join('\n'));
			app.notify(`Copied ${visible.length} log line${visible.length === 1 ? '' : 's'}.`, 'info');
		} catch (e) {
			app.notify(String(e));
		}
	}
</script>

<HeaderActions>
	<button class="btn" disabled={!visible.length} onclick={copy}>Copy</button>
	<button class="btn danger" disabled={!app.logs.length} onclick={() => app.clearLogs()}>Clear</button>
</HeaderActions>

<div class="page">
	<div class="tools">
		<input class="input search" type="search" placeholder="Search logs" aria-label="Search logs" bind:value={query} />
		<Segmented {options} value={minLevel} label="Minimum level" onchange={(l) => (minLevel = l)} />
	</div>

	<div class="frame card">
		<div class="logs mono" bind:this={box} {onscroll} role="log" aria-live="off" aria-label="Log output">
			{#each visible as line, i (i)}
				<div class="line {line.level}">
					<span class="tag">{tags[line.level]}</span>
					<span class="text">{line.text}</span>
				</div>
			{:else}
				<div class="none">
					{#if app.logs.length}
						<strong>No matching lines</strong>
						<span>Try a lower level or a different search.</span>
					{:else}
						<strong>No logs yet</strong>
						<span>Output from the engine appears here once you connect.</span>
					{/if}
				</div>
			{/each}
		</div>
		{#if !follow && visible.length}
			<button class="jump btn primary" onclick={jumpToLatest}>↓ Jump to latest</button>
		{/if}
	</div>

	<div class="status">
		<span class="tabular">{visible.length} of {app.logs.length} lines</span>
		{#if problems}<span class="problems tabular">{problems} warning{problems === 1 ? '' : 's'} or error{problems === 1 ? '' : 's'}</span>{/if}
		<span class="spacer"></span>
		<span class="follow" class:live={follow}>{follow ? 'Following' : 'Paused'}</span>
	</div>
</div>

<style>
	.page {
		width: 100%;
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.tools {
		display: flex;
		gap: 10px;
		align-items: center;
		flex-wrap: wrap;
	}
	.search {
		flex: 1;
		min-width: 140px;
	}
	.frame {
		position: relative;
		flex: 1;
		min-height: 0;
		display: flex;
		overflow: hidden;
	}
	.logs {
		flex: 1;
		overflow-y: auto;
		padding: 6px 0;
		font-size: 12px;
		line-height: 1.5;
		user-select: text;
	}
	.line {
		display: grid;
		grid-template-columns: 44px minmax(0, 1fr);
		gap: 8px;
		padding: 1px 12px;
	}
	.line:hover {
		background: var(--hairline);
	}
	.tag {
		font-size: 11px;
		font-weight: 700;
		color: var(--fg-muted);
		letter-spacing: 0.02em;
	}
	.text {
		white-space: pre-wrap;
		word-break: break-word;
	}
	.warn .tag,
	.warn .text {
		color: var(--warn);
	}
	.error .tag,
	.error .text {
		color: var(--danger);
	}
	.error {
		background: color-mix(in srgb, var(--danger) 8%, transparent);
	}
	.debug .text {
		color: var(--fg-muted);
	}
	.none {
		display: grid;
		justify-items: center;
		gap: 4px;
		padding: 48px 16px;
		color: var(--fg-muted);
		text-align: center;
		font-family: -apple-system, BlinkMacSystemFont, sans-serif;
	}
	.none strong {
		color: var(--fg);
		font-size: 13px;
	}
	.jump {
		position: absolute;
		right: 14px;
		bottom: 12px;
		border-radius: 999px;
		box-shadow: 0 2px 8px rgb(0 0 0 / 0.25);
	}
	.status {
		display: flex;
		align-items: center;
		gap: 12px;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.status .spacer {
		flex: 1;
	}
	.problems {
		color: var(--warn);
	}
	.follow::before {
		content: '';
		display: inline-block;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--fg-muted);
		margin-right: 5px;
	}
	.follow.live::before {
		background: var(--ok);
	}
</style>
