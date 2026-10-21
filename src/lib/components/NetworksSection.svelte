<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type CurrentNetwork, type KnownNetwork, type NetworkAction } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import Select from './Select.svelte';
	import { saveToggle } from '$lib/toggle';

	const actions: { value: NetworkAction; label: string }[] = [
		{ value: 'connect', label: 'Connect' },
		{ value: 'disconnect', label: 'Disconnect' },
		{ value: 'ignore', label: 'Do nothing' }
	];

	let current = $state<CurrentNetwork | null>(null);
	const known = $derived(current ? app.networks.networks.find((n) => n.id === current!.id) : undefined);

	onMount(() => {
		api.currentNetwork().then((n) => (current = n), () => {});
	});

	function change(id: string, patch: Partial<KnownNetwork>) {
		app.saveNetworks({ ...app.networks, networks: app.networks.networks.map((n) => (n.id === id ? { ...n, ...patch } : n)) });
	}

	function forget(id: string) {
		app.saveNetworks({ ...app.networks, networks: app.networks.networks.filter((n) => n.id !== id) });
	}

	function remember(action: NetworkAction) {
		if (!current) return;
		app.saveNetworks({ ...app.networks, networks: [...app.networks.networks, { id: current.id, name: current.name, action, lastSeen: Math.floor(Date.now() / 1000) }] });
	}
</script>

<span class="caption">Networks</span>
<div class="settings card">
	<label>
		<span><strong>Connect or disconnect by network</strong><small>Each network is recognised by its router's hardware address, so no location permission is needed. A network you have not used before asks once. Connecting or disconnecting by hand wins until the network changes.</small></span>
		<input type="checkbox" checked={app.networks.enabled} onchange={(e) => saveToggle(e, (on) => app.saveNetworks({ ...app.networks, enabled: on }))} />
	</label>
	{#if app.networks.enabled}
		{#if current && !known}
			<div class="row">
				<span><strong>This network: {current.name}</strong><small>Not remembered yet.</small></span>
				<div class="buttons">
					<button class="btn" onclick={() => remember('connect')}>Connect here</button>
					<button class="btn" onclick={() => remember('disconnect')}>Disconnect here</button>
				</div>
			</div>
		{/if}
		{#each app.networks.networks as network (network.id)}
			<div class="row">
				<span>
					<input class="input name" value={network.name} maxlength="40" aria-label="Network name" onchange={(e) => change(network.id, { name: e.currentTarget.value.trim() || network.name })} />
					<small>{current?.id === network.id ? 'You are on this network. ' : ''}{network.action === 'disconnect' ? 'Turns the VPN off here, and also lifts the kill switch.' : network.id}</small>
				</span>
				<div class="buttons">
					<Select value={network.action} label="Action on {network.name}" options={actions} onchange={(v: NetworkAction) => change(network.id, { action: v })} />
					<button class="btn danger" onclick={() => forget(network.id)}>Forget</button>
				</div>
			</div>
		{/each}
		{#if app.networks.networks.length === 0 && !current}
			<div class="row"><small>No networks yet. They appear here as you use them.</small></div>
		{/if}
	{/if}
</div>

<style>
	.settings {
		display: grid;
		margin-bottom: 10px;
	}
	label,
	.row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 16px;
		padding: 12px 14px;
	}
	label + .row,
	.row + .row {
		border-top: 1px solid var(--hairline);
	}
	span {
		display: grid;
		gap: 2px;
	}
	small {
		color: var(--fg-muted);
	}
	.name {
		width: 220px;
		max-width: 100%;
	}
	.buttons {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		align-items: center;
		gap: 8px;
	}
</style>
