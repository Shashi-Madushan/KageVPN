/** Everything the app downloads, in one list, so a single place can show progress and history. */
export type DownloadState = 'active' | 'done' | 'failed' | 'cancelled';

export interface DownloadItem {
	id: string;
	title: string;
	detail: string;
	state: DownloadState;
	/** 0–100, or null when the size is unknown. */
	percent: number | null;
	error: string | null;
	/** Starts the same download again after it failed or was cancelled. */
	retry?: () => void;
	cancel?: () => void;
}

const KEEP_FINISHED = 8;

class Downloads {
	items = $state<DownloadItem[]>([]);

	active = $derived(this.items.filter((i) => i.state === 'active'));
	finished = $derived(this.items.filter((i) => i.state !== 'active'));
	/** Combined progress of what is running; null when nothing is running or sizes are unknown. */
	overall = $derived.by(() => {
		const known = this.active.filter((i) => i.percent !== null);
		return known.length ? known.reduce((sum, i) => sum + (i.percent ?? 0), 0) / known.length : null;
	});

	private find(id: string) {
		return this.items.find((i) => i.id === id);
	}

	start(item: Pick<DownloadItem, 'id' | 'title' | 'detail' | 'retry' | 'cancel'>) {
		const fresh: DownloadItem = { ...item, state: 'active', percent: null, error: null };
		this.items = [fresh, ...this.items.filter((i) => i.id !== item.id)];
	}

	update(id: string, patch: Partial<Pick<DownloadItem, 'detail' | 'percent' | 'title'>>) {
		const item = this.find(id);
		if (item?.state === 'active') Object.assign(item, patch);
	}

	finish(id: string, error: string | null = null) {
		const item = this.find(id);
		if (!item || item.state !== 'active') return;
		const cancelled = !!error && /cancel/i.test(error);
		item.state = error ? (cancelled ? 'cancelled' : 'failed') : 'done';
		item.error = cancelled ? null : error;
		item.percent = error ? item.percent : 100;
		item.detail = error ? (cancelled ? 'Cancelled' : 'Failed') : 'Finished';
		this.items = [...this.active, ...this.finished.slice(0, KEEP_FINISHED)];
	}

	clearFinished() {
		this.items = this.active;
	}
}

export const downloads = new Downloads();
