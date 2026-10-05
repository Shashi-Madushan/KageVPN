<script lang="ts">
	import type { Snippet } from 'svelte';
	import { privacy } from '$lib/privacy.svelte';

	let { id, children }: { id: string; children: Snippet } = $props();
	const hidden = $derived(privacy.isHidden(id));
</script>

<!-- Addresses stay in the page but blurred, so layout does not jump when they are shown. -->
<span class="sensitive" class:hidden aria-hidden={hidden}>{@render children()}</span>
{#if hidden}<span class="sr-only">Hidden</span>{/if}

<style>
	.hidden {
		filter: blur(6px);
		user-select: none;
		pointer-events: none;
	}
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip-path: inset(50%);
		white-space: nowrap;
	}
</style>
