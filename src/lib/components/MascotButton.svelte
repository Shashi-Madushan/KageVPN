<script lang="ts">
	import { app } from '$lib/app.svelte';
	import Mascot from './Mascot.svelte';

	let { size = 120 }: { size?: number } = $props();

	const connecting = $derived(app.status.state === 'connecting');
	const tone = $derived(app.tunnelDown ? 'down' : app.verifying ? 'checking' : app.protectedNow ? 'ok' : 'off');
	const mood = $derived(tone === 'ok' ? 'happy' : tone === 'down' ? 'worried' : tone === 'checking' || connecting ? 'busy' : 'sleep');
	const label = $derived(app.busy ? 'Cancel connecting' : app.connected ? 'Disconnect from VPN' : 'Connect to VPN');
</script>

<button
	class="mascot-btn {tone}"
	disabled={!app.busy && !app.activeProfile}
	onclick={() => app.toggle()}
	aria-label={label}
	title={label}
	aria-busy={connecting}
>
	<Mascot {mood} {size} />
</button>

<style>
	.mascot-btn {
		--c: var(--fg-muted);
		display: grid;
		place-items: center;
		padding: 6px 14px 0;
		border: 0;
		border-radius: 24px;
		background: radial-gradient(closest-side, color-mix(in srgb, var(--c) 16%, transparent), transparent);
		transition: transform 0.12s;
	}
	.ok {
		--c: var(--ok);
	}
	.checking {
		--c: var(--warn);
	}
	.down {
		--c: var(--danger);
	}
	.mascot-btn:hover:not(:disabled) {
		transform: scale(1.03);
	}
	.mascot-btn:active:not(:disabled) {
		transform: scale(0.97);
	}
	.mascot-btn:disabled {
		opacity: 0.7;
	}
</style>
