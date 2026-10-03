<script lang="ts">
	let { height = 14 }: { height?: number } = $props();
</script>

<!-- Redrawn as strokes so the letters follow the text colour in any theme. The dots keep the brand greens. -->
<svg class="logo" viewBox="0 0 520 132" style:height="{height}px" role="img" aria-label="Apzur" fill="none">
	<g stroke="currentColor" stroke-width="20" stroke-linecap="round" stroke-linejoin="round">
		<path d="M14 112 L54 24 L94 112" />
		<path d="M136 112 V24 H172 Q202 24 202 52 Q202 80 172 80 H136" />
		<path d="M232 24 H298 L232 112 H300" />
		<path d="M334 24 V72 Q334 112 372 112 Q410 112 410 72 V24" />
		<path d="M444 112 V24 H480 Q510 24 510 52 Q510 80 480 80 H444 M478 82 L508 112" />
	</g>
	<g stroke="none">
		<circle cx="112" cy="118" r="9" fill="#2F8F4E" />
		<circle cx="166" cy="52" r="8" fill="#8ABB50" />
		<circle cx="318" cy="118" r="9" fill="#2F8F4E" />
		<circle cx="414" cy="14" r="9" fill="#8ABB50" />
		<circle cx="512" cy="118" r="8" fill="#8ABB50" />
	</g>
</svg>

<style>
	.logo {
		display: block;
		width: auto;
		overflow: visible;
	}
</style>
