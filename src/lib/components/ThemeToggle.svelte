<script lang="ts">
	import { theme, type ThemeMode } from '$lib/theme.svelte';
	import Icon, { type IconName } from './Icon.svelte';

	const options: { value: ThemeMode; label: string; icon: IconName }[] = [
		{ value: 'system', label: 'System', icon: 'monitor' },
		{ value: 'light', label: 'Light', icon: 'sun' },
		{ value: 'dark', label: 'Dark', icon: 'moon' }
	];

	function onkeydown(event: KeyboardEvent) {
		const step = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[event.key];
		if (!step) return;
		event.preventDefault();
		const index = options.findIndex((o) => o.value === theme.mode);
		const next = options[(index + step + options.length) % options.length];
		theme.set(next.value);
		(event.currentTarget as HTMLElement).querySelector<HTMLButtonElement>(`[data-value="${next.value}"]`)?.focus();
	}

	const current = $derived(options.find((o) => o.value === theme.mode) ?? options[0]);

	/** The narrow icon rail has no room for three buttons, so one button steps through them. */
	function cycle() {
		const index = options.findIndex((o) => o.value === theme.mode);
		theme.set(options[(index + 1) % options.length].value);
	}
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<div class="toggle" role="radiogroup" aria-label="Appearance" {onkeydown}>
	{#each options as option (option.value)}
		<button
			type="button"
			role="radio"
			data-value={option.value}
			aria-checked={theme.mode === option.value}
			aria-label={option.label}
			title={option.label === 'System' ? 'System appearance' : `${option.label} appearance`}
			class:active={theme.mode === option.value}
			tabindex={theme.mode === option.value ? 0 : -1}
			onclick={() => theme.set(option.value)}
		>
			<Icon name={option.icon} size={15} />
		</button>
	{/each}
</div>

<button type="button" class="cycle" aria-label="Appearance: {current.label}. Click to change." title="Appearance: {current.label}" onclick={cycle}>
	<Icon name={current.icon} size={16} />
</button>

<style>
	.toggle {
		display: inline-flex;
		gap: 2px;
		padding: 2px;
		border-radius: 9px;
		background: var(--hairline);
	}
	button {
		display: grid;
		place-items: center;
		width: 32px;
		height: 26px;
		padding: 0;
		border: 0;
		border-radius: 7px;
		background: transparent;
		color: var(--fg-muted);
		transition:
			background 0.15s,
			color 0.15s;
	}
	button:hover {
		color: var(--fg);
	}
	button.active {
		background: var(--surface-strong);
		color: var(--fg);
		box-shadow: 0 1px 3px rgb(0 0 0 / 0.18);
	}
	.cycle {
		display: none;
		place-items: center;
		width: 32px;
		height: 30px;
		padding: 0;
		border: 0;
		border-radius: 8px;
		background: var(--hairline);
		color: var(--fg);
	}
	@media (max-width: 759px) {
		.toggle {
			display: none;
		}
		.cycle {
			display: grid;
		}
	}
</style>
