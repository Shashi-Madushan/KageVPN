<script lang="ts">
	/** A determinate bar for 0-100, or a sliding one when the size is unknown (value null). */
	let { value, label = 'Progress' }: { value: number | null; label?: string } = $props();
</script>

<div class="track" role="progressbar" aria-label={label} aria-valuemin={0} aria-valuemax={100} aria-valuenow={value === null ? undefined : Math.round(value)}>
	<div class="fill" class:indeterminate={value === null} style:width={value === null ? '40%' : `${Math.max(2, Math.min(100, value))}%`}></div>
</div>

<style>
	.track {
		position: relative;
		height: 8px;
		border-radius: 999px;
		background: var(--hairline);
		overflow: hidden;
	}
	.fill {
		height: 100%;
		border-radius: 999px;
		background: linear-gradient(90deg, color-mix(in srgb, var(--accent) 80%, white), var(--accent));
		transition: width 0.25s ease;
	}
	.fill.indeterminate {
		animation: slide 1.2s ease-in-out infinite;
	}
	@keyframes slide {
		from {
			transform: translateX(-100%);
		}
		to {
			transform: translateX(250%);
		}
	}
</style>
