import assert from 'node:assert/strict';
import { test } from 'node:test';
import {
	artifactName,
	assertVersion,
	buildManifest,
	githubBase,
	githubFeed,
	keysMatch,
	platformKey,
	readCargoVersion,
	readRustOption,
	setCargoVersion,
	setJsonVersion
} from '../release-lib.mjs';

test('versions must be semver', () => {
	assert.equal(assertVersion('1.2.3'), '1.2.3');
	assert.equal(assertVersion('1.2.3-beta.1'), '1.2.3-beta.1');
	for (const bad of ['1.2', 'v1.2.3', '1.2.x', '']) assert.throws(() => assertVersion(bad), bad);
});

test('reads set and unset options from app_config.rs', () => {
	const src = 'pub const APP_UPDATE_FEED: Option<&str> = None;\npub const APP_UPDATE_PUBKEY: Option<&str> = Some("abc123==");\n';
	assert.equal(readRustOption(src, 'APP_UPDATE_FEED'), null);
	assert.equal(readRustOption(src, 'APP_UPDATE_PUBKEY'), 'abc123==');
	assert.throws(() => readRustOption(src, 'NOPE'));
});

test('the package version is read and replaced without touching dependencies', () => {
	const toml = '[package]\nname = "x"\nversion = "0.1.0"\n\n[dependencies]\nserde = { version = "1" }\n';
	assert.equal(readCargoVersion(toml), '0.1.0');
	const next = setCargoVersion(toml, '0.2.0');
	assert.equal(readCargoVersion(next), '0.2.0');
	assert.ok(next.includes('serde = { version = "1" }'));
});

test('json files keep their indentation when the version changes', () => {
	assert.equal(setJsonVersion('{\n\t"version": "0.1.0"\n}\n', '0.2.0'), '{\n\t"version": "0.2.0"\n}\n');
	assert.equal(setJsonVersion('{\n  "version": "0.1.0"\n}\n', '0.2.0'), '{\n  "version": "0.2.0"\n}\n');
});

test('platform keys and artifact names match what the updater looks up', () => {
	assert.equal(platformKey('arm64'), 'darwin-aarch64');
	assert.equal(platformKey('x64'), 'darwin-x86_64');
	assert.throws(() => platformKey('ia32'));
	assert.equal(artifactName('kage-vpn', '1.0.0', 'darwin-aarch64', 'app.tar.gz'), 'kage-vpn_1.0.0_aarch64.app.tar.gz');
});

test('the manifest carries the signature and url for this platform', () => {
	const m = buildManifest({
		version: '1.0.0',
		notes: 'Hi',
		platform: 'darwin-aarch64',
		signature: 'SIG\n',
		url: 'https://h/x.tar.gz',
		now: new Date('2030-01-02T03:04:05Z')
	});
	assert.deepEqual(m, {
		version: '1.0.0',
		notes: 'Hi',
		pub_date: '2030-01-02T03:04:05.000Z',
		platforms: { 'darwin-aarch64': { signature: 'SIG', url: 'https://h/x.tar.gz' } }
	});
});

test('a second CPU is added to the same version, but an older version is replaced', () => {
	const first = buildManifest({ version: '1.0.0', notes: '', platform: 'darwin-aarch64', signature: 'a', url: 'u1' });
	const both = buildManifest({ version: '1.0.0', notes: '', platform: 'darwin-x86_64', signature: 'b', url: 'u2', previous: first });
	assert.deepEqual(Object.keys(both.platforms).sort(), ['darwin-aarch64', 'darwin-x86_64']);
	const next = buildManifest({ version: '1.1.0', notes: '', platform: 'darwin-x86_64', signature: 'c', url: 'u3', previous: both });
	assert.deepEqual(Object.keys(next.platforms), ['darwin-x86_64']);
});

test('github locations', () => {
	assert.equal(githubBase('me/app', '1.2.3'), 'https://github.com/me/app/releases/download/v1.2.3');
	assert.equal(githubFeed('me/app'), 'https://github.com/me/app/releases/latest/download/latest.json');
	assert.throws(() => githubBase('not a repo', '1.0.0'));
});

test('a mismatched or missing public key is caught before shipping', () => {
	assert.ok(keysMatch('KEY==', 'KEY==\n'));
	assert.ok(!keysMatch('OTHER==', 'KEY=='));
	assert.ok(!keysMatch(null, 'KEY=='));
});
