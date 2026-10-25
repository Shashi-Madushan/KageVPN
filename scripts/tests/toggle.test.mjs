import { test } from 'node:test';
import assert from 'node:assert/strict';
import { saveToggle } from '../../src/lib/toggle.ts';

const click = (checked) => ({ currentTarget: { checked } });

test('a saved toggle keeps the click', async () => {
	const event = click(true);
	let saved = null;
	await saveToggle(event, async (on) => ((saved = on), true));
	assert.equal(saved, true);
	assert.equal(event.currentTarget.checked, true);
});

test('a failed save puts the box back', async () => {
	for (const checked of [true, false]) {
		const event = click(checked);
		await saveToggle(event, async () => false);
		assert.equal(event.currentTarget.checked, !checked);
	}
});
