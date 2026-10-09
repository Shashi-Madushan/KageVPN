<script lang="ts">
	import { onMount } from 'svelte';
	import { usage } from '$lib/usage.svelte';
	import Select from './Select.svelte';

	const GIB = 1024 ** 3;
	let confirmReset = $state(false);

	onMount(() => {
		if (!usage.settings) usage.loadMeta();
	});

	const quotaGb = $derived(usage.settings?.quotaBytes ? +(usage.settings.quotaBytes / GIB).toFixed(2) : '');
</script>

<span class="caption">Data usage</span>
{#if usage.settings}
	{@const s = usage.settings}
	<div class="settings card">
		<label>
			<span><strong>Monthly data limit</strong><small>In GB. Leave empty for no limit. You are warned as you get close.</small></span>
			<input class="input num" type="number" min="0" step="1" value={quotaGb} placeholder="No limit" onchange={(e) => usage.saveSettings({ quotaBytes: e.currentTarget.value ? Math.round(Number(e.currentTarget.value) * GIB) : null })} />
		</label>
		<label>
			<span><strong>Month starts on day</strong><small>The day your data plan or billing cycle resets.</small></span>
			<input class="input num" type="number" min="1" max="31" value={s.cycleDay} onchange={(e) => usage.saveSettings({ cycleDay: Math.min(31, Math.max(1, Number(e.currentTarget.value) || 1)) })} />
		</label>
		<div class="row">
			<span><strong>When the limit is reached</strong><small>Choose whether to just tell you or also disconnect.</small></span>
			<Select
				value={s.quotaAction}
				label="When the limit is reached"
				options={[{ value: 'notify', label: 'Notify me' }, { value: 'disconnect', label: 'Disconnect' }]}
				onchange={(v) => usage.saveSettings({ quotaAction: v })}
			/>
		</div>
	</div>

	<span class="caption">Usage privacy</span>
	<div class="settings card">
		<label>
			<span><strong>Track apps</strong><small>See which apps used your data. Estimated, and stored only on this Mac.</small></span>
			<input type="checkbox" checked={s.recordApps} onchange={(e) => usage.saveSettings({ recordApps: e.currentTarget.checked })} />
		</label>
		<label>
			<span><strong>Track websites</strong><small>See which sites used your data. Stored only on this Mac. Off by default.</small></span>
			<input type="checkbox" checked={s.recordDomains} onchange={(e) => usage.saveSettings({ recordDomains: e.currentTarget.checked })} />
		</label>
		<label>
			<span><strong>Pause counting</strong><small>Stop recording new usage until you turn this off.</small></span>
			<input type="checkbox" checked={s.paused} onchange={(e) => usage.saveSettings({ paused: e.currentTarget.checked })} />
		</label>
		<div class="row">
			<span><strong>Keep app and website history</strong><small>Older detail is deleted automatically. Totals are kept.</small></span>
			<Select
				value={String(s.detailRetentionDays)}
				label="Keep app and website history"
				options={[
					{ value: '30', label: '30 days' },
					{ value: '90', label: '90 days' },
					{ value: '180', label: '180 days' },
					{ value: '365', label: '1 year' }
				]}
				onchange={(v) => usage.saveSettings({ detailRetentionDays: Number(v) })}
			/>
		</div>
		<div class="row">
			<span><strong>Delete history</strong><small>This cannot be undone.</small></span>
			<span class="buttons">
				<button class="btn" onclick={() => usage.clearDetails()}>Clear app and website history</button>
				{#if confirmReset}
					<button class="btn danger" onclick={() => (usage.resetAll(), (confirmReset = false))}>Yes, delete all usage</button>
					<button class="btn" onclick={() => (confirmReset = false)}>Cancel</button>
				{:else}
					<button class="btn danger" onclick={() => (confirmReset = true)}>Reset all usage…</button>
				{/if}
			</span>
		</div>
	</div>
{:else}
	<div class="settings card"><div class="row"><small>Loading…</small></div></div>
{/if}

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
	label + label,
	label + .row,
	.row + .row {
		border-top: 1px solid var(--hairline);
	}
	span {
		display: grid;
	}
	small {
		color: var(--fg-muted);
	}
	.num {
		width: 110px;
		text-align: right;
	}
	.buttons {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: 8px;
	}
</style>
