<script lang="ts">
	import { api } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import type { UpdatePolicy } from '$lib/api';
	import PoweredBy from './PoweredBy.svelte';
	import Select from './Select.svelte';
	import { clipboardWatch } from '$lib/clipboard.svelte';
	import { appUpdate } from '$lib/appUpdate.svelte';
	import { feedback } from '$lib/feedback.svelte';
	import { onboarding } from '$lib/onboarding.svelte';

	let restoring = $state(false);

	async function restore() {
		restoring = true;
		await api.restoreNetwork().then(
			() => app.notify('Firewall rules removed.', 'info'),
			(e) => app.notify(String(e))
		);
		restoring = false;
	}
	const policies = [
		{ value: 'ask' as const, label: 'Ask me' },
		{ value: 'auto' as const, label: 'Install automatically' },
		{ value: 'off' as const, label: 'Do nothing' }
	];
</script>

<div class="page">
	<span class="caption">General</span>
	<div class="settings card">
		<label>
			<span><strong>Launch at login</strong><small>Start in the menu bar when you sign in.</small></span>
			<input type="checkbox" checked={app.launchAtLogin} onchange={(e) => app.setLaunchAtLogin(e.currentTarget.checked)} />
		</label>
		<label>
			<span><strong>Connect on launch</strong><small>Connect the selected profile automatically.</small></span>
			<input type="checkbox" checked={app.settings.autoConnect} onchange={(e) => app.updateSettings({ autoConnect: e.currentTarget.checked })} />
		</label>
		<label>
			<span><strong>Speed in menu bar</strong><small>Show live download and upload next to the icon.</small></span>
			<input type="checkbox" checked={app.settings.showSpeed} onchange={(e) => app.updateSettings({ showSpeed: e.currentTarget.checked })} />
		</label>
		<label>
			<span><strong>Offer to import copied links</strong><small>While this window is focused, notice a copied vless:// link. macOS may ask to allow clipboard access.</small></span>
			<input type="checkbox" checked={clipboardWatch.enabled} onchange={(e) => clipboardWatch.setEnabled(e.currentTarget.checked)} />
		</label>
	</div>

	<span class="caption">Reliability</span>
	<div class="settings card">
		<label>
			<span><strong>Reconnect automatically</strong><small>Retry when the tunnel drops or sing-box stops.</small></span>
			<input type="checkbox" checked={app.reliability.autoReconnect} onchange={(e) => app.saveReliability({ autoReconnect: e.currentTarget.checked })} />
		</label>
		<label>
			<span><strong>Connection watchdog</strong><small>Check the tunnel every 30 seconds and reconnect after three failures.</small></span>
			<input type="checkbox" checked={app.reliability.watchdog} onchange={(e) => app.saveReliability({ watchdog: e.currentTarget.checked })} />
		</label>
		<label>
			<span><strong>Global shortcut</strong><small>Toggle the VPN from anywhere with {app.hotkey.replace('CommandOrControl', '⌘').replace('Alt', '⌥').replace(/\+/g, ' ')}.</small></span>
			<input type="checkbox" checked={app.reliability.hotkey} onchange={(e) => app.saveReliability({ hotkey: e.currentTarget.checked })} />
		</label>
		<label>
			<span>
				<strong>Kill switch <em>experimental</em></strong>
				<small>TUN mode only. If sing-box stops unexpectedly, block internet traffic until you disconnect or reconnect. Reconnecting asks for your password again.</small>
			</span>
			<input type="checkbox" checked={app.reliability.killSwitch} onchange={(e) => app.saveReliability({ killSwitch: e.currentTarget.checked })} />
		</label>
		{#if app.reliability.killSwitch}
			<div class="restore">
				<small>If your internet stays blocked after the app quits unexpectedly, remove the firewall rules:</small>
				<button class="btn" disabled={restoring} onclick={restore}>{restoring ? 'Working…' : 'Restore internet access'}</button>
			</div>
		{/if}
	</div>

	<span class="caption">App updates</span>
	<div class="settings card">
		<div class="restore">
			<span>
				<strong>Version {app.info.version}</strong>
				<small>
					{#if !app.info.updatesConfigured}
						Update checks are off because this build has no update feed (set APP_UPDATE_FEED in app_config.rs).
					{:else}
						{appUpdate.checkedMessage ?? 'Updates are checked once a day.'}
					{/if}
				</small>
			</span>
			<button class="btn" disabled={!app.info.updatesConfigured || appUpdate.checking} onclick={() => appUpdate.check()}>{appUpdate.checking ? 'Checking…' : 'Check now'}</button>
		</div>
			<div class="restore">
				<span><strong>When an update is available</strong><small>Automatic installs download in the background. Restarting is always your choice.</small></span>
				<Select options={policies} value={appUpdate.policy} label="App update policy" onchange={(p: UpdatePolicy) => appUpdate.setPolicy(p)} />
			</div>
		</div>

	<span class="caption">Help</span>
	<div class="settings card">
		<div class="restore">
			<span><strong>{app.info.name}</strong><small>Version {app.info.version}</small></span>
		</div>
		<div class="restore">
			<span><strong>Welcome tour</strong><small>A quick walkthrough of adding a server and connecting.</small></span>
			<button class="btn" onclick={() => onboarding.show()}>Show tour</button>
		</div>
		{#if app.info.feedbackConfigured}
			<div class="restore">
				<span><strong>Feedback</strong><small>Report a problem, suggest an idea or ask a question.</small></span>
				<button class="btn" onclick={() => feedback.show()}>Send feedback…</button>
			</div>
		{/if}
	</div>

	<button class="btn quit" onclick={() => app.quit()}>Quit {app.info.name}</button>

	<div class="brand"><PoweredBy /></div>
</div>

<style>
	.page {
		display: grid;
		gap: 8px;
		align-content: start;
	}
	.settings {
		display: grid;
		margin-bottom: 10px;
	}
	label {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 16px;
		padding: 12px 14px;
	}
	label + label,
	label + .restore {
		border-top: 1px solid var(--hairline);
	}
	span {
		display: grid;
	}
	small {
		color: var(--fg-muted);
	}
	em {
		font-style: normal;
		font-size: 11px;
		padding: 1px 6px;
		margin-left: 4px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--warn) 22%, transparent);
		color: var(--warn);
	}
	.restore {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 12px;
		padding: 10px 14px 12px;
	}
	.brand {
		margin-top: 14px;
		padding-bottom: 4px;
	}
	.quit {
		margin-top: 4px;
		width: 100%;
		padding: 9px;
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--danger);
		font-weight: 600;
	}
</style>
