<script lang="ts">
	import { api } from '$lib/api';
	import { app } from '$lib/app.svelte';
	import { clipboardWatch } from '$lib/clipboard.svelte';

	let restoring = $state(false);

	async function restore() {
		restoring = true;
		await api.restoreNetwork().then(
			() => app.notify('Firewall rules removed.', 'info'),
			(e) => app.notify(String(e))
		);
		restoring = false;
	}
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
				<button disabled={restoring} onclick={restore}>{restoring ? 'Working…' : 'Restore internet access'}</button>
			</div>
		{/if}
	</div>

	<button class="quit" onclick={() => app.quit()}>Quit SingBox GUI</button>
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
		font-size: 10px;
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
	input[type='checkbox'] {
		width: 34px;
		height: 20px;
		appearance: none;
		border-radius: 10px;
		background: var(--hairline);
		position: relative;
		flex: none;
		transition: background 0.2s;
	}
	input[type='checkbox']::after {
		content: '';
		position: absolute;
		top: 2px;
		left: 2px;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: white;
		box-shadow: 0 1px 2px rgb(0 0 0 / 0.3);
		transition: transform 0.2s;
	}
	input[type='checkbox']:checked {
		background: var(--ok);
	}
	input[type='checkbox']:checked::after {
		transform: translateX(14px);
	}
	button {
		border: 1px solid var(--hairline);
		background: var(--surface-strong);
		border-radius: 8px;
		padding: 6px 14px;
		white-space: nowrap;
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
