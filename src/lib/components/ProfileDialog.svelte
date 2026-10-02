<script lang="ts">
	import { app } from '$lib/app.svelte';
	import { blankProfile, type Profile, type TransportKind } from '$lib/api';
	import Segmented from './Segmented.svelte';

	let { profile, onclose }: { profile: Profile | null; onclose: () => void } = $props();

	let dialog: HTMLDialogElement;
	let draft = $state<Profile>(blankProfile());
	let alpn = $state('');
	let error = $state<string | null>(null);
	let saving = $state(false);

	const creating = $derived(!draft.id);
	const fingerprints = ['chrome', 'firefox', 'safari', 'ios', 'android', 'edge', 'random'];
	const transports: { value: TransportKind; label: string }[] = [
		{ value: 'tcp', label: 'TCP' },
		{ value: 'ws', label: 'WebSocket' },
		{ value: 'grpc', label: 'gRPC' },
		{ value: 'httpupgrade', label: 'HTTPUpgrade' }
	];
	const securities = [
		{ value: 'tls' as const, label: 'TLS' },
		{ value: 'reality' as const, label: 'Reality' }
	];

	$effect(() => {
		if (profile && !dialog.open) {
			draft = structuredClone($state.snapshot(profile));
			alpn = draft.alpn.join(',');
			error = null;
			dialog.showModal();
		}
		if (!profile && dialog.open) dialog.close();
	});

	const text = (value: string | null) => value ?? '';
	const orNull = (value: string) => (value.trim() === '' ? null : value.trim());

	function setSecurity(security: 'tls' | 'reality') {
		draft.security = security;
		draft.reality = security === 'reality' ? (draft.reality ?? { publicKey: '', shortId: '' }) : null;
		if (security === 'reality' && !draft.flow) draft.flow = 'xtls-rprx-vision';
	}

	async function save() {
		saving = true;
		const next: Profile = {
			...draft,
			name: draft.name.trim(),
			server: draft.server.trim(),
			uuid: draft.uuid.trim(),
			alpn: alpn.split(',').map((a) => a.trim()).filter(Boolean),
			sni: orNull(text(draft.sni)),
			flow: orNull(text(draft.flow)),
			packetEncoding: orNull(text(draft.packetEncoding)),
			group: orNull(text(draft.group)),
			transport: {
				kind: draft.transport.kind,
				path: orNull(text(draft.transport.path)),
				host: orNull(text(draft.transport.host)),
				serviceName: orNull(text(draft.transport.serviceName))
			}
		};
		error = await app.saveProfile(next);
		saving = false;
		if (!error) onclose();
	}
</script>

