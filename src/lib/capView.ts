// How a data cap is read, typed and described. Pure, so it can be tested without the UI.

import type { CapPeriod, CapState, CapStatus } from './api';

export type LimitUnit = 'MB' | 'GB';

const MB = 1024 ** 2;
const GB = 1024 ** 3;
export const MIN_LIMIT = MB;
export const MAX_LIMIT = 10 * 1024 ** 4;
const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

/** The limit in bytes, or null when the text is not a number or the size is out of range. */
export function toBytes(value: string | number, unit: LimitUnit): number | null {
	const text = String(value).trim();
	if (!/^\d+(\.\d+)?$/.test(text)) return null;
	const bytes = Math.round(Number(text) * (unit === 'GB' ? GB : MB));
	return bytes >= MIN_LIMIT && bytes <= MAX_LIMIT ? bytes : null;
}

/** Why the typed limit cannot be saved, or null when it can. */
export function limitError(value: string, unit: LimitUnit): string | null {
	if (value.trim() === '') return 'Enter a limit.';
	return toBytes(value, unit) === null ? 'Use a number between 1 MB and 10 TB.' : null;
}

/** The value and unit to show for a stored limit: whole gigabytes where that is exact, else megabytes or decimals. */
export function splitBytes(bytes: number): { value: string; unit: LimitUnit } {
	if (bytes >= GB) {
		const gb = bytes / GB;
		return { value: String(Math.round(gb * 100) / 100), unit: 'GB' };
	}
	return { value: String(Math.max(1, Math.round(bytes / MB))), unit: 'MB' };
}

export const periodWord = (period: CapPeriod) => ({ day: 'daily', week: 'weekly', month: 'monthly' })[period];
export const periodSpan = (period: CapPeriod) => ({ day: 'today', week: 'this week', month: 'this month' })[period];

/** When the count starts over, in words. `periodEnd` is Unix seconds. */
export function resetText(periodEnd: number, period: CapPeriod): string {
	if (period === 'day') return 'resets at midnight';
	if (period === 'week') return 'resets on Monday';
	const end = new Date(periodEnd * 1000);
	return `resets on ${end.getDate()} ${MONTHS[end.getMonth()]}`;
}

export type Tone = 'ok' | 'warn' | 'over' | 'off';

export function barTone(state: CapState): Tone {
	if (state === 'over' || state === 'blocked' || state === 'dismissed') return 'over';
	if (state === 'warn80') return 'warn';
	if (state === 'notCounting') return 'off';
	return 'ok';
}

/** Block now is on offer only while the cap says so and the app is not already blocked. */
export const canBlock = (status: CapStatus) => status.offer === 'pending' && status.state !== 'blocked';
