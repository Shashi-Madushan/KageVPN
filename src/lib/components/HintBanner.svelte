<script lang="ts">
	import { app } from '$lib/app.svelte';
</script>

{#if app.hint?.kind === 'keyMismatch'}
	{@const key = app.hint.key}
	<div class="banner card">
		<span>The server presented a different certificate key than this profile trusts.</span>
		<button onclick={() => app.trustHintedKey(key)}>Trust &amp; reconnect</button>
	</div>
{:else if app.hint?.kind === 'unresolved'}
	<div class="banner card">The server hostname does not resolve. Check the link with its owner.</div>
{/if}

<style>
	.banner {
		display: grid;
		gap: 8px;
		padding: 10px 12px;
		border-color: var(--warn);
		background: color-mix(in srgb, var(--warn) 12%, var(--surface));
		font-size: 12px;
	}
	button {
		justify-self: start;
		border: 0;
		border-radius: 8px;
		padding: 5px 12px;
		background: var(--warn);
		color: #1d1d1f;
		font-weight: 600;
	}
</style>
