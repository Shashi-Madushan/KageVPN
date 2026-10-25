import { test } from 'node:test';
import assert from 'node:assert/strict';
import { barTone, canBlock, limitError, periodSpan, periodWord, resetText, splitBytes, toBytes } from '../../src/lib/capView.ts';

const GB = 1024 ** 3;
const MB = 1024 ** 2;

test('a typed limit becomes bytes, or null', () => {
	assert.equal(toBytes('2', 'GB'), 2 * GB);
	assert.equal(toBytes(' 0.5 ', 'GB'), GB / 2);
	assert.equal(toBytes('500', 'MB'), 500 * MB);
	assert.equal(toBytes('1', 'MB'), MB);
	assert.equal(toBytes('0.5', 'MB'), null, 'below 1 MB');
	assert.equal(toBytes('10240', 'GB'), 10 * 1024 ** 4, 'exactly 10 TB');
	assert.equal(toBytes('10241', 'GB'), null, 'above 10 TB');
	for (const bad of ['', 'abc', '-3', '1e3', '2,5', '1.2.3']) assert.equal(toBytes(bad, 'GB'), null, bad);
});

test('the error text says what to fix', () => {
	assert.equal(limitError('', 'GB'), 'Enter a limit.');
	assert.match(limitError('x', 'GB'), /between 1 MB and 10 TB/);
	assert.equal(limitError('2', 'GB'), null);
});

test('a stored limit is shown in the unit that reads best and round trips', () => {
	assert.deepEqual(splitBytes(2 * GB), { value: '2', unit: 'GB' });
	assert.deepEqual(splitBytes(1.5 * GB), { value: '1.5', unit: 'GB' });
	assert.deepEqual(splitBytes(512 * MB), { value: '512', unit: 'MB' });
	assert.deepEqual(splitBytes(MB), { value: '1', unit: 'MB' });
	for (const bytes of [2 * GB, 1.5 * GB, 512 * MB, 3 * MB]) {
		const { value, unit } = splitBytes(bytes);
		assert.equal(toBytes(value, unit), bytes);
	}
});

test('periods and resets are described in words', () => {
	assert.equal(periodWord('day'), 'daily');
	assert.equal(periodSpan('month'), 'this month');
	assert.equal(resetText(0, 'day'), 'resets at midnight');
	assert.equal(resetText(0, 'week'), 'resets on Monday');
	const nov1 = new Date(2026, 10, 1).getTime() / 1000;
	assert.equal(resetText(nov1, 'month'), 'resets on 1 Nov');
});

test('tones and the block offer follow the state', () => {
	assert.equal(barTone('ok'), 'ok');
	assert.equal(barTone('warn80'), 'warn');
	assert.equal(barTone('over'), 'over');
	assert.equal(barTone('blocked'), 'over');
	assert.equal(barTone('notCounting'), 'off');
	assert.equal(canBlock({ offer: 'pending', state: 'over' }), true);
	assert.equal(canBlock({ offer: 'pending', state: 'blocked' }), false);
	assert.equal(canBlock({ offer: 'dismissed', state: 'dismissed' }), false);
	assert.equal(canBlock({ offer: 'none', state: 'ok' }), false);
});
