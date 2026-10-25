<script lang="ts" module>
	import { api } from '$lib/api';

	/** Icons already asked for, by bundle path, so each is read once per window. */
	const cache = new Map<string, Promise<string | null>>();
	function iconFor(path: string): Promise<string | null> {
		let found = cache.get(path);
		if (!found) {
			found = api.appIcon(path).catch(() => null);
			cache.set(path, found);
			// Only a found icon is kept; a miss or error is asked again next time the row shows.
			found.then((u) => {
				if (!u && cache.get(path) === found) cache.delete(path);
			});
		}
		return found;
	}
</script>

<script lang="ts">
	/** An installed app's real icon, loaded when it scrolls into view; its initial until then or without one. */
	let { path, name, size = 28 }: { path: string; name: string; size?: number } = $props();

	let url = $state<string | null>(null);

	/** Asks for the icon each time the row scrolls into view until one is found. */
	function visible(node: HTMLElement) {
		if (!path) return;
		let asked = 0;
		const observer = new IntersectionObserver((entries) => {
			if (url || !entries.some((e) => e.isIntersecting)) return;
			const ticket = ++asked;
			iconFor(path).then((u) => {
				if (ticket === asked) url = u;
			});
		});
		observer.observe(node);
		return { destroy: () => observer.disconnect() };
	}
</script>

<span class="icon" style="width:{size}px;height:{size}px;border-radius:{Math.round(size / 4)}px;font-size:{Math.round(size * 0.46)}px" aria-hidden="true" use:visible>
	{#if url}<img src={url} alt="" onerror={() => { cache.delete(path); url = null; }} />{:else}{name.charAt(0).toUpperCase()}{/if}
</span>

<style>
	.icon {
		flex: none;
		display: grid;
		place-items: center;
		background: var(--hairline);
		color: var(--fg-muted);
		font-weight: 700;
		overflow: hidden;
	}
	img {
		width: 100%;
		height: 100%;
		object-fit: contain;
	}
</style>
