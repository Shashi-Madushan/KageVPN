<script lang="ts">
	import { privacy } from '$lib/privacy.svelte';
	import Icon from './Icon.svelte';
</script>

<button
	type="button"
	class="eye"
	aria-label={privacy.hidden ? 'Show server and IP addresses' : 'Hide server and IP addresses'}
	aria-pressed={!privacy.hidden}
	title={privacy.hidden ? 'Show addresses' : 'Hide addresses'}
	onclick={() => privacy.toggle()}
>
	<Icon name={privacy.hidden ? 'eye-off' : 'eye'} size={15} />
</button>

<style>
	.eye {
		display: grid;
		place-items: center;
		width: 28px;
		height: 28px;
		padding: 0;
		border: 1px solid transparent;
		border-radius: 8px;
		background: transparent;
		color: var(--fg-muted);
	}
	.eye:hover {
		background: var(--hairline);
		color: var(--fg);
	}
</style>
