import type { Latency } from './app.svelte';

const pad = (n: number) => String(n).padStart(2, '0');

export function formatDuration(totalSeconds: number): string {
	const h = Math.floor(totalSeconds / 3600);
	const m = Math.floor((totalSeconds % 3600) / 60);
	return `${pad(h)}:${pad(m)}:${pad(totalSeconds % 60)}`;
}

const UNITS = ['B', 'KB', 'MB', 'GB', 'TB'];

export function formatBytes(bytes: number): string {
	let value = bytes;
	let unit = 0;
	while (value >= 1024 && unit < UNITS.length - 1) {
		value /= 1024;
		unit++;
	}
	return `${unit === 0 ? value : value.toFixed(value < 10 ? 1 : 0)} ${UNITS[unit]}`;
}

export const formatRate = (bytesPerSecond: number) => `${formatBytes(bytesPerSecond)}/s`;

export type LatencyTone = 'ok' | 'warn' | 'bad' | 'idle';

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
