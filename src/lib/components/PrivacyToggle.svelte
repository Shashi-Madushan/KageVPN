<script lang="ts">
	import { privacy } from '$lib/privacy.svelte';
	import Icon from './Icon.svelte';

	/** What this switch hides; see `privacyKey`. */
	let { id, label = 'address' }: { id: string; label?: string } = $props();
	const hidden = $derived(privacy.isHidden(id));
</script>

<button
	type="button"
	class="eye"
	aria-label={hidden ? `Show ${label}` : `Hide ${label}`}
	aria-pressed={!hidden}
	title={hidden ? `Show ${label}` : `Hide ${label}`}
	onclick={() => privacy.toggle(id)}
>
	<Icon name={hidden ? 'eye-off' : 'eye'} size={13} />
</button>

<style>
	.eye {
		display: grid;
		place-items: center;
		flex: none;
		width: 22px;
		height: 22px;
		padding: 0;
		border: 1px solid transparent;
		border-radius: 6px;
		background: transparent;
		color: var(--fg-muted);
	}
	.eye:hover {
		background: var(--hairline);
		color: var(--fg);
	}
</style>
