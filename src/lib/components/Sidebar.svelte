<script lang="ts">
	import Icon, { type IconName } from './Icon.svelte';

	export type Page = 'home' | 'profiles' | 'usage' | 'routing' | 'connections' | 'diagnostics' | 'core' | 'logs' | 'settings';

	let { page, badges = {}, onselect }: { page: Page; badges?: Partial<Record<Page, string>>; onselect: (page: Page) => void } = $props();

	const items: { page: Page; label: string; icon: IconName }[] = [
		{ page: 'home', label: 'Home', icon: 'home' },
		{ page: 'profiles', label: 'Profiles', icon: 'profiles' },
		{ page: 'usage', label: 'Usage', icon: 'usage' },
		{ page: 'routing', label: 'Routing', icon: 'routing' },
		{ page: 'connections', label: 'Connections', icon: 'connections' },
		{ page: 'diagnostics', label: 'Diagnostics', icon: 'diagnostics' },
		{ page: 'logs', label: 'Logs', icon: 'logs' },
		{ page: 'core', label: 'Core', icon: 'core' },
		{ page: 'settings', label: 'Settings', icon: 'settings' }
	];
</script>

<nav aria-label="Sections">
	{#each items as item (item.page)}
		<button class:active={page === item.page} onclick={() => onselect(item.page)}>
			<Icon name={item.icon} />
			<span>{item.label}</span>
			{#if badges[item.page]}<em>{badges[item.page]}</em>{/if}
		</button>
	{/each}
</nav>

<style>
	nav {
		display: grid;
		gap: 2px;
		padding: 0 10px;
	}
	button {
		display: flex;
		align-items: center;
		gap: 9px;
		border: 0;
		background: none;
		padding: 7px 10px;
		border-radius: 8px;
		color: var(--fg);
		text-align: left;
	}
	button:hover {
		background: var(--hairline);
	}
	button.active {
		background: color-mix(in srgb, var(--accent) 18%, transparent);
		color: var(--accent);
		font-weight: 600;
	}
	em {
		margin-left: auto;
		font-style: normal;
		font-size: 10px;
		font-weight: 700;
		padding: 1px 6px;
		border-radius: 999px;
		background: var(--accent);
		color: white;
	}
</style>
