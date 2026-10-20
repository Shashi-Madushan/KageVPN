import { test } from 'node:test';
import assert from 'node:assert/strict';
import { activePreset, appChoice, applyPreset, blockedApps, mergeCapRules, otherAppRules, reconcileDraft, setAppChoice } from '../../src/lib/appRules.ts';

const slack = '/Applications/Slack.app';
const rule = (value, action, extra = {}) => ({ id: value + action, enabled: true, kind: 'app', value, action, ...extra });
let n = 0;
const id = () => `new-${++n}`;

test('an app with no rule follows the default', () => {
	assert.equal(appChoice([], 'app', slack), null);
	assert.equal(appChoice([rule('/x.app', 'block')], 'app', slack), null);
	assert.equal(appChoice([rule(slack, 'block', { enabled: false })], 'app', slack), null);
});

test('the choice is read back, and the first matching rule wins', () => {
	const rules = [rule(slack, 'direct'), rule(slack, 'block')];
	assert.deepEqual(appChoice(rules, 'app', slack), { action: 'direct', origin: 'manual' });
	assert.equal(appChoice([rule(slack, 'block', { origin: 'cap' })], 'app', slack).origin, 'cap');
});

test('setting a choice replaces the old one and goes on top', () => {
	const rules = [rule('/a.app', 'proxy'), rule(slack, 'direct'), rule('/b.app', 'block')];
	const next = setAppChoice(rules, 'app', slack, 'block', id);
	assert.equal(next.length, 3);
	assert.deepEqual([next[0].value, next[0].action], [slack, 'block']);
	assert.deepEqual(next.slice(1).map((r) => r.value), ['/a.app', '/b.app']);
	assert.equal(rules.length, 3, 'the input is not changed');
});

test('clearing a choice removes only the manual rule', () => {
	const cap = rule(slack, 'block', { origin: 'cap', id: 'cap-rule' });
	const next = setAppChoice([rule(slack, 'direct'), cap], 'app', slack, null, id);
	assert.deepEqual(next.map((r) => r.id), ['cap-rule']);
});

test('process-name rules and uninstalled apps are listed apart', () => {
	const rules = [rule(slack, 'direct'), { id: 'p', enabled: true, kind: 'processName', value: 'curl', action: 'direct' }, rule('/Gone.app', 'block'), { id: 'd', enabled: true, kind: 'domain', value: 'a.com', action: 'block' }];
	const other = otherAppRules(rules, new Set([slack]));
	assert.deepEqual(other.map((r) => r.value), ['curl', '/Gone.app']);
});

test('blocked apps say who blocked them and skip repeats and overridden rules', () => {
	const rules = [rule(slack, 'block', { origin: 'cap' }), rule(slack, 'block'), rule('/b.app', 'block'), rule('/c.app', 'direct'), rule('/c.app', 'block')];
	assert.deepEqual(blockedApps(rules), [
		{ kind: 'app', value: slack, origin: 'cap' },
		{ kind: 'app', value: '/b.app', origin: 'manual' }
	]);
});

test('presets set the default and keep per-app rules', () => {
	const settings = { mode: 'global', finalAction: 'proxy', rules: [rule(slack, 'direct')] };
	const only = applyPreset(settings, 'only');
	assert.deepEqual([only.mode, only.finalAction, only.rules.length], ['rule', 'direct', 1]);
	assert.equal(activePreset(only), 'only');
	assert.equal(activePreset(applyPreset(settings, 'except')), 'except');
	assert.equal(activePreset(settings), null);
	assert.equal(settings.mode, 'global', 'the input is not changed');
});

test('a stale draft takes the stored cap blocks and keeps the users edits', () => {
	const stored = [rule(slack, 'block', { origin: 'cap' }), rule('/a.app', 'proxy')];
	const draft = [rule('/a.app', 'direct'), rule('/old.app', 'block', { origin: 'cap' })];
	const merged = mergeCapRules(draft, stored);
	assert.deepEqual(merged.map((r) => [r.value, r.action]), [[slack, 'block'], ['/a.app', 'direct']]);
});

test('a page copy follows the stored settings until the user edits it', () => {
	const stored = { mode: 'rule', finalAction: 'proxy', rules: [rule('/a.app', 'direct')] };
	const first = reconcileDraft(null, stored, '');
	assert.equal(first.changed, true);
	assert.deepEqual(first.draft, stored);
	assert.notEqual(first.draft, stored, 'a copy, not the stored object');

	const same = reconcileDraft(first.draft, stored, first.synced);
	assert.equal(same.changed, false);

	// Something else changes the stored settings while the copy is unedited: the copy follows.
	const changedStored = { ...stored, mode: 'global' };
	const followed = reconcileDraft(first.draft, changedStored, first.synced);
	assert.equal(followed.draft.mode, 'global');

	// The user edited the copy: it keeps the edit and takes only the stored cap blocks.
	const edited = { ...first.draft, rules: [rule('/b.app', 'block')] };
	const withCap = { ...stored, rules: [rule(slack, 'block', { origin: 'cap' }), rule('/a.app', 'direct')] };
	const merged = reconcileDraft(edited, withCap, first.synced);
	assert.deepEqual(merged.draft.rules.map((r) => r.value), [slack, '/b.app']);
	assert.equal(merged.draft.mode, 'rule');
	assert.equal(merged.synced, JSON.stringify(withCap));
});
