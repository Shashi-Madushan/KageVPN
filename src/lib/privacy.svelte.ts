const KEY = 'kage-vpn.revealed-addresses';

/** Keys for the things that can be hidden, so each one has its own switch. */
export const privacyKey = {
	exitIp: 'exit-ip',
	server: (profileId: string) => `server:${profileId}`
};

function read(): Record<string, true> {
	try {
		const list = JSON.parse(localStorage.getItem(KEY) ?? '[]');
		return Array.isArray(list) ? Object.fromEntries(list.map((key) => [String(key), true as const])) : {};
	} catch {
		return {};
	}
}

/**
 * Which addresses are blurred. Every address has its own key and its own eye button; an address
 * is hidden until the user reveals that one. Shared by the main window and the tray popover
 * (one origin, one stored value), so a screen share or a glance over the shoulder never shows
 * anything the user did not choose to show.
 */
class PrivacyStore {
	private revealed = $state<Record<string, true>>({});

	init() {
		this.revealed = read();
		// Fires in the *other* window when one of them changes a switch.
		window.addEventListener('storage', (event) => {
			if (event.key === KEY) this.revealed = read();
		});
	}

	isHidden(key: string): boolean {
		return !this.revealed[key];
	}

	toggle(key: string) {
		const next = { ...this.revealed };
		if (next[key]) delete next[key];
		else next[key] = true;
		this.revealed = next;
		try {
			localStorage.setItem(KEY, JSON.stringify(Object.keys(next)));
		} catch {
			// storage blocked: the choice still applies until the app restarts
		}
	}
}

export const privacy = new PrivacyStore();
