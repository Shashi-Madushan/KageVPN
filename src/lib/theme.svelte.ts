import { getCurrentWindow } from '@tauri-apps/api/window';

export type ThemeMode = 'system' | 'light' | 'dark';

const KEY = 'kage-vpn.theme';

function read(): ThemeMode {
	try {
		const saved = localStorage.getItem(KEY);
		return saved === 'light' || saved === 'dark' ? saved : 'system';
	} catch {
		return 'system';
	}
}

/**
 * Appearance setting shared by the main window and the tray popover (they share one origin, so one
 * stored value). The page colours follow `data-theme` on <html>; the native window is told too, so the
 * frosted sidebar material matches instead of following the OS.
 */
class ThemeStore {
	mode = $state<ThemeMode>('system');

	init() {
		this.mode = read();
		this.apply();
		// Fires in the *other* window when one of them changes the setting.
		window.addEventListener('storage', (event) => {
			if (event.key !== KEY) return;
			this.mode = read();
			this.apply();
		});
	}

	set(mode: ThemeMode) {
		this.mode = mode;
		try {
			localStorage.setItem(KEY, mode);
		} catch {
			// storage blocked: the choice still applies until the app restarts
		}
		this.apply();
	}

	private apply() {
		const root = document.documentElement;
		if (this.mode === 'system') root.removeAttribute('data-theme');
		else root.setAttribute('data-theme', this.mode);
		try {
			getCurrentWindow()
				.setTheme(this.mode === 'system' ? null : this.mode)
				.catch(() => {});
		} catch {
			// not running inside Tauri (for example a plain browser preview)
		}
	}
}

export const theme = new ThemeStore();
