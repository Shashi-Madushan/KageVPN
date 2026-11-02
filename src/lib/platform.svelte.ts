import { api, type Platform } from './api';

/** Before the backend answers, the user agent is a good enough guess for the first paint. */
function guess(): Platform {
	const windows = typeof navigator !== 'undefined' && /Windows/i.test(navigator.userAgent);
	return windows
		? { os: 'windows', tun: false, killSwitch: false, rootService: false, systemProxy: true, appNap: false }
		: { os: 'macos', tun: true, killSwitch: true, rootService: true, systemProxy: true, appNap: true };
}

/** The OS the app runs on and what works there. Components read this instead of checking the user agent. */
class PlatformStore {
	info = $state<Platform>(guess());
	isMac = $derived(this.info.os === 'macos');
	isWindows = $derived(this.info.os === 'windows');
	/** The modifier shown in shortcut hints. */
	modKey = $derived(this.info.os === 'macos' ? '⌘' : 'Ctrl+');

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
