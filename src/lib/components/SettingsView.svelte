<script lang="ts">
	import { api } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import type { LogLevel, UpdatePolicy } from '$lib/api';
	import ConfirmDialog from './ConfirmDialog.svelte';
	import Select from './Select.svelte';
	import UsageSettingsSection from './UsageSettingsSection.svelte';
	import NetworksSection from './NetworksSection.svelte';
	import { clipboardWatch } from '$lib/clipboard.svelte';
	import { appUpdate } from '$lib/appUpdate.svelte';
	import { feedback } from '$lib/feedback.svelte';
	import { onboarding } from '$lib/onboarding.svelte';

	let restoring = $state(false);
	let backupInput: HTMLInputElement;
	let backupText = $state<string | null>(null);
	let backupError = $state<string | null>(null);
	let backupBusy = $state(false);

	async function pickBackup(file: File | undefined) {
		backupError = null;
		if (!file) return;
		try {
			backupText = await file.text();
		} catch (e) {
			backupError = `Could not read the file: ${e}`;
		}
		backupInput.value = '';
	}

	async function confirmRestore() {
		if (backupText === null) return;
		backupBusy = true;
		backupError = await app.restoreBackup(backupText);
		backupBusy = false;
		backupText = null;
	}
	let rootService = $state(false);
	let rootBusy = $state(false);
	api.rootServiceInstalled().then((installed) => (rootService = installed), () => {});

	async function toggleRootService(install: boolean) {
		rootBusy = true;
		await (install ? api.installRootService() : api.uninstallRootService()).then(
			() => {
				rootService = install;
				app.notify(install ? 'VPN service installed. TUN mode will not ask for your password again.' : 'VPN service removed.', 'info');
			},
			(e) => app.notify(String(e))
		);
		rootBusy = false;
	}

	function toggleProxyAuth(on: boolean) {
		const bytes = crypto.getRandomValues(new Uint8Array(12));
		const fresh = Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
		app.saveReliability({ proxyAuth: on, proxyPassword: app.reliability.proxyPassword || fresh });
	}

	async function restore() {
		restoring = true;
		await api.restoreNetwork().then(
			() => app.notify('Firewall rules removed.', 'info'),
			(e) => app.notify(String(e))
		);
		restoring = false;
	}
	const logLevels = [
		{ value: 'errors' as const, label: 'Errors only' },
		{ value: 'normal' as const, label: 'Normal' },
		{ value: 'detailed' as const, label: 'Detailed' }
	];
	const logLevelNote: Record<LogLevel, string> = {
		errors: 'Only problems are written. The sites you visit are not listed, except ones that failed to load.',
		normal: 'Problems and every connection, including the sites you visit.',
		detailed: 'Everything, including the sites you visit. Use it to track down a problem, then switch back.'
	};
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
			<span><strong>Reconnect automatically</strong><small>Retry when the tunnel drops, and after your Mac wakes or changes network.</small></span>
			<input type="checkbox" checked={app.reliability.autoReconnect} onchange={(e) => app.saveReliability({ autoReconnect: e.currentTarget.checked })} />
		</label>
		<label>
			<span><strong>Restart a dead tunnel</strong><small>The tunnel is always checked in the background. When it stops passing traffic, restart it automatically.</small></span>
			<input type="checkbox" checked={app.reliability.watchdog} onchange={(e) => app.saveReliability({ watchdog: e.currentTarget.checked })} />
		</label>
		<label>
			<span><strong>Run as a root service</strong><small>Install a background service once so TUN mode never asks for your password again. It runs only when you connect, and sing-box still stops if the app quits. Needs your password to install or remove.</small></span>
			<input type="checkbox" checked={rootService} disabled={rootBusy} onchange={(e) => toggleRootService(e.currentTarget.checked)} />
		</label>
		<label>
			<span><strong>Global shortcut</strong><small>Toggle the VPN from anywhere with {app.hotkey.replace('CommandOrControl', '⌘').replace('Alt', '⌥').replace(/\+/g, ' ')}.</small></span>
			<input type="checkbox" checked={app.reliability.hotkey} onchange={(e) => app.saveReliability({ hotkey: e.currentTarget.checked })} />
		</label>
		<label>
			<span><strong>Password for the local proxy</strong><small>Require a login (user <code>kage</code>) on 127.0.0.1, so other users on this Mac cannot use your tunnel. In Proxy Only mode, enter the password in each app. Applies the next time you connect.</small></span>
			<input type="checkbox" checked={app.reliability.proxyAuth} onchange={(e) => toggleProxyAuth(e.currentTarget.checked)} />
		</label>
		{#if app.reliability.proxyAuth}
			<div class="restore">
				<small>Username <code>kage</code>, password <code class="mono">{app.reliability.proxyPassword}</code></small>
			</div>
		{/if}
		<label>
			<span>
				<strong>Kill switch <em>experimental</em></strong>
				<small>TUN mode only. If sing-box stops or the tunnel stops carrying traffic, block all other internet traffic until the tunnel is verified again. Failed reconnects keep the block; Home has an Unblock internet button. For up to 20 seconds while a reconnect starts, DNS lookups are allowed so the server name can be found.</small>
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

	<NetworksSection />

	<span class="caption">Logs</span>
	<div class="settings card">
		<div class="restore">
			<span><strong>Log detail</strong><small>{logLevelNote[app.settings.logLevel]} Applies the next time you connect.</small></span>
			<Select options={logLevels} value={app.settings.logLevel} label="Log detail" onchange={(level: LogLevel) => app.updateSettings({ logLevel: level })} />
		</div>
		<label>
			<span><strong>Clear logs when disconnected</strong><small>Empty the engine's log files each time the VPN turns off, so no list of visited sites stays on this Mac.</small></span>
			<input type="checkbox" checked={app.settings.clearLogs} onchange={(e) => app.updateSettings({ clearLogs: e.currentTarget.checked })} />
		</label>
	</div>

	<UsageSettingsSection />

	<span class="caption">Backup</span>
	<div class="settings card backup">
		<div class="restore">
			<span><strong>Back up your setup</strong><small>Servers, subscriptions, routing and settings, in one file in Downloads. It holds your server passwords, so keep it private.</small></span>
			<button class="btn" onclick={() => app.exportBackup()}>Export</button>
		</div>
		<div class="restore">
			<span>
				<strong>Restore from a backup</strong>
				<small>{app.locked ? 'Disconnect first.' : 'Replaces your servers, subscriptions and settings. What you have now is kept as data.before-restore.json.'}</small>
				{#if backupError}<small class="fail" role="alert">{backupError}</small>{/if}
			</span>
			<input type="file" accept=".json,application/json" hidden bind:this={backupInput} onchange={(e) => pickBackup(e.currentTarget.files?.[0])} />
			<button class="btn" disabled={app.locked || backupBusy} onclick={() => backupInput.click()}>{backupBusy ? 'Restoring…' : 'Choose file…'}</button>
		</div>
	</div>

	<ConfirmDialog
		open={backupText !== null}
		title="Restore this backup?"
		message="Your servers, subscriptions, routing and settings will be replaced with the ones in the backup. Installed engines and usage history stay as they are."
		confirmLabel="Restore"
		onconfirm={confirmRestore}
		onclose={() => (backupText = null)}
	/>

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
	.fail {
		color: var(--danger);
	}
	.backup .restore + .restore {
		border-top: 1px solid var(--hairline);
	}
	.restore {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 12px;
		padding: 10px 14px 12px;
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
