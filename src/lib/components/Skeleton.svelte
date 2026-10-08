<script lang="ts">
	let {
		width = '100%',
		height = '14px',
		radius = '6px'
	}: { width?: string; height?: string; radius?: string } = $props();
</script>

<span class="sk" style:width style:height style:border-radius={radius} aria-hidden="true"></span>

<style>
	.sk {
		display: block;
		max-width: 100%;
		background: linear-gradient(90deg, var(--hairline) 25%, color-mix(in srgb, var(--hairline) 55%, transparent) 50%, var(--hairline) 75%);
		background-size: 200% 100%;
		animation: shimmer 1.3s linear infinite;
	}
	@keyframes shimmer {
		to {
			background-position: -200% 0;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.sk {
			animation: none;
		}
	}
</style>
