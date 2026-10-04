const KEY = 'apzur-vpn.hide-addresses';

function read(): boolean {
	try {
		// Hidden unless the user chose to show them.
		return localStorage.getItem(KEY) !== '0';
	} catch {
		return true;
	}
}

/**
 * Whether server and IP addresses are blurred. Shared by the main window and the tray popover
 * (one origin, one stored value). Starts hidden, so a screen share or a glance over the shoulder
 * does not reveal anything until the user clicks the eye.
 */
class PrivacyStore {
	hidden = $state(true);

	init() {
		this.hidden = read();
		// Fires in the *other* window when one of them changes the setting.
		window.addEventListener('storage', (event) => {
			if (event.key === KEY) this.hidden = read();
		});
	}

	toggle() {
		this.hidden = !this.hidden;
		try {
			localStorage.setItem(KEY, this.hidden ? '1' : '0');
		} catch {
			// storage blocked: the choice still applies until the app restarts
		}
	}
}

export const privacy = new PrivacyStore();
