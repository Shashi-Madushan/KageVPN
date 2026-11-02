import { api, type Platform } from './api';

/** Before the backend answers, the user agent is a good enough guess for the first paint. */
function guess(): Platform {
	const windows = typeof navigator !== 'undefined' && /Windows/i.test(navigator.userAgent);
	return windows
		? { os: 'windows', tun: false, killSwitch: false, rootService: false, systemProxy: true, appNap: false, speedInTray: false }
		: { os: 'macos', tun: true, killSwitch: true, rootService: true, systemProxy: true, appNap: true, speedInTray: true };
}

/** The OS the app runs on and what works there. Components read this instead of checking the user agent. */
class PlatformStore {
	info = $state<Platform>(guess());
	isMac = $derived(this.info.os === 'macos');
	isWindows = $derived(this.info.os === 'windows');
	/** The modifier shown in shortcut hints. */
	modKey = $derived(this.info.os === 'macos' ? '⌘' : 'Ctrl+');
	/** What to call the computer and its OS in sentences: "your Mac", "macOS"; "your PC", "Windows". */
	device = $derived(this.info.os === 'macos' ? 'Mac' : 'PC');
	osName = $derived(this.info.os === 'macos' ? 'macOS' : 'Windows');
	/** Where the app's icon lives. */
	trayName = $derived(this.info.os === 'macos' ? 'menu bar' : 'system tray');

	/** A global shortcut as the user reads it: `CommandOrControl+Alt+K` is "⌘ ⌥ K" on a Mac, "Ctrl Alt K" on Windows. */
	shortcut(accelerator: string): string {
		const mac = this.info.os === 'macos';
		return accelerator
			.replace('CommandOrControl', mac ? '⌘' : 'Ctrl')
			.replace('Alt', mac ? '⌥' : 'Alt')
			.replace(/\+/g, ' ');
	}

	async init() {
		try {
			this.info = await api.platform();
		} catch {
			// an older backend or the browser preview: keep the guess
		}
		document.documentElement.dataset.os = this.info.os;
	}
}

export const platform = new PlatformStore();
