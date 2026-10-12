<script lang="ts">
	import { untrack } from 'svelte';
	import type { Subscription, SubscriptionOptions } from '$lib/api';
	import { focusOnMount } from '$lib/actions';
	import Select from './Select.svelte';

	let {
		subscription = null,
		busy = false,
		error = null,
		submitLabel,
		onsubmit,
		oncancel
	}: {
		/** The subscription being edited, or `null` to add one. */
		subscription?: Subscription | null;
		busy?: boolean;
		error?: string | null;
		submitLabel: string;
		onsubmit: (name: string, url: string, options: SubscriptionOptions) => void;
		oncancel: () => void;
	} = $props();

	/** Client names providers commonly recognise. Some send a different list, or none, by name. */
	const clients = [
		{ value: '', label: 'This app (default)' },
		{ value: 'v2rayN/7.10.0', label: 'v2rayN' },
		{ value: 'SFM/1.14.2', label: 'sing-box' },
		{ value: 'clash.meta/1.19.0', label: 'Clash Meta' },
		{ value: 'custom', label: 'Custom…' }
	];

	// The form edits a copy; later changes to the saved subscription do not reset what is typed.
	const initial = untrack(() => subscription);
	const initialAgent = initial?.userAgent ?? '';
	const preset = clients.some((c) => c.value === initialAgent && c.value !== 'custom');

	let name = $state(initial?.name ?? '');
	let url = $state(initial?.url ?? '');
	let client = $state(preset ? initialAgent : 'custom');
	let customAgent = $state(preset ? '' : initialAgent);
	let sendDeviceId = $state(initial?.sendDeviceId ?? false);
	// Editing opens the options a user changed before; adding keeps them out of the way.
	let showOptions = $state(!!initial?.userAgent || !!initial?.sendDeviceId);

	function submit(e: SubmitEvent) {
		e.preventDefault();
		const userAgent = client === 'custom' ? customAgent.trim() || null : client || null;
		onsubmit(name, url, { userAgent, sendDeviceId });
	}
</script>

<form class="item sub-form" onsubmit={submit}>
	<div class="line">
		<input class="input name" placeholder="Name (optional)" bind:value={name} aria-label="Subscription name" />
		<input class="input url mono" placeholder="https://…/subscription" bind:value={url} aria-label="Subscription URL" spellcheck="false" use:focusOnMount />
	</div>
	{#if showOptions}
		<div class="options">
			<div class="option">
				<span class="grow">Ask as<small>Some providers send a different list, or none, depending on the app asking. Try another if the list comes back empty.</small></span>
				<Select bind:value={client} options={clients} label="Client name sent to the provider" />
			</div>
			{#if client === 'custom'}
				<input class="input mono" placeholder="Client name, such as v2rayNG/1.9.0" bind:value={customAgent} aria-label="Custom client name" spellcheck="false" maxlength="200" />
			{/if}
			<label class="option">
				<span class="grow">Send device ID<small>For providers that limit how many devices can use a subscription. Sends a random ID made for this Mac, the macOS version and the Mac model.</small></span>
				<input type="checkbox" bind:checked={sendDeviceId} />
			</label>
		</div>
	{/if}
	<div class="line actions">
		{#if !showOptions}<button type="button" class="linkbtn" onclick={() => (showOptions = true)}>Options…</button>{/if}
		<span class="grow"></span>
		<button type="button" class="btn" onclick={oncancel}>Cancel</button>
		<button class="btn primary" disabled={!url.trim() || busy}>{busy ? 'Working…' : submitLabel}</button>
	</div>
	{#if error}<span class="error" role="alert">{error}</span>{/if}
</form>

<style>
	.sub-form {
		display: grid;
		gap: 8px;
		align-items: stretch;
	}
	.line {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}
	.name {
		width: 150px;
	}
	.url {
		flex: 1;
		min-width: 160px;
	}
	.options {
		display: grid;
		gap: 8px;
		padding: 8px 10px;
		border-radius: 8px;
		background: color-mix(in srgb, var(--hairline) 40%, transparent);
	}
	.option {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.option .grow {
		display: grid;
		font-size: 12.5px;
	}
	small {
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	.error {
		font-size: 12px;
		color: var(--danger);
	}
</style>
