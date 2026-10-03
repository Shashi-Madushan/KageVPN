const KEY = 'apzur-vpn.onboarding-seen';

function read(): boolean {
	try {
		return localStorage.getItem(KEY) === '1';
	} catch {
		return false;
	}
}

/** First-run tour state. "Seen" is a per-device convenience, so it lives in the webview's storage. */
class Onboarding {
	open = $state(false);
	seen = $state(read());
	/** Set by the tour's last step; the Servers page picks it up and opens "Add server". */
	addServerRequested = $state(false);

	show() {
		this.open = true;
	}

	finish() {
		this.open = false;
		this.markSeen();
	}

	markSeen() {
		this.seen = true;
		try {
			localStorage.setItem(KEY, '1');
		} catch {
			// private mode or blocked storage: the tour may show again next launch
		}
	}
}

export const onboarding = new Onboarding();
