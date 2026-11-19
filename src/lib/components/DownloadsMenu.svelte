<script lang="ts">
	import { downloads } from '$lib/downloads.svelte';
	import { formatBytes } from '$lib/format';
	import Icon from './Icon.svelte';
	import ProgressBar from './ProgressBar.svelte';

	let open = $state(false);
	let root = $state<HTMLDivElement>();

	const busy = $derived(downloads.active.length > 0);
	const failed = $derived(downloads.finished.some((i) => i.state === 'failed'));
	const ring = 2 * Math.PI * 9;

	function onwindowpointer(e: PointerEvent) {
		if (open && root && !root.contains(e.target as Node)) open = false;
	}
</script>

<svelte:window onpointerdown={onwindowpointer} onkeydown={(e) => e.key === 'Escape' && (open = false)} onblur={() => (open = false)} />

<div class="downloads" bind:this={root}>
	<button
		type="button"
		class="trigger"
		class:busy
		class:open
		aria-label={busy ? `Downloads, ${downloads.active.length} in progress` : 'Downloads'}
		aria-haspopup="dialog"
		aria-expanded={open}
		title="Downloads"
		onclick={() => (open = !open)}
	>
		{#if busy}
			<svg class="ring" viewBox="0 0 24 24" aria-hidden="true">
				<circle cx="12" cy="12" r="9" class="track" />
				<circle cx="12" cy="12" r="9" class="fill" class:spin={downloads.overall === null} stroke-dasharray={ring} stroke-dashoffset={ring * (1 - (downloads.overall ?? 30) / 100)} />
			</svg>
			<span class="arrow"><Icon name="download" size={11} /></span>
		{:else}
			<Icon name="download" size={15} />
		{/if}
		{#if failed && !busy}<span class="dot" aria-hidden="true"></span>{/if}
	</button>

	{#if open}
		<div class="panel" role="dialog" aria-label="Downloads">
			<div class="panel-head">
				<strong>Downloads</strong>
				{#if downloads.finished.length}<button type="button" class="linkbtn" onclick={() => downloads.clearFinished()}>Clear</button>{/if}
			</div>
			{#each downloads.items as item (item.id)}
				<div class="row" class:failed={item.state === 'failed'}>
					<div class="line">
						<span class="title">{item.title}</span>
						{#if item.state === 'active' && item.cancel}
							<button type="button" class="linkbtn" onclick={item.cancel}>Cancel</button>
						{:else if item.state !== 'active' && item.state !== 'done' && item.retry}
							<button type="button" class="linkbtn" onclick={item.retry}>Retry</button>
						{/if}
					</div>
					{#if item.state === 'active'}
						<ProgressBar value={item.percent} label={item.title} />
						<span class="detail sizes">
							<span>{item.detail}</span>
							{#if item.received !== null && item.total}<span class="tabular">{formatBytes(item.received)} of {formatBytes(item.total)}</span>{/if}
						</span>
					{:else}
						<span class="detail" class:ok={item.state === 'done'}>{item.error ?? item.detail}</span>
					{/if}
				</div>
			{:else}
				<p class="none">No downloads yet. Engine downloads and updates appear here.</p>
			{/each}
		</div>
	{/if}
</div>

<style>
	.downloads {
		position: relative;
	}
	.trigger {
		position: relative;
		display: grid;
		place-items: center;
		width: 28px;
		height: 28px;
		padding: 0;
		border: 1px solid transparent;
		border-radius: 8px;
		background: transparent;
		color: var(--fg-muted);
	}
	.trigger:hover,
	.trigger.open {
		background: var(--hairline);
		color: var(--fg);
	}
	.trigger.busy {
		color: var(--accent);
	}
	.ring {
		position: absolute;
		inset: 1px;
		width: 24px;
		height: 24px;
		transform: rotate(-90deg);
	}
	.ring circle {
		fill: none;
		stroke-width: 2;
	}
	.ring .track {
		stroke: var(--hairline);
	}
	.ring .fill {
		stroke: var(--accent);
		stroke-linecap: round;
		transition: stroke-dashoffset 0.25s;
	}
	.ring .fill.spin {
		transform-origin: center;
		animation: spin 1.1s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	.arrow {
		display: grid;
		place-items: center;
	}
	.dot {
		position: absolute;
		top: 4px;
		right: 4px;
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--danger);
		border: 1.5px solid var(--content-bg);
	}
	.panel {
		position: absolute;
		top: calc(100% + 8px);
		right: 0;
		z-index: 50;
		width: 320px;
		max-width: calc(100vw - 24px);
		padding: 10px;
		display: grid;
		gap: 8px;
		border: 1px solid var(--hairline);
		border-radius: var(--radius);
		background: var(--dialog-bg);
		box-shadow: 0 12px 32px rgb(0 0 0 / 0.22);
	}
	.panel-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 2px 4px 0;
		font-size: var(--text-xs);
		font-weight: 600;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		color: var(--fg-muted);
	}
	.panel-head strong {
		font-weight: inherit;
	}
	.row {
		display: grid;
		gap: 8px;
		padding: 10px 12px;
		border: 1px solid var(--hairline);
		border-radius: var(--radius-sm);
		background: var(--surface);
	}
	.row.failed {
		border-color: color-mix(in srgb, var(--danger) 45%, var(--hairline));
	}
	.line {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}
	.title {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: var(--text-md);
		font-weight: 600;
	}
	.detail {
		font-size: var(--text-sm);
		color: var(--fg-muted);
	}
	.detail.sizes {
		display: flex;
		justify-content: space-between;
		gap: 8px;
	}
	.detail.ok {
		color: var(--ok-text);
	}
	.failed .detail {
		color: var(--danger);
	}
	.none {
		margin: 0;
		padding: 14px 10px 16px;
		font-size: 12px;
		line-height: 1.5;
		color: var(--fg-muted);
		text-align: center;
	}
</style>
