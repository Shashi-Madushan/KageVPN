<script lang="ts">
	import { app } from '$lib/app.svelte';
	import { core } from '$lib/core.svelte';
	import { crashLog } from '$lib/crashLog.svelte';
	import type { Page } from './Sidebar.svelte';

	let { onnavigate }: { onnavigate: (page: Page) => void } = $props();

	const failure = $derived(app.status.state === 'disconnected' ? app.status.failure : null);
	const down = $derived(app.tunnelDown);

	function act(action: string | null) {
		if (action === 'installEngine') core.promptInstall = true;
		else if (action === 'diagnostics') onnavigate('diagnostics');
		else app.toggle();
	}
</script>

{#if failure}
	<div class="banner" role="alert">
		<div class="text">
			<strong>{failure.title}</strong>
			<small>{failure.detail}</small>
		</div>
		<div class="actions">
			<button onclick={() => act(failure.action)}>
				{failure.action === 'installEngine' ? 'Download engine' : failure.action === 'diagnostics' ? 'Open diagnostics' : 'Try again'}
			</button>
			{#if failure.code === 'engineStopped'}<button class="quiet" onclick={() => crashLog.show()}>View crash log</button>{/if}
		</div>
	</div>
{:else if down}
	<div class="banner" role="alert">
		<div class="text">
			<strong>Connected, but no internet</strong>
			<small>{app.status.reason?.message ?? 'Traffic is not getting through the tunnel.'}</small>
		</div>
		<div class="actions">
			{#if app.profiles.length > 1}<button onclick={() => app.switchServer()}>Try another server</button>{/if}
			<button onclick={() => app.restartTunnel()}>Reconnect</button>
			<button class="quiet" onclick={() => onnavigate('diagnostics')}>Diagnostics</button>
			<button class="quiet" onclick={() => app.toggle()}>Disconnect</button>
		</div>
	</div>
{/if}

<style>
	.banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
		padding: 10px 12px;
		border: 1px solid var(--danger);
		border-radius: 12px;
		background: color-mix(in srgb, var(--danger) 10%, var(--surface));
		font-size: 12px;
	}
	.text {
		display: grid;
		gap: 2px;
		min-width: 0;
		flex: 1 1 220px;
	}
	small {
		color: var(--fg-muted);
		overflow-wrap: anywhere;
	}
	.actions {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}
	button {
		border: 0;
		border-radius: 8px;
		padding: 5px 12px;
		background: var(--danger);
		color: #fff;
		font-weight: 600;
	}
	button.quiet {
		background: var(--hairline);
		color: var(--fg);
	}
</style>
