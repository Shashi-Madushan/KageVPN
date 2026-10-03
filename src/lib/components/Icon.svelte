<script lang="ts">
	export type IconName = 'home' | 'profiles' | 'usage' | 'routing' | 'connections' | 'diagnostics' | 'core' | 'logs' | 'settings' | 'shield' | 'bolt' | 'refresh' | 'sun' | 'moon' | 'monitor';

	let { name, size = 16 }: { name: IconName; size?: number } = $props();

	const paths: Record<IconName, string> = {
		home: 'M3 11.5 12 4l9 7.5M5.5 10v9h13v-9',
		profiles: 'M4 7h16M4 12h16M4 17h10',
		usage: 'M5 20V10M12 20V4M19 20v-7',
		routing: 'M6 4v6a4 4 0 0 0 4 4h4a4 4 0 0 1 4 4v2M6 4 3 7M6 4l3 3M18 20l-3-3M18 20l3-3',
		connections: 'M4 12h6M14 12h6M10 12a2 2 0 1 0 4 0 2 2 0 0 0-4 0ZM4 6h4M16 6h4M4 18h4M16 18h4',
		diagnostics: 'M3 12h4l2-6 4 12 2-6h6',
		core: 'M12 3 4 7.5v9L12 21l8-4.5v-9L12 3Zm0 9 8-4.5M12 12v9M12 12 4 7.5',
		logs: 'M5 4h14v16H5zM9 9h6M9 13h6M9 17h3',
		shield: 'M12 3l7 2.6v5.6c0 4.4-3 7.6-7 9.8-4-2.2-7-5.4-7-9.8V5.6L12 3Zm-3 9 2.2 2.2L15.2 10',
		bolt: 'M13 2 5 13.5h6L10 22l9-12h-6l0-8Z',
		refresh: 'M20 12a8 8 0 1 1-2.5-5.8M20 4v4h-4',
		sun: 'M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8Zm0-13v2m0 14v2M3 12h2m14 0h2M5.6 5.6 7 7m10 10 1.4 1.4M5.6 18.4 7 17M17 7l1.4-1.4',
		moon: 'M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5Z',
		monitor: 'M4 5h16v11H4zM9 20h6M12 16v4',
		settings: 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm7.4-3a7.4 7.4 0 0 0-.1-1.3l2-1.5-2-3.4-2.3 1a7.4 7.4 0 0 0-2.2-1.3L14.4 3h-4l-.4 2.5a7.4 7.4 0 0 0-2.2 1.3l-2.3-1-2 3.4 2 1.5a7.4 7.4 0 0 0 0 2.6l-2 1.5 2 3.4 2.3-1a7.4 7.4 0 0 0 2.2 1.3l.4 2.5h4l.4-2.5a7.4 7.4 0 0 0 2.2-1.3l2.3 1 2-3.4-2-1.5c.1-.4.1-.9.1-1.3Z'
	};
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
	<path d={paths[name]} />
</svg>
