<script lang="ts">
	import { app } from '$lib/app.svelte';
</script>

<div class="settings card">
	<label>
		<span>
			<strong>Launch at login</strong>
			<small>Start in the menu bar when you sign in.</small>
		</span>
		<input type="checkbox" checked={app.launchAtLogin} onchange={(e) => app.setLaunchAtLogin(e.currentTarget.checked)} />
	</label>
	<label>
		<span>
			<strong>Connect on launch</strong>
			<small>Connect the selected profile automatically.</small>
		</span>
		<input type="checkbox" checked={app.settings.autoConnect} onchange={(e) => app.updateSettings({ autoConnect: e.currentTarget.checked })} />
	</label>
	<label>
		<span>
			<strong>Speed in menu bar</strong>
			<small>Show live download and upload next to the icon.</small>
		</span>
		<input type="checkbox" checked={app.settings.showSpeed} onchange={(e) => app.updateSettings({ showSpeed: e.currentTarget.checked })} />
	</label>
</div>

<button class="quit" onclick={() => app.quit()}>Quit SingBox GUI</button>

<style>
	.settings {
		display: grid;
	}
	label {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 16px;
		padding: 12px 14px;
	}
	label + label {
		border-top: 1px solid var(--hairline);
	}
	span {
		display: grid;
	}
	small {
		color: var(--fg-muted);
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
	.quit {
		margin-top: 14px;
		width: 100%;
		padding: 9px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--hairline);
		background: var(--surface);
		color: var(--danger);
		font-weight: 600;
	}
</style>
