<script lang="ts">
	import type { ConnState } from '$lib/api';

	let {
		state,
		disabled = false,
		size = 104,
		onclick
	}: { state: ConnState; disabled?: boolean; size?: number; onclick: () => void } = $props();

	const labels: Record<ConnState, string> = {
		disconnected: 'Connect',
		connecting: 'Connecting',
		connected: 'Disconnect'
	};
</script>

<button
	class="power {state}"
	style:--size="{size}px"
	{disabled}
	{onclick}
	aria-label={labels[state]}
	title={labels[state]}
>
	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
		<path d="M12 3v8" />
		<path d="M6.6 6.6a8 8 0 1 0 10.8 0" />
	</svg>
</button>

<style>
	.power {
		width: var(--size);
		height: var(--size);
		border-radius: 50%;
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		color: var(--fg-muted);
		display: grid;
		place-items: center;
		padding: 0;
		transition:
			color 0.25s,
			box-shadow 0.25s,
			transform 0.1s;
	}
	.power:active:not(:disabled) {
		transform: scale(0.96);
	}
	svg {
		width: 42%;
		height: 42%;
	}
	.connected {
		color: var(--ok);
		box-shadow:
			0 0 0 3px color-mix(in srgb, var(--ok) 35%, transparent),
			0 0 28px color-mix(in srgb, var(--ok) 35%, transparent);
	}
	.connecting {
		color: var(--warn);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--warn) 35%, transparent);
		animation: pulse 1.3s ease-in-out infinite;
	}
	@keyframes pulse {
		50% {
			box-shadow:
				0 0 0 7px color-mix(in srgb, var(--warn) 18%, transparent),
				0 0 24px color-mix(in srgb, var(--warn) 30%, transparent);
		}
	}
</style>
