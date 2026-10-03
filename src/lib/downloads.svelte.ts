/** Everything the app downloads, in one list, so a single place can show progress and history. */
import { listen } from '@tauri-apps/api/event';
import { downloadsApi, type DownloadFinished, type DownloadKind, type DownloadProgress } from './api';

export type DownloadState = 'active' | 'done' | 'failed' | 'cancelled';

export interface DownloadItem {
	id: string;
	kind: DownloadKind | 'other';
	version: string | null;
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
export const STAGES: Record<DownloadProgress['stage'], string> = {
	download: 'Downloading',
	verify: 'Verifying checksum',
	extract: 'Extracting',
	check: 'Checking config',
	install: 'Installing',
	done: 'Finishing'
};

type Retry = (version: string | null) => void;

class Downloads {
	items = $state<DownloadItem[]>([]);
	private retryByKind: Partial<Record<DownloadKind, Retry>> = {};
	private started = false;

	active = $derived(this.items.filter((i) => i.state === 'active'));
	finished = $derived(this.items.filter((i) => i.state !== 'active'));
	/** Combined progress of what is running; null when nothing is running or sizes are unknown. */
	overall = $derived.by(() => {
		const known = this.active.filter((i) => i.percent !== null);
		return known.length ? known.reduce((sum, i) => sum + (i.percent ?? 0), 0) / known.length : null;
	});

	/** Listens for progress from the backend, whichever provider started the download. */
	async init() {
		if (this.started) return;
		this.started = true;
		await listen<DownloadProgress>('download-progress', (e) => this.onProgress(e.payload));
		await listen<DownloadFinished>('download-finished', (e) => this.finish(e.payload.id, e.payload.error));
	}

	/** Lets a feature say how to restart its own kind of download from the menu. */
	onRetry(kind: DownloadKind, retry: Retry) {
		this.retryByKind[kind] = retry;
	}

	private find(id: string) {
		return this.items.find((i) => i.id === id);
	}

	private onProgress(p: DownloadProgress) {
		if (!this.find(p.id) || this.find(p.id)?.state !== 'active') {
			this.start({ id: p.id, kind: p.kind, version: p.version, title: `${p.title} ${p.version}`, detail: STAGES[p.stage] });
		}
		const sized = p.stage === 'download' && p.total > 0;
		const percent = sized ? Math.min(100, (p.received / p.total) * 100) : p.stage === 'download' ? null : 100;
		const detail = sized ? `${STAGES.download} ${Math.round(percent ?? 0)}%` : STAGES[p.stage];
		this.update(p.id, { title: `${p.title} ${p.version}`, detail, percent });
	}

	start(item: Pick<DownloadItem, 'id' | 'title' | 'detail'> & Partial<Pick<DownloadItem, 'kind' | 'version' | 'retry' | 'cancel'>>) {
		const kind = item.kind ?? 'other';
		const version = item.version ?? null;
		const fresh: DownloadItem = {
			kind,
			version,
			cancel: () => downloadsApi.cancel().catch(() => null),
			retry: kind !== 'other' && this.retryByKind[kind] ? () => this.retryByKind[kind]?.(version) : undefined,
			...item,
			state: 'active',
			percent: null,
			error: null
		};
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
