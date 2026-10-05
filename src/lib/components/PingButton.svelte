<script lang="ts">
	import { app } from '$lib/app.svelte';
	import { latencyLabel, latencyTone } from '$lib/format';
	import Icon from './Icon.svelte';

	const via = $derived(app.connected ? 'tunnel' : 'server');
	// Connected, the latency measured on connect is shown until the button is pressed.
	const check = $derived(
		app.check && app.checkVia === via
			? app.check
			: app.connected && app.delayMs !== null
				? { ms: app.delayMs }
				: undefined
	);
	const pending = $derived(!!check && 'pending' in check);
	const tone = $derived(latencyTone(check));
	const title = $derived(
		app.connected ? 'Check that traffic flows through the tunnel' : 'Check that the server answers'
	);
</script>

<button class="ping {tone}" disabled={pending || !app.activeProfile} {title} onclick={() => app.checkServer()}>
	<Icon name="bolt" size={12} />
	{check ? (pending ? 'Pinging…' : latencyLabel(check)) : 'Ping'}
</button>

<style>
	.ping {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		min-height: 24px;
		padding: 3px 10px;
		border: 1px solid var(--hairline);
		border-radius: 999px;
		background: var(--surface-strong);
		color: var(--fg);
		font-size: 11.5px;
		font-weight: 600;
	}
	.ping:hover:not(:disabled) {
		border-color: var(--accent);
	}
	.ping.ok {
		color: var(--ok-text);
	}
	.ping.warn {
		color: var(--warn);
	}
	.ping.bad {
		color: var(--danger);
	}
</style>
