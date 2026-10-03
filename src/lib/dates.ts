import type { Bucket } from './api';

const DAY = 86400;

export type Preset = 'today' | 'yesterday' | 'week' | 'month30' | 'thisMonth' | 'lastMonth' | 'cycle' | 'custom';

export const PRESETS: { value: Preset; label: string }[] = [
	{ value: 'today', label: 'Today' },
	{ value: 'yesterday', label: 'Yesterday' },
	{ value: 'week', label: 'Last 7 days' },
	{ value: 'month30', label: 'Last 30 days' },
	{ value: 'thisMonth', label: 'This month' },
	{ value: 'lastMonth', label: 'Last month' },
	{ value: 'cycle', label: 'Billing cycle' },
	{ value: 'custom', label: 'Custom' }
];

const startOfDay = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate());
const addDays = (d: Date, days: number) => new Date(d.getFullYear(), d.getMonth(), d.getDate() + days);
const seconds = (d: Date) => Math.floor(d.getTime() / 1000);

export function toDateInput(d: Date): string {
	const pad = (n: number) => String(n).padStart(2, '0');
	return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function fromDateInput(value: string): Date {
	const [y, m, d] = value.split('-').map(Number);
	return new Date(y, (m || 1) - 1, d || 1);
}

export function rangeFor(
	preset: Preset,
	custom: { from: string; to: string },
	cycle: { start: number; end: number } | null
): [number, number] {
	const today = startOfDay(new Date());
	switch (preset) {
		case 'today':
			return [seconds(today), seconds(addDays(today, 1))];
		case 'yesterday':
			return [seconds(addDays(today, -1)), seconds(today)];
		case 'week':
			return [seconds(addDays(today, -6)), seconds(addDays(today, 1))];
		case 'month30':
			return [seconds(addDays(today, -29)), seconds(addDays(today, 1))];
		case 'thisMonth':
			return [
				seconds(new Date(today.getFullYear(), today.getMonth(), 1)),
				seconds(new Date(today.getFullYear(), today.getMonth() + 1, 1))
			];
		case 'lastMonth':
			return [
				seconds(new Date(today.getFullYear(), today.getMonth() - 1, 1)),
				seconds(new Date(today.getFullYear(), today.getMonth(), 1))
			];
		case 'cycle':
			return cycle ? [cycle.start, cycle.end] : [seconds(new Date(today.getFullYear(), today.getMonth(), 1)), seconds(addDays(today, 1))];
		case 'custom': {
			const from = startOfDay(fromDateInput(custom.from));
			const to = addDays(startOfDay(fromDateInput(custom.to)), 1);
			return to > from ? [seconds(from), seconds(to)] : [seconds(from), seconds(addDays(from, 1))];
		}
	}
}

export function autoBucket(from: number, to: number): Bucket {
	const days = (to - from) / DAY;
	if (days <= 2.01) return 'hour';
	if (days <= 93) return 'day';
	if (days <= 400) return 'week';
	return 'month';
}

export function bucketLabel(start: number, bucket: Bucket): string {
	const d = new Date(start * 1000);
	switch (bucket) {
		case 'hour':
			return d.toLocaleString(undefined, { month: 'short', day: 'numeric', hour: 'numeric' });
		case 'day':
			return d.toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric' });
		case 'week':
			return `Week of ${d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}`;
		case 'month':
			return d.toLocaleDateString(undefined, { month: 'long', year: 'numeric' });
	}
}

export function shortLabel(start: number, bucket: Bucket): string {
	const d = new Date(start * 1000);
	switch (bucket) {
		case 'hour':
			return d.toLocaleTimeString(undefined, { hour: 'numeric' });
		case 'day':
		case 'week':
			return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
		case 'month':
			return d.toLocaleDateString(undefined, { month: 'short' });
	}
}

export function rangeDays(from: number, to: number): number {
	return Math.max(1, (to - from) / DAY);
}
