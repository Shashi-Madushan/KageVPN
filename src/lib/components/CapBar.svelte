<script lang="ts">
	import type { Tone } from '$lib/capView';

	/** A data cap's progress bar: blue, amber near the limit, red when over. */
	let { percent, tone = 'ok', label, height = 6 }: { percent: number; tone?: Tone; label: string; height?: number } = $props();

	const width = $derived(Math.max(percent > 0 ? 3 : 0, Math.min(100, percent)));
</script>

<span class="bar {tone}" style="height:{height}px" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.min(100, Math.round(percent))} aria-label={label}>
	<i style="width:{width}%"></i>
</span>

<style>
	.bar {
		display: block;
		min-width: 60px;
		border-radius: 999px;
		background: color-mix(in srgb, var(--fg) 16%, transparent);
		overflow: hidden;
	}
	i {
		display: block;
		height: 100%;
		border-radius: 999px;
		background: var(--accent);
		transition: width 0.4s ease;
	}
	.warn i {
		background: var(--warn);
	}
	.over i {
		background: var(--danger);
	}
	.off i {
		background: var(--fg-muted);
	}
</style>
