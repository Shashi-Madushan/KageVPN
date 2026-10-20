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
			type="button"
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
	/* The chosen option has to stand out in both themes: an accent tint and ring, not just a slightly lighter grey. */
	button.active {
		background: color-mix(in srgb, var(--accent) 24%, var(--surface-strong));
		color: var(--fg);
		font-weight: 600;
		box-shadow:
			inset 0 0 0 1px color-mix(in srgb, var(--accent) 60%, transparent),
			0 1px 3px rgb(0 0 0 / 0.15);
	}
	button:not(.active):not(:disabled):hover {
		color: var(--fg);
	}
	button:disabled {
		opacity: 0.55;
	}
	button.active:disabled {
		opacity: 0.8;
	}
</style>
