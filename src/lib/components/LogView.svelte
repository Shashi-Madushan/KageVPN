<script lang="ts">
	import Select from './Select.svelte';
	import { tick } from 'svelte';
	import type { Level } from '$lib/api';
	import { app } from '$lib/app.svelte';

	const levels: Level[] = ['debug', 'info', 'warn', 'error'];
	let minLevel = $state<Level>('info');
	let query = $state('');
	let box: HTMLDivElement;
	let follow = true;

	const visible = $derived(
		app.logs.filter(
			(l) =>
				levels.indexOf(l.level) >= levels.indexOf(minLevel) &&
				l.text.toLowerCase().includes(query.toLowerCase())
		)
	);

	$effect(() => {
		visible.length;
		if (follow) tick().then(() => box && (box.scrollTop = box.scrollHeight));
	});

	function onscroll() {
		follow = box.scrollHeight - box.scrollTop - box.clientHeight < 24;
	}
</script>

<div class="tools">
	<input placeholder="Search logs" bind:value={query} />
	<Select bind:value={minLevel} label="Minimum level" options={levels.map((l) => ({ value: l, label: l }))} />
	<button onclick={() => app.clearLogs()}>Clear</button>
</div>

<div class="logs mono" bind:this={box} {onscroll}>
	{#each visible as line, i (i)}
		<div class="line {line.level}">{line.text}</div>
	{:else}
		<div class="none">No logs yet.</div>
	{/each}
</div>

<style>
	:global(.logs-root) {
		display: flex;
		flex-direction: column;
		height: 100%;
	}
	.tools {
		display: flex;
		gap: 6px;
		margin-bottom: 8px;
	}
	input,
	button {
		background: var(--surface);
		border: 1px solid var(--hairline);
		border-radius: 8px;
		padding: 5px 8px;
	}
	input {
		flex: 1;
		min-width: 0;
		user-select: text;
	}
	.logs {
		flex: 1;
		overflow-y: auto;
		background: var(--surface);
		border: 1px solid var(--hairline);
		border-radius: var(--radius);
		padding: 8px;
		font-size: 11px;
		user-select: text;
	}
	.line {
		white-space: pre-wrap;
		word-break: break-all;
		padding: 1px 0;
	}
	.warn {
		color: var(--warn);
	}
	.error {
		color: var(--danger);
	}
	.debug {
		color: var(--fg-muted);
	}
	.none {
		color: var(--fg-muted);
		text-align: center;
		padding: 24px 0;
	}
</style>
