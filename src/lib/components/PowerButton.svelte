<script lang="ts">
	import type { ConnState } from '$lib/api';

	let {
		state,
		disabled = false,
		size = 104,
		onclick
	}: { state: ConnState; disabled?: boolean; size?: number; onclick: () => void } = $props();

	const labels: Record<ConnState, string> = {
		disconnected: 'Connect to VPN',
		connecting: 'Connecting to VPN…',
		connected: 'Disconnect from VPN'
	};
</script>

<button
	class="power {state}"
	style:--size="{size}px"
	{disabled}
	{onclick}
	aria-label={labels[state]}
	aria-busy={state === 'connecting'}
	title={labels[state]}
>
	{#if state === 'connecting'}
		<svg class="spinner" viewBox="0 0 44 44" aria-hidden="true">
			<circle class="track" cx="22" cy="22" r="18" fill="none" stroke-width="3" />
			<circle class="arc" cx="22" cy="22" r="18" fill="none" stroke-width="3" stroke-linecap="round" stroke-dasharray="60 50" />
		</svg>
	{/if}
	<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
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
		position: relative;
		transition:
			color 0.25s,
			box-shadow 0.25s,
			transform 0.1s;
	}
	.power:active:not(:disabled) {
		transform: scale(0.96);
	}
	.icon {
		width: 42%;
		height: 42%;
		position: relative;
		z-index: 2;
	}
	.spinner {
		position: absolute;
		inset: -2px;
		width: calc(100% + 4px);
		height: calc(100% + 4px);
		border-radius: 50%;
		animation: spin 1.2s linear infinite;
		pointer-events: none;
		z-index: 1;
	}
	.track {
		stroke: color-mix(in srgb, var(--warn) 20%, transparent);
	}
	.arc {
		stroke: var(--warn);
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
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
	}

	@media (prefers-reduced-motion: reduce) {
		.spinner {
			animation: none;
		}
		.arc {
			stroke-dasharray: 20 15;
		}
	}
</style>
