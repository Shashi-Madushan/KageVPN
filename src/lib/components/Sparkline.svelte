<script lang="ts">
	import type { Sample } from '$lib/api';

	let {
		samples,
		height = 56
	}: { samples: Sample[]; length?: number; height?: number } = $props();

	const WIDTH = 240;
	const PAD = 3;

	const peak = $derived(Math.max(1024, ...samples.map((s) => Math.max(s.up, s.down))));

	function points(pick: (s: Sample) => number): string {
		if (samples.length <= 1) return `0,${height - PAD} ${WIDTH},${height - PAD}`;
		const denom = Math.max(1, samples.length - 1);
		return samples
			.map((s, i) => {
				const x = (i / denom) * WIDTH;
				const y = height - PAD - (pick(s) / peak) * (height - PAD * 2);
				return `${x.toFixed(1)},${y.toFixed(1)}`;
			})
			.join(' ');
	}

	const down = $derived(points((s) => s.down));
	const up = $derived(points((s) => s.up));
	const area = $derived(
		samples.length > 1
			? `0,${height} ${down} ${WIDTH},${height}`
			: ''
	);
</script>

<svg viewBox="0 0 {WIDTH} {height}" preserveAspectRatio="none" style:height="{height}px" aria-hidden="true">
	{#if samples.length > 1}
		<polygon points={area} class="area" />
		<polyline points={down} class="line down" />
		<polyline points={up} class="line up" />
	{:else}
		<line x1="0" x2={WIDTH} y1={height - PAD} y2={height - PAD} class="flat" />
	{/if}
</svg>

<style>
	svg {
		width: 100%;
		display: block;
	}
	.line {
		fill: none;
		stroke-width: 1.6;
		stroke-linejoin: round;
		stroke-linecap: round;
		vector-effect: non-scaling-stroke;
	}
	.down {
		stroke: var(--down);
	}
	.up {
		stroke: var(--up);
	}
	.area {
		fill: color-mix(in srgb, var(--down) 16%, transparent);
	}
	.flat {
		stroke: var(--hairline);
		stroke-width: 1;
		vector-effect: non-scaling-stroke;
	}
</style>
