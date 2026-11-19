<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { theme } from '$lib/theme.svelte';
	import { privacy } from '$lib/privacy.svelte';
	import { platform } from '$lib/platform.svelte';
	let { children } = $props();

	onMount(() => {
		theme.init();
		privacy.init();
		platform.init();
	});
</script>

{@render children()}
