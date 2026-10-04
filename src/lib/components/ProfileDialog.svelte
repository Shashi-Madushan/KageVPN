<script lang="ts">
	import { untrack } from 'svelte';
	import { app } from '$lib/app.svelte';
	import { blankProfile, type Profile, type Protocol, type TransportKind } from '$lib/api';
	import { PROTOCOLS, protocolInfo } from '$lib/protocols';
	import Select from './Select.svelte';
	import Dialog from './Dialog.svelte';
	import Segmented from './Segmented.svelte';

	let { profile, onclose }: { profile: Profile | null; onclose: () => void } = $props();

	let draft = $state<Profile>(blankProfile());
	let alpn = $state('');
	let error = $state<string | null>(null);
	let saving = $state(false);

	const creating = $derived(!draft.id);
	const info = $derived(protocolInfo(draft.protocol));
	const protocolOptions = PROTOCOLS.map((p) => ({ value: p.id, label: p.label }));
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

	// Load the form only when a different server is opened, never when the user edits it.
	$effect(() => {
		const source = profile;
		if (!source) return;
		untrack(() => {
			draft = structuredClone($state.snapshot(source));
			alpn = draft.alpn.join(',');
			error = null;
		});
	});

	const text = (value: string | null) => value ?? '';
	const orNull = (value: string) => (value.trim() === '' ? null : value.trim());

	function setProtocol(protocol: Protocol) {
		draft.protocol = protocol;
		draft.security = protocol === 'shadowsocks' ? 'none' : 'tls';
		draft.reality = null;
		draft.method = protocol === 'shadowsocks' ? (draft.method ?? 'aes-256-gcm') : null;
		if (protocol !== 'vless') draft.flow = null;
		if (protocol !== 'vless') draft.packetEncoding = null;
		if (protocol !== 'hysteria2') draft.obfs = null;
		draft.fingerprint = protocol === 'vless' || protocol === 'trojan' ? (draft.fingerprint ?? 'chrome') : null;
	}

	function setObfsPassword(password: string) {
		draft.obfs = password ? { kind: 'salamander', password } : null;
	}

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
			password: draft.password,
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

	const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
	const uuidBad = $derived(draft.uuid.trim() !== '' && !UUID.test(draft.uuid.trim()));
	const credentialOk = $derived(info.usesPassword ? draft.password !== '' && (draft.protocol !== 'shadowsocks' || !!draft.method) : UUID.test(draft.uuid.trim()));
	const ready = $derived(
		draft.name.trim() !== '' &&
			draft.server.trim() !== '' &&
			credentialOk &&
			draft.port >= 1 &&
			draft.port <= 65535 &&
			(draft.security !== 'reality' || (draft.reality?.publicKey.trim() ?? '') !== '')
	);
</script>

<Dialog
	open={profile !== null}
	title={creating ? 'New server' : 'Edit server'}
	subtitle={creating ? 'Enter the details from your provider.' : undefined}
	width={500}
	{onclose}