<dialog bind:this={dialog} onclose={onclose} aria-labelledby="dialog-title">
	<h3 id="dialog-title">{creating ? 'New Profile' : 'Edit Profile'}</h3>
	<form onsubmit={(e) => (e.preventDefault(), save())}>
		<div class="grid">
			<label class="wide">Name <input bind:value={draft.name} required aria-label="Profile name" placeholder="e.g. US West" /></label>
			<label>Server Host/IP <input bind:value={draft.server} class="mono" required aria-label="Server address" placeholder="example.com" /></label>
			<label>Port <input type="number" min="1" max="65535" bind:value={draft.port} aria-label="Port number" /></label>
			<label class="wide">UUID <input bind:value={draft.uuid} class="mono" spellcheck="false" required aria-label="Client UUID" /></label>
			<label class="wide">
				Group / Folder
				<input list="group-options" value={text(draft.group)} oninput={(e) => (draft.group = e.currentTarget.value)} placeholder="None" aria-label="Profile group" />
				<datalist id="group-options">{#each app.groups as group (group)}<option value={group}></option>{/each}</datalist>
			</label>

			<details class="wide advanced-box" open={draft.security === 'reality'}>
				<summary>Advanced VLESS Parameters (Reality, Transport, Flow, ALPN)</summary>
				<div class="grid adv-grid">
					<div class="wide field">
						<span>Security</span>
						<Segmented options={securities} value={draft.security} label="Security protocol" onchange={setSecurity} />
					</div>

					<label>SNI <input value={text(draft.sni)} oninput={(e) => (draft.sni = e.currentTarget.value)} placeholder={draft.server || 'server name'} aria-label="Server Name Indication" /></label>
					<label>
						Fingerprint
						<select bind:value={draft.fingerprint} aria-label="TLS Fingerprint">
							{#each fingerprints as fp (fp)}<option value={fp}>{fp}</option>{/each}
						</select>
					</label>

					{#if draft.security === 'reality' && draft.reality}
						<label class="wide">Reality Public Key <input bind:value={draft.reality.publicKey} class="mono" spellcheck="false" aria-label="Reality public key" /></label>
						<label>Short ID <input bind:value={draft.reality.shortId} class="mono" aria-label="Short ID" /></label>
					{:else}
						<label>ALPN <input bind:value={alpn} placeholder="h2,http/1.1" aria-label="Application-Layer Protocol Negotiation" /></label>
					{/if}

					<label>
						Flow
						<select value={draft.flow ?? ''} onchange={(e) => (draft.flow = e.currentTarget.value || null)} aria-label="Flow control">
							<option value="">None</option>
							<option value="xtls-rprx-vision">xtls-rprx-vision</option>
						</select>
					</label>

					<div class="wide field">
						<span>Transport</span>
						<Segmented options={transports} value={draft.transport.kind} label="Transport protocol" onchange={(k) => (draft.transport.kind = k)} />
					</div>

					{#if draft.transport.kind === 'ws' || draft.transport.kind === 'httpupgrade'}
						<label>Path <input value={text(draft.transport.path)} oninput={(e) => (draft.transport.path = e.currentTarget.value)} placeholder="/" aria-label="WebSocket Path" /></label>
						<label>Host Header <input value={text(draft.transport.host)} oninput={(e) => (draft.transport.host = e.currentTarget.value)} aria-label="Host header" /></label>
					{:else if draft.transport.kind === 'grpc'}
						<label class="wide">Service Name <input value={text(draft.transport.serviceName)} oninput={(e) => (draft.transport.serviceName = e.currentTarget.value)} aria-label="gRPC Service name" /></label>
					{/if}

					<label class="wide">
						Packet Encoding
						<select value={draft.packetEncoding ?? ''} onchange={(e) => (draft.packetEncoding = e.currentTarget.value || null)} aria-label="Packet encoding">
							<option value="">Default</option>
							<option value="xudp">xudp</option>
							<option value="packetaddr">packetaddr</option>
						</select>
					</label>
				</div>
			</details>
		</div>

		{#if error}<p class="error">{error}</p>{/if}
		<div class="actions">
			<button type="button" onclick={onclose}>Cancel</button>
			<button type="submit" class="primary" disabled={saving}>{saving ? 'Saving…' : 'Save'}</button>
		</div>
	</form>
</dialog>

<style>
	dialog {
		width: min(520px, 94vw);
		max-height: 88vh;
		overflow-y: auto;
		border: 1px solid var(--hairline);
		border-radius: var(--radius);
		background: var(--surface-strong);
		color: var(--fg);
		padding: 18px;
		box-shadow: 0 20px 60px rgb(0 0 0 / 0.35);
	}
	dialog::backdrop {
		background: rgb(0 0 0 / 0.3);
	}
	h3 {
		margin: 0 0 12px;
		font-size: 15px;
	}
	.grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px 12px;
	}
	.adv-grid {
		margin-top: 10px;
	}
	.wide {
		grid-column: 1 / -1;
	}
	label,
	.field {
		display: grid;
		gap: 4px;
		font-size: 11px;
		color: var(--fg-muted);
		font-weight: 600;
	}
	input,
	select {
		font-weight: 400;
		font-size: 13px;
		padding: 6px 8px;
		border-radius: 8px;
		border: 1px solid var(--hairline);
		background: var(--surface);
		user-select: text;
		min-width: 0;
	}
	.advanced-box {
		border: 1px solid var(--hairline);
		border-radius: 8px;
		padding: 8px 10px;
		background: color-mix(in srgb, var(--hairline) 50%, transparent);
	}
	.advanced-box summary {
		cursor: pointer;
		font-weight: 600;
		font-size: 11.5px;
		color: var(--accent);
		user-select: none;
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 14px;
	}
	button {
		border: 1px solid var(--hairline);
		background: var(--surface);
		border-radius: 8px;
		padding: 6px 16px;
	}
	button.primary {
		background: var(--accent);
		border-color: var(--accent);
		color: white;
		font-weight: 600;
	}
	.error {
		color: var(--danger);
		font-size: 12px;
		margin: 10px 0 0;
	}
</style>
