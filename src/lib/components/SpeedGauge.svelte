<script lang="ts">
	import type { SpeedPhase } from '$lib/api';

	let {
		phase,
		mbps,
		samples,
		pingMs = null,
		final = null
	}: {
		/** `idle` before a test, then the phase being measured. */
		phase: 'idle' | SpeedPhase;
		mbps: number;
		/** Readings from the current phase, drawn as a live graph under the dial. */
		samples: number[];
		pingMs?: number | null;
		/** Download result shown on the dial once the test is complete. */
		final?: number | null;
	} = $props();

	// The dial is non-linear (like most speed tests) so both 5 Mbps and 500 Mbps read clearly.
	const SCALE = [0, 5, 10, 25, 50, 100, 250, 500, 1000];
	const CX = 150;
	const CY = 140;
	const R = 108;
	const START = 135;
	const SWEEP = 270;

	function fraction(value: number): number {
		const v = Math.max(0, value);
		if (v >= SCALE[SCALE.length - 1]) return 1;
		const i = SCALE.findIndex((tick, n) => v >= tick && v < SCALE[n + 1]);
		return (i + (v - SCALE[i]) / (SCALE[i + 1] - SCALE[i])) / (SCALE.length - 1);
	}

	const point = (angle: number, radius: number) => {
		const rad = (angle * Math.PI) / 180;
		return { x: CX + radius * Math.cos(rad), y: CY + radius * Math.sin(rad) };
	};

	const from = point(START, R);
	const to = point(START + SWEEP, R);
	const arc = `M ${from.x.toFixed(1)} ${from.y.toFixed(1)} A ${R} ${R} 0 1 1 ${to.x.toFixed(1)} ${to.y.toFixed(1)}`;
	const ticks = SCALE.map((value, i) => {
		const angle = START + (i / (SCALE.length - 1)) * SWEEP;
		return { value, label: point(angle, R - 26), mark: [point(angle, R - 9), point(angle, R - 15)] };
	});

	const finished = $derived(phase === 'done' && final !== null);
	const showsSpeed = $derived(phase === 'download' || phase === 'upload' || finished);
	const value = $derived(finished ? (final ?? 0) : mbps);
	const frac = $derived(showsSpeed ? fraction(value) : 0);
	const angle = $derived(START + frac * SWEEP);
	const color = $derived(phase === 'upload' ? 'var(--up)' : phase === 'ping' ? 'var(--accent)' : 'var(--down)');
	const label = $derived(
		finished ? 'Download' : { idle: 'Ready', ping: 'Ping', download: 'Download', upload: 'Upload', done: 'Complete' }[phase]
	);
	const idle = $derived(!showsSpeed && phase !== 'ping');
	const big = $derived(
		phase === 'ping'
			? pingMs === null
				? '0'
				: pingMs.toFixed(0)
			: showsSpeed
				? value < 10
					? value.toFixed(1)
					: value.toFixed(0)
				: '0'
	);
	const unit = $derived(phase === 'ping' ? 'ms' : 'Mbps');

	const peak = $derived(Math.max(1, ...samples));
	const graph = $derived(
		samples.length > 1
			? samples
					.map((v, i) => `${((i / (samples.length - 1)) * 300).toFixed(1)},${(44 - 3 - (v / peak) * 38).toFixed(1)}`)
					.join(' ')
			: ''
	);
</script>

<div class="gauge">
	<svg viewBox="0 0 300 240" role="img" aria-label="{label}{showsSpeed ? `: ${big} Mbps` : phase === 'ping' ? `: ${big} ms` : ''}">
		<path d={arc} class="track" pathLength="100" />
		<path
			d={arc}
			class="value"
			pathLength="100"
			stroke={color}
			stroke-dasharray="{frac * 100} 100"
			opacity={frac > 0.004 ? 1 : 0}
		/>
		{#each ticks as tick (tick.value)}
			<line x1={tick.mark[0].x} y1={tick.mark[0].y} x2={tick.mark[1].x} y2={tick.mark[1].y} class="mark" />
			<text x={tick.label.x} y={tick.label.y} class="tick">{tick.value}</text>
		{/each}
		<g class="needle" style:transform="rotate({angle}deg)" style:opacity={showsSpeed ? 1 : 0.35}>
			<line x1={CX} y1={CY} x2={CX + R - 36} y2={CY} stroke={color} />
		</g>
		<circle cx={CX} cy={CY} r="8" class="hub" />
		<circle cx={CX} cy={CY} r="3" fill={color} />

		<text x={CX} y="190" class="big" class:idle>{big}</text>
		<text x={CX} y="210" class="unit">{unit}</text>
		<text x={CX} y="232" class="phase">{label}</text>
	</svg>

	<svg class="graph" viewBox="0 0 300 44" preserveAspectRatio="none" aria-hidden="true">
		<line x1="0" y1="41" x2="300" y2="41" class="baseline" />
		{#if graph}
			<polygon points="0,44 {graph} 300,44" fill={color} opacity="0.14" />
			<polyline points={graph} fill="none" stroke={color} stroke-width="1.6" stroke-linejoin="round" vector-effect="non-scaling-stroke" />
		{/if}
	</svg>
</div>

<style>
	.gauge {
		display: grid;
		justify-items: center;
		gap: 4px;
	}
	svg {
		width: min(100%, 320px);
		height: auto;
		overflow: visible;
	}
	.track,
	.value {
		fill: none;
		stroke-width: 13;
		stroke-linecap: round;
	}
	.track {
		stroke: var(--hairline);
	}
	.value {
		transition:
			stroke-dasharray 0.2s linear,
			opacity 0.2s,
			stroke 0.3s;
	}
	.mark {
		stroke: var(--fg-muted);
		stroke-width: 1.2;
		opacity: 0.5;
	}
	.tick {
		fill: var(--fg-muted);
		font-size: 9.5px;
		text-anchor: middle;
		dominant-baseline: middle;
		font-variant-numeric: tabular-nums;
	}
	.needle {
		transform-origin: 150px 140px;
		transition:
			transform 0.2s linear,
			opacity 0.3s;
	}
	.needle line {
		stroke-width: 3;
		stroke-linecap: round;
	}
	.hub {
		fill: var(--dialog-bg, var(--surface-strong));
		stroke: var(--hairline);
		stroke-width: 2;
	}
	.phase {
		fill: var(--fg-muted);
		font-size: 11px;
		font-weight: 650;
		letter-spacing: 0.08em;
		text-anchor: middle;
		text-transform: uppercase;
	}
	.big {
		fill: var(--fg);
		font-size: 40px;
		font-weight: 650;
		letter-spacing: -0.02em;
		text-anchor: middle;
		font-variant-numeric: tabular-nums;
	}
	.big.idle {
		fill: var(--fg-muted);
		opacity: 0.55;
	}
	.unit {
		fill: var(--fg-muted);
		font-size: 12px;
		text-anchor: middle;
	}
	.graph {
		width: min(100%, 320px);
		height: 44px;
	}
	.baseline {
		stroke: var(--hairline);
		stroke-width: 1;
	}
	@media (prefers-reduced-motion: reduce) {
		.value,
		.needle {
			transition: none;
		}
	}
</style>
