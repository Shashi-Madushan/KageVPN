import { test } from 'node:test';
import assert from 'node:assert/strict';
import { bypassLines, formatBytes, formatRate, formatRateOrDash, importMessage, isWireGuardFile, planSummary } from '../../src/lib/format.ts';

const GB = 1024 ** 3;
const DAY = 86400;
const now = 1_000_000;

test('a plan reads as data left and days left', () => {
	const plan = planSummary({ upload: 0, download: 20 * GB, total: 100 * GB, expire: now + 30 * DAY }, now);
	assert.equal(plan.text, 'Plan: 80 GB left · 30 days left');
	assert.equal(plan.low, false);
});

test('the plan is low at 10 percent of the data or 7 days from the end', () => {
	assert.equal(planSummary({ upload: 0, download: 90 * GB, total: 100 * GB, expire: 0 }, now).low, true);
	assert.equal(planSummary({ upload: 0, download: 89 * GB, total: 100 * GB, expire: 0 }, now).low, false);
	assert.equal(planSummary({ upload: 0, download: 0, total: 0, expire: now + 7 * DAY }, now).low, true);
	assert.equal(planSummary({ upload: 0, download: 0, total: 0, expire: now + 8 * DAY }, now).low, false);
});

test('unlimited data, one day, expired and an empty plan', () => {
	assert.equal(planSummary({ upload: 0, download: 5, total: 0, expire: now + DAY }, now).text, 'Plan: 1 day left');
	assert.equal(planSummary({ upload: 0, download: 0, total: 0, expire: now - 5 }, now).text, 'Plan: expired');
	assert.equal(planSummary({ upload: 0, download: 0, total: 0, expire: 0 }, now), null);
	assert.equal(planSummary({ upload: 60 * GB, download: 60 * GB, total: 100 * GB, expire: 0 }, now).text, 'Plan: no data left');
});

test('speeds are rounded, never raw decimals', () => {
	assert.equal(formatBytes(896.0519740123), '896 B');
	assert.equal(formatBytes(39.98000999), '40 B');
	assert.equal(formatBytes(NaN), '0 B');
	assert.equal(formatBytes(-5), '0 B');
	assert.equal(formatRate(3.6 * 1024), '3.6 KB/s');
	assert.equal(formatRateOrDash(0), '—');
	assert.equal(formatRateOrDash(0.4), '—');
	assert.equal(formatRateOrDash(2048), '2.0 KB/s');
});

test('an import message names skips and notes', () => {
	assert.equal(importMessage({ added: 2, duplicates: 0, failed: [], notes: [] }), 'Imported 2 profiles.');
	assert.equal(
		importMessage({ added: 1, duplicates: 0, failed: [], notes: ['HK: no speeds in the link, using 10 Mbps up and 50 Mbps down'] }),
		'Imported 1 profile, note (HK: no speeds in the link, using 10 Mbps up and 50 Mbps down).'
	);
	assert.equal(importMessage({ added: 3, duplicates: 0, failed: ['x'], notes: ['a: b', 'c: d'] }), 'Imported 3 profiles, 1 skipped (x), 2 notes (a: b).');
});

test('a wg-quick file is told apart from other text', () => {
	assert.equal(isWireGuardFile('# exported\n[Interface]\nPrivateKey = x\nAddress = 10.0.0.2/32\n\n[Peer]\nPublicKey = y\nEndpoint = h:51820\n'), true);
	assert.equal(isWireGuardFile('[interface]\r\nPrivateKey = x\r\n[peer]\r\nEndpoint = h:1\r\n'), true);
	assert.equal(isWireGuardFile('[Interface]\nPrivateKey = x\n'), false, 'no peer');
	assert.equal(isWireGuardFile('[Peer]\nEndpoint = h:1\n[Interface]\n'), false, 'interface not first');
	assert.equal(isWireGuardFile('vless://x@h:443'), false);
});

test('bypass entries are read one per line or comma-separated', () => {
	assert.deepEqual(bypassLines(' *.corp.example \n\n10.20.0.0/16, intranet\n'), ['*.corp.example', '10.20.0.0/16', 'intranet']);
	assert.deepEqual(bypassLines(''), []);
});
