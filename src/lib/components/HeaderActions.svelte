<script lang="ts">
	import type { Snippet } from 'svelte';
	import { header } from '$lib/header.svelte';

	let { children }: { children: Snippet } = $props();

	$effect(() => {
		header.actions = children;
		return () => (header.actions = null);
	});
</script>
