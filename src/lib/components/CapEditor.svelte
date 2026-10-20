<script lang="ts">
	import { untrack } from 'svelte';
	import type { AtLimit, CapPeriod } from '$lib/api';
	import { caps } from '$lib/caps.svelte';
	import { barTone, canBlock, limitError, periodSpan, periodWord, resetText, splitBytes, toBytes, type LimitUnit } from '$lib/capView';
	import { formatBytes } from '$lib/format';
	import Segmented from './Segmented.svelte';

	let { path, name, onclose }: { path: string; name: string; onclose: () => void } = $props();

	const status = $derived(caps.byPath(path));

	// The form starts from the saved cap, or from a sensible new one.
	const start = untrack(() => caps.byPath(path));
	const shown = splitBytes(start?.limitBytes ?? 1024 ** 3);
	let limit = $state(shown.value);
	let unit = $state<LimitUnit>(shown.unit);
	let period = $state<CapPeriod>(start?.period ?? 'day');
	let atLimit = $state<AtLimit>(start?.atLimit ?? 'offerBlock');
	let saving = $state(false);

	const error = $derived(limitError(limit, unit));
	const bytes = $derived(toBytes(limit, unit));
	const changed = $derived(!status || bytes !== status.limitBytes || period !== status.period || atLimit !== status.atLimit);

	const units: { value: LimitUnit; label: string }[] = [
		{ value: 'MB', label: 'MB' },
		{ value: 'GB', label: 'GB' }
	];
	const periods: { value: CapPeriod; label: string }[] = [
		{ value: 'day', label: 'Day' },
		{ value: 'week', label: 'Week' },
		{ value: 'month', label: 'Month' }
	];
	const actions: { value: AtLimit; label: string }[] = [
		{ value: 'notify', label: 'Tell me' },
		{ value: 'offerBlock', label: 'Offer to block' }
	];

	async function save() {
		if (bytes === null || saving) return;
		saving = true;
		await caps.save({ path, limitBytes: bytes, period, atLimit });
		saving = false;
	}

	async function remove() {
		if (await caps.remove(path)) onclose();
	}
</script>

<div class="editor">
	{#if status}
		<div class="usage" class:warn={barTone(status.state) === 'warn'} class:over={barTone(status.state) === 'over'} class:off={barTone(status.state) === 'off'}>
			<div class="bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.min(100, Math.round(status.percent))} aria-label="{name} data used">
				<i style="width:{Math.min(100, status.percent)}%"></i>
			</div>
			<small>
				{formatBytes(status.used)} of {formatBytes(status.limitBytes)} {periodSpan(status.period)}, {resetText(status.periodEnd, status.period)}. Counted through the VPN, approximate.
			</small>
		</div>

		{#if status.state === 'notCounting'}
			<div class="notice">
				<span>App usage is off, so nothing is counted.</span>
				<button type="button" class="btn small" onclick={() => caps.turnOnRecording()}>Turn on</button>
			</div>
		{:else if status.state === 'blocked'}
			<div class="notice over">
				<span>{name} is blocked by its data cap until the {status.period} ends.</span>
				<button type="button" class="btn small" onclick={() => caps.unblock(path)}>Unblock</button>
			</div>
		{:else if canBlock(status)}
			<div class="notice over">
				<span>{name} reached its cap. Blocking reconnects the VPN once.</span>
				<span class="pair">
					<button type="button" class="btn small danger" onclick={() => caps.block(path)}>Block now</button>
					<button type="button" class="btn small" onclick={() => caps.notNow(path)}>Not now</button>
				</span>
			</div>
		{:else if status.state === 'dismissed'}
			<div class="notice"><span>{name} is over its cap, and you chose not to block it this {status.period}.</span></div>
		{/if}
		{#if !status.installed}
			<div class="notice"><span>{name} is not installed, so nothing is counted.</span></div>
		{/if}
	{/if}

	<div class="form">
		<label class="field">
			<span class="field-label">Limit</span>
			<span class="limit">
				<input class="input" inputmode="decimal" bind:value={limit} aria-label="Data limit" aria-invalid={error !== null} spellcheck="false" />
				<Segmented options={units} value={unit} label="Unit" onchange={(v) => (unit = v)} />
			</span>
		</label>
		<div class="field">
			<span class="field-label">Per</span>
			<Segmented options={periods} value={period} label="Period" onchange={(v) => (period = v)} />
		</div>
		<div class="field">
			<span class="field-label">At the limit</span>
			<Segmented options={actions} value={atLimit} label="At the limit" onchange={(v) => (atLimit = v)} />
		</div>
	</div>
	{#if error}<small class="err" role="alert">{error}</small>{/if}
	<small class="hint">
		{periodWord(period)[0].toUpperCase() + periodWord(period).slice(1)} cap. You are told at 80% and 100%.
		{atLimit === 'offerBlock' ? 'At 100% you can block the app until the period ends.' : 'The app is never blocked.'}
	</small>

	<div class="buttons">
		<button type="button" class="btn primary small" disabled={error !== null || !changed || saving} onclick={save}>{status ? 'Save cap' : 'Set cap'}</button>
		{#if status}<button type="button" class="btn small" onclick={remove}>Remove cap</button>{/if}
		<button type="button" class="btn small" onclick={onclose}>Close</button>
	</div>
</div>

<style>
	.editor {
		display: grid;
		gap: 10px;
		padding: 10px 12px 12px 50px;
		background: color-mix(in srgb, var(--hairline) 50%, transparent);
	}
	.usage {
		display: grid;
		gap: 4px;
	}
	.bar {
		height: 6px;
		border-radius: 3px;
		background: var(--hairline);
		overflow: hidden;
	}
	.bar i {
		display: block;
		height: 100%;
		background: var(--accent);
		transition: width 0.4s ease;
	}
	.usage.warn .bar i {
		background: var(--warn);
	}
	.usage.over .bar i {
		background: var(--danger);
	}
	.usage.off .bar i {
		background: var(--fg-muted);
		opacity: 0.5;
	}
	small {
		color: var(--fg-muted);
		font-size: 11px;
	}
	.notice {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 8px 10px;
		border-radius: 8px;
		background: var(--hairline);
		font-size: 12px;
	}
	.notice.over {
		background: color-mix(in srgb, var(--danger) 14%, transparent);
	}
	.pair,
	.buttons {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.form {
		display: flex;
		flex-wrap: wrap;
		gap: 12px 18px;
	}
	.field {
		display: grid;
		gap: 4px;
	}
	.field-label {
		font-size: 11px;
		color: var(--fg-muted);
	}
	.limit {
		display: flex;
		gap: 6px;
		align-items: center;
	}
	.limit .input {
		width: 90px;
	}
	.err {
		color: var(--danger);
	}
	.btn.small {
		min-height: 24px;
		padding: 2px 10px;
		font-size: 12px;
	}
</style>
