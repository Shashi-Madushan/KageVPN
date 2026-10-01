<script lang="ts" generics="T extends string">
	let {
		options,
		value,
		disabled = false,
		label,
		onchange
	}: {
		options: { value: T; label: string }[];
		value: T;
		disabled?: boolean;
		label: string;
		onchange: (value: T) => void;
	} = $props();
</script>

<div class="segmented" role="radiogroup" aria-label={label}>
	{#each options as option (option.value)}
		<button
			role="radio"
			aria-checked={value === option.value}
			class:active={value === option.value}
			{disabled}
			onclick={() => onchange(option.value)}
		>
			{option.label}
		</button>
	{/each}
</div>

<style>
	.segmented {
		display: flex;
		gap: 2px;
		padding: 2px;
		border-radius: 10px;
		background: var(--hairline);
	}
	button {
		flex: 1;
		border: 0;
		background: transparent;
		padding: 5px 10px;
		border-radius: 8px;
		color: var(--fg-muted);
		white-space: nowrap;
		transition:
			background 0.15s,
			color 0.15s;
	}
	button.active {
		background: var(--surface-strong);
		color: var(--fg);
		box-shadow: 0 1px 3px rgb(0 0 0 / 0.15);
	}
</style>
