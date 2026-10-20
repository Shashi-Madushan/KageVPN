<script lang="ts" module>
	export type { Page } from '$lib/pages';
	export { pageOrder } from '$lib/pages';
</script>

<script lang="ts">
	import Icon, { type IconName } from './Icon.svelte';
	import { shortcutFor, type Page } from '$lib/pages';

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
				{ page: 'usage', label: 'Data usage', icon: 'usage' }
			]
		},
		{
			title: 'Protection',
			items: [
				{ page: 'routing', label: 'Routing', icon: 'routing' },
				{ page: 'apps', label: 'Apps', icon: 'apps' },
				{ page: 'connections', label: 'Connections', icon: 'connections' },
				{ page: 'diagnostics', label: 'Diagnostics', icon: 'diagnostics' }
			]
		},
		{
			title: 'System',
			items: [
				{ page: 'core', label: 'Engine', icon: 'core' },
				{ page: 'logs', label: 'Logs', icon: 'logs' },
				{ page: 'settings', label: 'Settings', icon: 'settings' }
			]
		}
	];

	function navKeydown(e: KeyboardEvent) {
		if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return;
		const items = [...(e.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>('button')];
		const i = items.indexOf(document.activeElement as HTMLButtonElement);
		if (i < 0) return;
		e.preventDefault();
		items[(i + (e.key === 'ArrowDown' ? 1 : -1) + items.length) % items.length].focus();
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<nav aria-label="Main Navigation" onkeydown={navKeydown}>
	{#each sections as section (section.title)}
		<div class="section-group">
			<span class="section-title">{section.title}</span>
			{#each section.items as item (item.page)}
				<button class:active={page === item.page} title={shortcutFor(item.page) ? `${item.label} (⌘${shortcutFor(item.page)})` : item.label} aria-label={item.label} aria-current={page === item.page ? 'page' : undefined} onclick={() => onselect(item.page)}>
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
		font-size: 11px;
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
	@media (max-width: 759px) {
		nav {
			padding: 0 8px;
			gap: 8px;
		}
		.section-title,
		button span {
			display: none;
		}
		.section-group + .section-group {
			border-top: 1px solid var(--hairline);
			padding-top: 8px;
		}
		button {
			position: relative;
			justify-content: center;
			padding: 9px 0;
		}
		em {
			position: absolute;
			top: 4px;
			right: 6px;
			width: 8px;
			height: 8px;
			padding: 0;
			font-size: 0;
		}
	}
	em {
		margin-left: auto;
		font-style: normal;
		font-size: 11px;
		font-weight: 700;
		padding: 1px 6px;
		border-radius: 999px;
		background: var(--accent);
		color: white;
	}
</style>
