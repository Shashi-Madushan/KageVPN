import { app } from './app.svelte';
import { releaseNotes } from './releaseNotes';
import { compareVersions } from './version';

const KEY = 'kage-vpn.whats-new-seen';

function read(): string | null {
	try {
		return localStorage.getItem(KEY);
	} catch {
		return null;
	}
}

/** Which release notes the user has looked at. "Seen" is a per-device convenience, so it lives in the webview's storage. */
class WhatsNew {
	private seen = $state<string | null>(read());

	/** True when the installed version has notes the user has not opened yet. */
	unread = $derived.by(() => {
		const current = app.info.version;
		if (!current || !releaseNotes.some((r) => r.version === current)) return false;
		return this.seen === null || compareVersions(current, this.seen) > 0;
	});

	/** Records the installed version as seen. */
	markSeen() {
		const current = app.info.version;
		if (!current) return;
		this.seen = current;
		try {
			localStorage.setItem(KEY, current);
		} catch {
			// blocked storage: the badge may show again next launch
		}
	}
}

export const whatsNew = new WhatsNew();
