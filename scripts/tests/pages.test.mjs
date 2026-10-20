import { test } from 'node:test';
import assert from 'node:assert/strict';
import { pageForKey, pageOrder, shortcutFor } from '../../src/lib/pages.ts';

test('the Apps page sits right after Routing', () => {
	assert.equal(pageOrder.indexOf('apps'), pageOrder.indexOf('routing') + 1);
	assert.equal(new Set(pageOrder).size, pageOrder.length, 'no page twice');
});

test('Cmd+1 to Cmd+9 follow the sidebar order and the tenth page has no number', () => {
	assert.equal(shortcutFor('home'), 1);
	assert.equal(shortcutFor('apps'), 5);
	assert.equal(shortcutFor('logs'), 9);
	assert.equal(shortcutFor('settings'), null);
	for (let digit = 1; digit <= 9; digit++) assert.equal(shortcutFor(pageForKey(String(digit))), digit);
});

test('other keys open nothing', () => {
	for (const key of ['0', 'a', '10', '', ' ', 'Enter']) assert.equal(pageForKey(key), null, JSON.stringify(key));
});
