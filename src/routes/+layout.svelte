<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { theme } from '$lib/theme.svelte';
	import { privacy } from '$lib/privacy.svelte';
	let { children } = $props();

	onMount(() => {
		theme.init();
		privacy.init();
	});
</script>

{@render children()}
