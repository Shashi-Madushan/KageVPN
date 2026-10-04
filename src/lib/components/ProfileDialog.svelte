<script lang="ts">
	import { untrack } from 'svelte';
	import { api, type FieldSection, type FormField, type Profile } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { getPath, setPath } from '$lib/path';
	import { protocols } from '$lib/protocols.svelte';
	import Dialog from './Dialog.svelte';
	import Segmented from './Segmented.svelte';
	import Select from './Select.svelte';

	let { profile, onclose }: { profile: Profile | null; onclose: () => void } = $props();

	let draft = $state<Profile>({} as Profile);
	/** Comma-separated text for list fields, kept as typed until save. */
	let lists = $state<Record<string, string>>({});
	let error = $state<string | null>(null);
	let saving = $state(false);

	const creating = $derived(!draft.id);
	const info = $derived(protocols.find(draft.protocol));
	const fields = $derived(info?.fields ?? []);
	const protocolOptions = $derived(protocols.list.map((p) => ({ value: p.id, label: p.label })));
	const sections: { id: FieldSection; title: string }[] = [
		{ id: 'connection', title: 'Connection' },
		{ id: 'security', title: 'Security' },
		{ id: 'transport', title: 'Transport' }
	];

	function load(source: Profile) {
		draft = structuredClone($state.snapshot(source)) as Profile;
		lists = {};
		for (const field of protocols.find(draft.protocol)?.fields ?? []) {
			if (field.kind === 'list') lists[field.key] = ((getPath(draft.settings, field.key) as string[] | undefined) ?? []).join(',');
		}
		error = null;
	}

	// Load the form only when a different server is opened, never when the user edits it.
	$effect(() => {
		const source = profile;
		if (!source) return;
		untrack(() => load(source));
	});

	const visible = (field: FormField) => {
		if (!field.showWhen) return true;
		return field.showWhen.values.includes(String(getPath(draft.settings, field.showWhen.key) ?? ''));
	};
	const text = (field: FormField) => String(getPath(draft.settings, field.key) ?? '');
	const set = (field: FormField, value: unknown) => setPath(draft.settings, field.key, value);
	const inSection = (section: FieldSection) => fields.filter((f) => f.section === section && visible(f));
	const advanced = $derived(inSection('advanced'));

	function bad(field: FormField): string | null {
		const value = text(field).trim();
		if (!field.pattern || value === '') return null;
		return new RegExp(field.pattern.regex, 'i').test(value) ? null : field.pattern.message;
	}

	async function switchProtocol(id: string) {
		try {
			const next = await api.newProfile(id);
			const { name, server, port, group } = draft;
			load({ ...next, name, server, port, group });
		} catch (e) {
			error = String(e);
		}
	}

	/** Writes the typed values back, turning empty optional text into null and lists into arrays. */
	function collect(): Profile {
		const next = structuredClone($state.snapshot(draft)) as Profile;
		for (const field of fields.filter(visible)) {
			if (field.kind === 'list') {
				setPath(next.settings, field.key, lists[field.key].split(',').map((v) => v.trim()).filter(Boolean));
			} else if (field.kind === 'text' || field.kind === 'select' || field.kind === 'secret') {
				const raw = getPath(next.settings, field.key);
				const value = field.kind === 'secret' ? String(raw ?? '') : String(raw ?? '').trim();
				if (value !== '' || raw !== undefined) setPath(next.settings, field.key, value === '' && field.optional ? null : value);
			}
		}
		next.group = (next.group ?? '').trim() || null;
		return next;
	}

	async function save() {
		saving = true;
		error = await app.saveProfile(collect());
		saving = false;
		if (!error) onclose();
	}

	const ready = $derived(
		draft.name?.trim() !== '' &&
			draft.server?.trim() !== '' &&
			draft.port >= 1 &&
			draft.port <= 65535 &&
			fields.filter(visible).every((f) => f.optional || f.kind === 'toggle' || f.kind === 'segmented' || (f.kind === 'list' ? true : text(f).trim() !== '')) &&
			fields.filter(visible).every((f) => bad(f) === null)
	);
</script>

{#snippet control(field: FormField)}
	{#if field.kind === 'segmented'}
		<Segmented options={field.choices} value={text(field)} label={field.label} onchange={(v) => set(field, v)} />
	{:else if field.kind === 'select'}
		<Select value={text(field)} label={field.label} options={field.choices} onchange={(v) => set(field, v === '' ? null : v)} />
	{:else if field.kind === 'toggle'}
		<label class="check">
			<input type="checkbox" checked={getPath(draft.settings, field.key) === true} onchange={(e) => set(field, e.currentTarget.checked)} />
			<span>{field.label}</span>
		</label>
		{#if field.warning && getPath(draft.settings, field.key) === true}<p class="warn">{field.warning}</p>{/if}
	{:else if field.kind === 'list'}
		<input class="input mono" bind:value={lists[field.key]} placeholder={field.placeholder ?? ''} spellcheck="false" aria-label={field.label} />
	{:else}
		<input
			class="input"
			class:mono={field.mono}
			class:invalid={bad(field) !== null}
			value={text(field)}
			oninput={(e) => set(field, e.currentTarget.value)}
			placeholder={field.placeholder ?? ''}
			spellcheck="false"
			autocomplete="off"
			aria-label={field.label}
		/>
		{#if bad(field)}<span class="field-error">{bad(field)}</span>{/if}
	{/if}
{/snippet}

{#snippet fieldRow(field: FormField)}
	{#if field.kind === 'toggle'}
		<div class="field">{@render control(field)}</div>
	{:else}
		<div class="field">
			<span class="field-label">{field.label}{#if field.optional}{' '}<em>optional</em>{/if}</span>
			{@render control(field)}
		</div>
	{/if}
{/snippet}

<Dialog
	open={profile !== null}
	title={creating ? 'New server' : 'Edit server'}
	subtitle={creating ? 'Enter the details from your provider.' : undefined}
	width={500}
	{onclose}
>
	<form id="server-form" onsubmit={(e) => (e.preventDefault(), save())}>
		{#each sections as section (section.id)}
			{@const sectionFields = inSection(section.id)}
			{#if section.id === 'connection' || sectionFields.length > 0}
				<section>
					<h3 class="caption">{section.title}</h3>
					{#if section.id === 'connection'}
						{#if creating}
							<div class="field">
								<span class="field-label">Protocol</span>
								<Segmented options={protocolOptions} value={draft.protocol} label="Protocol" onchange={switchProtocol} />
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
					{/if}
					{#each sectionFields as field (field.key)}
						{@render fieldRow(field)}
					{/each}
					{#if section.id === 'connection'}
						<label class="field">
							<span class="field-label">Group <em>optional</em></span>
							<input class="input" list="group-options" value={draft.group ?? ''} oninput={(e) => (draft.group = e.currentTarget.value)} placeholder="Ungrouped" />
							<datalist id="group-options">{#each app.groups as group (group)}<option value={group}></option>{/each}</datalist>
						</label>
					{/if}
				</section>
			{/if}
		{/each}

		{#if advanced.length > 0}
			<details class="advanced">
				<summary>Advanced options</summary>
				<div class="advanced-body">
					<div class="field-row even">
						{#each advanced as field (field.key)}
							{@render fieldRow(field)}
						{/each}
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
		.field-row.even {
			grid-template-columns: 1fr;
		}
	}
</style>
