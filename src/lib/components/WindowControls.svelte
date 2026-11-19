<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	// Windows draws no title bar for this window, so the app provides the three standard buttons,
	// sized and coloured like Windows 11's own. The glyphs come from the system icon font.
	const win = getCurrentWindow();
	let maximized = $state(false);

	onMount(() => {
		const read = () => win.isMaximized().then((m) => (maximized = m)).catch(() => {});
		read();
		const off = win.onResized(read);
		return () => {
			off.then((stop) => stop());
		};
	});
</script>

<div class="controls">
	<button aria-label="Minimize" title="Minimize" onclick={() => win.minimize()}>&#xE921;</button>
	<button aria-label={maximized ? 'Restore' : 'Maximize'} title={maximized ? 'Restore down' : 'Maximize'} onclick={() => win.toggleMaximize()}>
		{@html maximized ? '&#xE923;' : '&#xE922;'}
	</button>
	<button class="close" aria-label="Close" title="Close" onclick={() => win.close()}>&#xE8BB;</button>
</div>

<style>
	.controls {
		position: absolute;
		top: 0;
		right: 0;
		z-index: 3;
		display: flex;
		-webkit-app-region: no-drag;
	}
	button {
		width: 46px;
		height: 32px;
		padding: 0;
		border: 0;
		border-radius: 0;
		background: transparent;
		color: var(--fg);
		font-family: 'Segoe Fluent Icons', 'Segoe MDL2 Assets';
		font-size: 10px;
		line-height: 32px;
		cursor: default;
	}
	button:hover {
		background: color-mix(in srgb, var(--fg) 9%, transparent);
	}
	button:active {
		background: color-mix(in srgb, var(--fg) 6%, transparent);
	}
	.close:hover {
		background: #c42b1c;
		color: #fff;
	}
	.close:active {
		background: #c84031;
		color: #fff;
	}
</style>
