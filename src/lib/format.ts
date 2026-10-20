import type { ImportSummary, UserInfo } from './api';
import type { Latency } from './app.svelte';

const pad = (n: number) => String(n).padStart(2, '0');

export function formatDuration(totalSeconds: number): string {
	const h = Math.floor(totalSeconds / 3600);
	const m = Math.floor((totalSeconds % 3600) / 60);
	return `${pad(h)}:${pad(m)}:${pad(totalSeconds % 60)}`;
}

const UNITS = ['B', 'KB', 'MB', 'GB', 'TB'];

export function formatBytes(bytes: number): string {
	let value = Number.isFinite(bytes) && bytes > 0 ? bytes : 0;
	let unit = 0;
	while (value >= 1024 && unit < UNITS.length - 1) {
		value /= 1024;
		unit++;
	}
	return `${unit === 0 ? Math.round(value) : value.toFixed(value < 10 ? 1 : 0)} ${UNITS[unit]}`;
}

/** A speed with a unit, always rounded, never a raw float. */
export const formatRate = (bytesPerSecond: number) => `${formatBytes(bytesPerSecond)}/s`;

/** Like `formatRate`, but a dash when nothing is moving, for tables where idle rows should look quiet. */
export const formatRateOrDash = (bytesPerSecond: number) => (Number.isFinite(bytesPerSecond) && bytesPerSecond >= 1 ? formatRate(bytesPerSecond) : '—');

type LatencyTone = 'ok' | 'warn' | 'bad' | 'idle';

export function latencyTone(latency: Latency | undefined): LatencyTone {
	if (!latency || 'pending' in latency) return 'idle';
	if ('failed' in latency) return 'bad';
	return latency.ms < 300 ? 'ok' : latency.ms < 800 ? 'warn' : 'bad';
}

export function latencyLabel(latency: Latency | undefined): string {
	if (!latency) return 'Test';
	if ('pending' in latency) return '…';
	if ('failed' in latency) return 'Timeout';
	return `${latency.ms} ms`;
}

const COUNTRY_MAP: Record<string, string> = {
	us: 'US',
	usa: 'US',
	'united states': 'US',
	uk: 'GB',
	gb: 'GB',
	'united kingdom': 'GB',
	jp: 'JP',
	japan: 'JP',
	sg: 'SG',
	singapore: 'SG',
	hk: 'HK',
	'hong kong': 'HK',
	de: 'DE',
	germany: 'DE',
	fr: 'FR',
	france: 'FR',
	ca: 'CA',
	canada: 'CA',
	au: 'AU',
	australia: 'AU',
	nl: 'NL',
	netherlands: 'NL',
	kr: 'KR',
	korea: 'KR',
	tw: 'TW',
	taiwan: 'TW',
	in: 'IN',
	india: 'IN',
	ru: 'RU',
	russia: 'RU',
	tr: 'TR',
	turkey: 'TR',
	br: 'BR',
	brazil: 'BR',
	it: 'IT',
	italy: 'IT',
	es: 'ES',
	spain: 'ES',
	se: 'SE',
	sweden: 'SE',
	ch: 'CH',
	switzerland: 'CH'
};

function countryCodeToFlag(code: string): string {
	if (!code || code.length !== 2) return '🌐';
	const base = 127397;
	return String.fromCodePoint(...[...code.toUpperCase()].map((c) => base + c.charCodeAt(0)));
}

export function profileFlag(name: string, server: string = ''): string {
	const text = `${name} ${server}`.toLowerCase();
	for (const [key, iso] of Object.entries(COUNTRY_MAP)) {
		const regex = new RegExp(`(^|[^a-z0-9])${key}([^a-z0-9]|$)`, 'i');
		if (regex.test(text)) {
			return countryCodeToFlag(iso);
		}
	}
	return '🌐';
}

/** " (why)" for the first reason a link was skipped, or nothing. Long reasons are cut short. */
export function skippedReason(reason: string | null | undefined): string {
	if (!reason) return '';
	return ` (${reason.length > 90 ? reason.slice(0, 89) + '…' : reason})`;
}

/** One-line result of an import, covering duplicates and unreadable links. */
export function importMessage(summary: ImportSummary): string {
	const parts: string[] = [];
	if (summary.added) parts.push(`Imported ${summary.added} profile${summary.added === 1 ? '' : 's'}`);
	if (summary.duplicates) parts.push(summary.added ? `${summary.duplicates} already saved` : 'Already imported');
	if (summary.failed.length) parts.push(`${summary.failed.length} skipped${skippedReason(summary.failed[0])}`);
	return (parts.join(', ') || 'Nothing to import') + '.';
}

/** Drops a leading emoji flag from a name, since the flag is shown separately. */
export const stripFlag = (name: string) => name.replace(/^\p{RI}{2}\s*/u, '');

/**
 * A provider's plan in a few words, such as "Plan: 12.4 GB left · 9 days left", from the figures
 * its subscription reports. `total` 0 means unlimited data. `low` is set near the end of either.
 */
export function planSummary(info: UserInfo, nowSeconds: number): { text: string; low: boolean } | null {
	const parts: string[] = [];
	let low = false;
	if (info.total > 0) {
		const left = Math.max(0, info.total - info.upload - info.download);
		parts.push(left > 0 ? `${formatBytes(left)} left` : 'no data left');
		low = left <= info.total * 0.1;
	}
	if (info.expire > 0) {
		const days = Math.ceil((info.expire - nowSeconds) / 86400);
		parts.push(days <= 0 ? 'expired' : days === 1 ? '1 day left' : `${days} days left`);
		low ||= days <= 3;
	}
	return parts.length ? { text: `Plan: ${parts.join(' · ')}`, low } : null;
}
