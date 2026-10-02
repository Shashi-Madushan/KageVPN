<script lang="ts" generics="T extends string">
	type Option = { value: T; label: string; icon?: string; hint?: string };

	let {
		options,
		value = $bindable(),
		disabled = false,
		label,
		placeholder = 'Select…',
		grow = false,
		size = 'sm',
		onchange
	}: {
		options: Option[];
		value: T;
		disabled?: boolean;
		label: string;
		placeholder?: string;
		grow?: boolean;
		size?: 'sm' | 'lg';
		onchange?: (value: T) => void;
	} = $props();

	const MENU_MAX = 168;

	let open = $state(false);
	let trigger = $state<HTMLButtonElement>();
	let menu = $state<HTMLDivElement>();
	let focused = $state(0);
	let pos = $state({ left: 0, width: 0, top: 0, bottom: 0, up: false });

	const selected = $derived(options.find((o) => o.value === value));

	function show() {
		if (disabled || !trigger) return;
		const r = trigger.getBoundingClientRect();
		const room = window.innerHeight - r.bottom;
		const up = room < Math.min(MENU_MAX, options.length * 30 + 10) && r.top > room;
		pos = {
			left: r.left,
			width: r.width,
			top: r.bottom + 4,
			bottom: window.innerHeight - r.top + 4,
			up
		};
		focused = Math.max(0, options.findIndex((o) => o.value === value));
		open = true;
		queueMicrotask(() => menu?.querySelector<HTMLElement>('[data-focused="true"]')?.scrollIntoView({ block: 'nearest' }));
	}

	function close(refocus = true) {
		open = false;
		if (refocus) trigger?.focus();
	}

	function pick(option: Option) {
		close();
		if (option.value === value) return;
		value = option.value;
		onchange?.(option.value);
	}

	function move(delta: number) {
		focused = (focused + delta + options.length) % options.length;
		queueMicrotask(() => menu?.querySelector<HTMLElement>('[data-focused="true"]')?.scrollIntoView({ block: 'nearest' }));
	}

	function onkeydown(e: KeyboardEvent) {
		if (!open) {
			if (['ArrowDown', 'ArrowUp', 'Enter', ' '].includes(e.key)) {
				e.preventDefault();
				show();
			}
			return;
		}
		if (e.key === 'ArrowDown') (e.preventDefault(), move(1));
		else if (e.key === 'ArrowUp') (e.preventDefault(), move(-1));
		else if (e.key === 'Enter' || e.key === ' ') (e.preventDefault(), options[focused] && pick(options[focused]));
		else if (e.key === 'Escape') (e.preventDefault(), e.stopPropagation(), close());
		else if (e.key === 'Tab') close(false);
	}

	function onwindowpointer(e: PointerEvent) {
		if (!open) return;
		const t = e.target as Node;
		if (!trigger?.contains(t) && !menu?.contains(t)) close(false);
	}
</script>

<svelte:window onpointerdown={onwindowpointer} onresize={() => (open = false)} onblur={() => (open = false)} />

<span class="select" class:grow class:lg={size === 'lg'}>
<button
	type="button"
	class="select-trigger"
	class:open
	bind:this={trigger}
	{disabled}
	aria-label={label}
	aria-haspopup="listbox"
	aria-expanded={open}
	onclick={() => (open ? close() : show())}
	{onkeydown}
>
	{#if selected?.icon}<span class="s-icon">{selected.icon}</span>{/if}
	<span class="s-label" class:placeholder={!selected}>{selected?.label ?? placeholder}</span>
	<svg class="chev" viewBox="0 0 10 6" aria-hidden="true"><path d="M1 1l4 4 4-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" /></svg>
</button>
</span>

{#if open}
	<div
		class="select-menu"
		bind:this={menu}
		role="listbox"
		aria-label={label}
		style:left="{pos.left}px"
		style:min-width="{pos.width}px"
		style:max-width="{Math.max(pos.width, 240)}px"
		style:top={pos.up ? 'auto' : `${pos.top}px`}
		style:bottom={pos.up ? `${pos.bottom}px` : 'auto'}
		style:max-height="{MENU_MAX}px"
	>
		{#each options as option, i (option.value)}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<div
				class="s-option"
				role="option"
				tabindex="-1"
				aria-selected={option.value === value}
				data-focused={i === focused}
				class:focused={i === focused}
				class:selected={option.value === value}
				onpointermove={() => (focused = i)}
				onclick={() => pick(option)}
			>
				{#if option.icon}<span class="s-icon">{option.icon}</span>{/if}
				<span class="s-label">{option.label}</span>
				{#if option.hint}<span class="s-hint">{option.hint}</span>{/if}
				{#if option.value === value}
					<svg class="check" viewBox="0 0 12 10" aria-hidden="true"><path d="M1 5l3.5 3.5L11 1.5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
				{/if}
			</div>
		{/each}
	</div>
{/if}

<style>
	.select {
		display: inline-flex;
		min-width: 0;
	}
	.select.grow {
		flex: 1;
		width: 100%;
	}
	.lg .select-trigger {
		padding: 9px 12px;
		font-size: 13px;
		border-radius: 10px;
	}
	.lg .s-icon {
		font-size: 17px;
	}
	.select-trigger {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
		width: 100%;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		border-radius: 8px;
		padding: 5px 10px;
		font-size: 12px;
		font-weight: 400;
		color: var(--fg);
		text-align: left;
	}
	.select-trigger:hover:not(:disabled),
	.select-trigger.open {
		border-color: var(--accent);
	}
	.s-label {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.s-label.placeholder {
		color: var(--fg-muted);
	}
	.s-icon {
		flex: none;
	}
	.chev {
		width: 9px;
		height: 6px;
		flex: none;
		color: var(--fg-muted);
		transition: transform 0.15s;
	}
	.open .chev {
		transform: rotate(180deg);
	}
	.select-menu {
		position: fixed;
		z-index: 2000;
		overflow-y: auto;
		padding: 4px;
		border: 1px solid var(--hairline);
		border-radius: 9px;
		color: var(--fg);
		font-weight: 400;
		background: color-mix(in srgb, var(--surface-strong) 96%, transparent);
		backdrop-filter: blur(20px);
		-webkit-backdrop-filter: blur(20px);
		box-shadow: 0 8px 24px rgb(0 0 0 / 0.28);
	}
	.s-option {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 8px;
		border-radius: 6px;
		font-size: 11.5px;
		cursor: pointer;
	}
	.s-option.focused {
		background: var(--hairline);
	}
	.s-option.selected {
		color: var(--accent);
		font-weight: 600;
	}
	.s-hint {
		font-size: 10px;
		color: var(--fg-muted);
		flex: none;
	}
	.check {
		width: 11px;
		height: 9px;
		flex: none;
	}
</style>
