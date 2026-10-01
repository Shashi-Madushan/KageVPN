<script lang="ts">
	let { values, width = 56, height = 16 }: { values: (number | null)[]; width?: number; height?: number } = $props();

	const valid = $derived(values.filter((v): v is number => v !== null));
	const peak = $derived(Math.max(100, ...valid));
	const points = $derived(
		values
			.map((v, i) => {
				if (v === null) return null;
				const x = values.length > 1 ? (i / (values.length - 1)) * width : width / 2;
				const y = height - 2 - (Math.min(v, peak) / peak) * (height - 4);
				return `${x.toFixed(1)},${y.toFixed(1)}`;
			})
			.filter((p): p is string => p !== null)
			.join(' ')
	);
</script>

{#if values.length > 1}
	<svg {width} {height} viewBox="0 0 {width} {height}" aria-hidden="true">
		<polyline {points} fill="none" stroke="var(--fg-muted)" stroke-width="1.2" stroke-linejoin="round" stroke-linecap="round" />
	</svg>
{/if}
