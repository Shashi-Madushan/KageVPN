<script lang="ts">
	import Icon, { type IconName } from './Icon.svelte';

	export type Page = 'home' | 'profiles' | 'usage' | 'routing' | 'connections' | 'diagnostics' | 'core' | 'logs' | 'settings';

	let { page, badges = {}, onselect }: { page: Page; badges?: Partial<Record<Page, string>>; onselect: (page: Page) => void } = $props();

	const sections: {
		title: string;
		items: { page: Page; label: string; icon: IconName }[];
	}[] = [
		{
			title: 'VPN',
			items: [
				{ page: 'home', label: 'Home', icon: 'home' },
				{ page: 'profiles', label: 'Servers', icon: 'profiles' },
				{ page: 'usage', label: 'Data Usage', icon: 'usage' }
			]
		},
		{
			title: 'Protection',
			items: [
				{ page: 'routing', label: 'Routing', icon: 'routing' },
				{ page: 'connections', label: 'Connections', icon: 'connections' },
				{ page: 'diagnostics', label: 'Diagnostics', icon: 'diagnostics' }
			]
		},
		{
			title: 'System',
			items: [
				{ page: 'core', label: 'Core Engine', icon: 'core' },
				{ page: 'logs', label: 'Logs', icon: 'logs' },
				{ page: 'settings', label: 'Settings', icon: 'settings' }
			]
		}
	];
</script>

<nav aria-label="Main Navigation">
	{#each sections as section (section.title)}
		<div class="section-group">
			<span class="section-title">{section.title}</span>
			{#each section.items as item (item.page)}
				<button class:active={page === item.page} onclick={() => onselect(item.page)}>
					<Icon name={item.icon} />
					<span>{item.label}</span>
					{#if badges[item.page]}<em>{badges[item.page]}</em>{/if}
				</button>
			{/each}
		</div>
	{/each}
</nav>

<style>
	nav {
		display: grid;
		gap: 12px;
		padding: 0 10px;
	}
	.section-group {
		display: grid;
		gap: 2px;
	}
	.section-title {
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--fg-muted);
		padding: 0 10px 4px;
	}
	button {
		display: flex;
		align-items: center;
		gap: 9px;
		border: 0;
		background: none;
		padding: 6px 10px;
		border-radius: 8px;
		color: var(--fg);
		text-align: left;
		font-size: 13px;
		transition: background 0.1s;
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
