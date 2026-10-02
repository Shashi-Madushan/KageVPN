<script lang="ts">
	import type { Bucket, Point } from '$lib/api';
	import { bucketLabel, shortLabel } from '$lib/dates';
	import { formatBytes } from '$lib/format';

	let { points, bucket, onselect }: { points: Point[]; bucket: Bucket; onselect?: (point: Point) => void } = $props();

	let hovered = $state<number | null>(null);

	const peak = $derived(Math.max(1, ...points.map((p) => p.up + p.down)));
	const labelEvery = $derived(Math.max(1, Math.ceil(points.length / 6)));
	const active = $derived(hovered !== null ? points[hovered] : null);
</script>

<div class="chart">
	<span class="ymax tabular">{formatBytes(peak)}</span>
	<div class="plot" role="img" aria-label="Data usage over time" onmouseleave={() => (hovered = null)}>
		{#each points as point, i (point.start)}
			<button
				class="col"
				class:hot={hovered === i}
				onmouseenter={() => (hovered = i)}
				onfocus={() => (hovered = i)}
				onclick={() => onselect?.(point)}
				aria-label="{bucketLabel(point.start, bucket)}: {formatBytes(point.up + point.down)}"
			>
				<span class="up" style:height="{(point.up / peak) * 100}%"></span>
				<span class="down" style:height="{(point.down / peak) * 100}%"></span>
			</button>
		{/each}
		{#if active && hovered !== null}
			<div class="tip" style:left="{((hovered + 0.5) / points.length) * 100}%" class:flip={hovered > points.length * 0.7}>
				<strong>{bucketLabel(active.start, bucket)}</strong>
				<span class="tabular"><i class="dot down-dot"></i>{formatBytes(active.down)} down</span>
				<span class="tabular"><i class="dot up-dot"></i>{formatBytes(active.up)} up</span>
				<span class="tabular total">{formatBytes(active.up + active.down)} total</span>
			</div>
		{/if}
	</div>
	<div class="axis">
		{#each points as point, i (point.start)}
			<span style:left="{((i + 0.5) / points.length) * 100}%">{i % labelEvery === 0 ? shortLabel(point.start, bucket) : ''}</span>
		{/each}
	</div>
</div>

<style>
	.chart {
		position: relative;
		padding-top: 4px;
	}
	.ymax {
		position: absolute;
		top: -4px;
		left: 0;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.plot {
		position: relative;
		height: 170px;
		display: flex;
		align-items: stretch;
		gap: 2px;
		margin-top: 14px;
		border-bottom: 1px solid var(--hairline);
		background:
			linear-gradient(var(--hairline), var(--hairline)) 0 0 / 100% 1px no-repeat,
			linear-gradient(var(--hairline), var(--hairline)) 0 50% / 100% 1px no-repeat;
	}
	.col {
		flex: 1;
		min-width: 1px;
		border: 0;
		padding: 0;
		background: none;
		display: flex;
		flex-direction: column-reverse;
		justify-content: flex-start;
		border-radius: 3px 3px 0 0;
		cursor: pointer;
	}
	.col.hot {
		background: color-mix(in srgb, var(--accent) 12%, transparent);
	}
	.down,
	.up {
		display: block;
		width: 100%;
		min-height: 0;
	}
	.down {
		background: var(--down);
	}
	.up {
		background: var(--up);
		border-radius: 2px 2px 0 0;
		order: 1;
	}
	.tip {
		position: absolute;
		top: 0;
		transform: translateX(-50%);
		z-index: 5;
		display: grid;
		gap: 1px;
		padding: 7px 10px;
		border-radius: 8px;
		background: var(--surface-strong);
		border: 1px solid var(--hairline);
		box-shadow: 0 6px 20px rgb(0 0 0 / 0.25);
		font-size: 11px;
		white-space: nowrap;
		pointer-events: none;
	}
	.tip.flip {
		transform: translateX(-100%);
	}
	.tip .total {
		color: var(--fg-muted);
	}
	.dot {
		display: inline-block;
		width: 7px;
		height: 7px;
		border-radius: 50%;
		margin-right: 5px;
	}
	.down-dot {
		background: var(--down);
	}
	.up-dot {
		background: var(--up);
	}
	.axis {
		position: relative;
		height: 16px;
		margin-top: 4px;
	}
	.axis span {
		position: absolute;
		transform: translateX(-50%);
		font-size: 11px;
		color: var(--fg-muted);
		white-space: nowrap;
	}
</style>
