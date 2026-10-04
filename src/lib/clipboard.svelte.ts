import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { protocols } from './protocols.svelte';

const KEY = 'clipboardWatch';
const POLL_MS = 2000;

class ClipboardWatch {
	enabled = $state(false);
	offer = $state<string | null>(null);
	private dismissed = new Set<string>();
	private timer: ReturnType<typeof setInterval> | null = null;

	start() {
		try {
			this.enabled = localStorage.getItem(KEY) === '1';
		} catch {
			this.enabled = false;
		}
		this.timer ??= setInterval(() => this.poll(), POLL_MS);
	}

	stop() {
		if (this.timer) clearInterval(this.timer);
		this.timer = null;
	}

	setEnabled(value: boolean) {
		this.enabled = value;
		if (!value) this.offer = null;
		try {
			localStorage.setItem(KEY, value ? '1' : '0');
		} catch {
			// storage can be unavailable; the setting then lasts for this session
		}
	}

	private async poll() {
		if (!this.enabled || !document.hasFocus() || this.offer) return;
		try {
			const text = ((await readText()) ?? '').trim();
			if (protocols.isLink(text) && !this.dismissed.has(text)) this.offer = text;
		} catch {
			// reading can be refused by the system; ignore
		}
	}

	dismiss() {
		if (this.offer) this.dismissed.add(this.offer);
		this.offer = null;
	}
}

export const clipboardWatch = new ClipboardWatch();