>
	<form id="server-form" onsubmit={(e) => (e.preventDefault(), save())}>
		<section>
			<h3 class="caption">Connection</h3>
			{#if creating}
				<div class="field">
					<span class="field-label">Protocol</span>
					<Segmented options={protocolOptions} value={draft.protocol} label="Protocol" onchange={setProtocol} />
				</div>
			{/if}
			<label class="field">
				<span class="field-label">Name</span>
				<input class="input" bind:value={draft.name} required placeholder="e.g. Frankfurt" />
			</label>
			<div class="field-row">
				<label class="field">
					<span class="field-label">Address</span>
					<input class="input mono" bind:value={draft.server} required placeholder="vpn.example.com" spellcheck="false" />
				</label>
				<label class="field">
					<span class="field-label">Port</span>
					<input class="input" type="number" min="1" max="65535" bind:value={draft.port} />
				</label>
			</div>
			{#if info.usesPassword}
				{#if draft.protocol === 'shadowsocks'}
					<div class="field">
						<span class="field-label">Encryption</span>
						<Select value={draft.method ?? ''} onchange={(v) => (draft.method = v)} label="Encryption method" options={app.info.shadowsocksMethods.map((m) => ({ value: m, label: m }))} />
					</div>
				{/if}
				<label class="field">
					<span class="field-label">Password</span>
					<input class="input mono" bind:value={draft.password} required spellcheck="false" autocomplete="off" />
				</label>
			{:else}
				<label class="field">
					<span class="field-label">User ID (UUID)</span>
					<input class="input mono" class:invalid={uuidBad} bind:value={draft.uuid} required spellcheck="false" placeholder="00000000-0000-0000-0000-000000000000" />
					{#if uuidBad}<span class="field-error">This is not a valid UUID.</span>{/if}
				</label>
			{/if}
			<label class="field">
				<span class="field-label">Group <em>optional</em></span>
				<input class="input" list="group-options" value={text(draft.group)} oninput={(e) => (draft.group = e.currentTarget.value)} placeholder="Ungrouped" />
				<datalist id="group-options">{#each app.groups as group (group)}<option value={group}></option>{/each}</datalist>
			</label>
		</section>

		{#if info.usesTls}
		<section>
			<h3 class="caption">Security</h3>
			{#if draft.protocol === 'vless'}
				<Segmented options={securities} value={draft.security === 'reality' ? 'reality' : 'tls'} label="Security" onchange={setSecurity} />
			{/if}
			<label class="field">
				<span class="field-label">Server name (SNI) <em>optional</em></span>
				<input class="input" value={text(draft.sni)} oninput={(e) => (draft.sni = e.currentTarget.value)} placeholder={draft.server || 'Same as the address'} spellcheck="false" />
			</label>
			{#if draft.security === 'reality' && draft.reality}
				<div class="field-row wide-first">
					<label class="field">
						<span class="field-label">Public key</span>
						<input class="input mono" bind:value={draft.reality.publicKey} spellcheck="false" />
					</label>
					<label class="field">
						<span class="field-label">Short ID</span>
						<input class="input mono" bind:value={draft.reality.shortId} spellcheck="false" />
					</label>
				</div>
			{/if}
			{#if draft.protocol === 'hysteria2'}
				<label class="field">
					<span class="field-label">Obfuscation password <em>optional</em></span>
					<input class="input mono" value={draft.obfs?.password ?? ''} oninput={(e) => setObfsPassword(e.currentTarget.value)} spellcheck="false" autocomplete="off" />
				</label>
			{/if}
			{#if draft.security !== 'reality'}
				<label class="check">
					<input type="checkbox" bind:checked={draft.allowInsecure} />
					<span>Skip certificate check</span>
				</label>
				{#if draft.allowInsecure}
					<p class="warn">Anyone on the network could impersonate this server. Only use this for a server you run yourself.</p>
				{/if}
			{/if}
		</section>
		{/if}

		{#if info.usesTransport}
		<section>
			<h3 class="caption">Transport</h3>
			<Segmented options={transports} value={draft.transport.kind} label="Transport" onchange={(k) => (draft.transport.kind = k)} />
			{#if draft.transport.kind === 'ws' || draft.transport.kind === 'httpupgrade'}
				<div class="field-row even">
					<label class="field">
						<span class="field-label">Path</span>
						<input class="input mono" value={text(draft.transport.path)} oninput={(e) => (draft.transport.path = e.currentTarget.value)} placeholder="/" spellcheck="false" />
					</label>
					<label class="field">
						<span class="field-label">Host header <em>optional</em></span>
						<input class="input mono" value={text(draft.transport.host)} oninput={(e) => (draft.transport.host = e.currentTarget.value)} spellcheck="false" />
					</label>
				</div>
			{:else if draft.transport.kind === 'grpc'}
				<label class="field">
					<span class="field-label">Service name</span>
					<input class="input mono" value={text(draft.transport.serviceName)} oninput={(e) => (draft.transport.serviceName = e.currentTarget.value)} spellcheck="false" />
				</label>
			{/if}
		</section>
		{/if}

		{#if info.usesTls}
		<details class="advanced">
			<summary>Advanced options</summary>
			<div class="advanced-body">
				<div class="field-row even">
					{#if draft.protocol !== 'hysteria2'}
					<div class="field">
						<span class="field-label">Fingerprint</span>
						<Select value={draft.fingerprint ?? ''} onchange={(v) => (draft.fingerprint = v)} label="TLS fingerprint" options={fingerprints.map((f) => ({ value: f, label: f }))} />
					</div>
					{/if}
					{#if draft.protocol === 'vless'}
					<div class="field">
						<span class="field-label">Flow</span>
						<Select
							value={draft.flow ?? ''}
							label="Flow"
							options={[{ value: '', label: 'None' }, { value: 'xtls-rprx-vision', label: 'xtls-rprx-vision' }]}
							onchange={(v) => (draft.flow = v || null)}
						/>
					</div>
					{/if}
				</div>
				<div class="field-row even">
					{#if draft.security !== 'reality'}
						<label class="field">
							<span class="field-label">ALPN</span>
							<input class="input mono" bind:value={alpn} placeholder="h2,http/1.1" spellcheck="false" />
						</label>
					{/if}
					{#if draft.protocol === 'vless'}
					<div class="field">
						<span class="field-label">Packet encoding</span>
						<Select
							value={draft.packetEncoding ?? ''}
							label="Packet encoding"
							options={[{ value: '', label: 'Default' }, { value: 'xudp', label: 'xudp' }, { value: 'packetaddr', label: 'packetaddr' }]}
							onchange={(v) => (draft.packetEncoding = v || null)}
						/>
					</div>
					{/if}
				</div>
			</div>
		</details>
		{/if}

		{#if error}<p class="error" role="alert">{error}</p>{/if}
	</form>

	{#snippet footer()}
		<button type="button" class="btn" onclick={onclose}>Cancel</button>
		<button type="submit" form="server-form" class="btn primary" disabled={saving || !ready}>{saving ? 'Saving…' : creating ? 'Add server' : 'Save changes'}</button>
	{/snippet}
</Dialog>

<style>
	form {
		display: grid;
		gap: 18px;
	}
	section {
		display: grid;
		gap: 10px;
	}
	h3 {
		margin: 0;
	}
	em {
		font-style: normal;
		font-weight: 400;
		color: var(--fg-muted);
	}
	.field-row.even {
		grid-template-columns: 1fr 1fr;
	}
	.field-row.wide-first {
		grid-template-columns: minmax(0, 1fr) 120px;
	}
	.advanced summary {
		cursor: pointer;
		font-size: 12px;
		font-weight: 600;
		color: var(--accent);
		user-select: none;
	}
	.advanced-body {
		display: grid;
		gap: 10px;
		margin-top: 10px;
	}
	.check {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
	}
	.warn {
		margin: 0;
		font-size: 12px;
		line-height: 1.45;
		color: var(--warn);
	}
	.error {
		margin: 0;
		padding: 8px 10px;
		border-radius: 8px;
		font-size: 12px;
		color: var(--danger);
		background: color-mix(in srgb, var(--danger) 10%, transparent);
	}
	@media (max-width: 520px) {
		.field-row,
		.field-row.even,
		.field-row.wide-first {
			grid-template-columns: 1fr;
		}
	}
</style>
