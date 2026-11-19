#!/usr/bin/env node
// Builds a signed release and prepares (and optionally publishes) the update feed.
//
//   npm run release -- --version 0.2.0 --github owner/repo --notes "What changed"      (build + stage)
//   npm run release -- --version 0.2.0 --github owner/repo --notes-file CHANGES.md --publish
//
// Nothing is published unless --publish is given. See docs/RELEASING.md.

import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { homedir, tmpdir } from 'node:os';
import { dirname, join, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import {
	artifactName,
	assertVersion,
	buildManifest,
	githubBase,
	githubFeed,
	keysMatch,
	missingTargets,
	readCargoVersion,
	readRustOption,
	setCargoVersion,
	setJsonVersion,
	UNIVERSAL_PLATFORMS,
	UNIVERSAL_TARGET,
	WINDOWS_PLATFORM,
	windowsInstallerName
} from './release-lib.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const file = (...p) => join(root, ...p);
const KEYCHAIN_SERVICE = 'apzur-vpn-updater-key';
const slug = 'kage-vpn';

const { values: opt } = parseArgs({
	options: {
		version: { type: 'string' },
		github: { type: 'string' },
		'base-url': { type: 'string' },
		notes: { type: 'string' },
		'notes-file': { type: 'string' },
		publish: { type: 'boolean', default: false },
		draft: { type: 'boolean', default: false },
		'skip-build': { type: 'boolean', default: false },
		dmg: { type: 'boolean', default: false },
		'allow-dirty': { type: 'boolean', default: false },
		'dry-run': { type: 'boolean', default: false }
	}
});

const fail = (message) => {
	console.error(`\nrelease: ${message}`);
	process.exit(1);
};
const run = (cmd, args, options = {}) => {
	const result = spawnSync(cmd, args, { cwd: root, stdio: 'inherit', ...options });
	if (result.status !== 0) fail(`${cmd} ${args.join(' ')} failed`);
};
const capture = (cmd, args) => spawnSync(cmd, args, { cwd: root, encoding: 'utf8' });

// ── Inputs ──────────────────────────────────────────────────────────────────
const confPath = file('src-tauri', 'tauri.conf.json');
const cargoPath = file('src-tauri', 'Cargo.toml');
const packagePath = file('package.json');
const configSource = readFileSync(file('src-tauri', 'src', 'app_config.rs'), 'utf8');

const current = JSON.parse(readFileSync(confPath, 'utf8')).version;
const version = assertVersion(opt.version ?? current);
const productName = JSON.parse(readFileSync(confPath, 'utf8')).productName;
// macOS releases are one universal app for Apple Silicon and Intel; Windows releases are an x64 installer.
const windows = process.platform === 'win32';
const platform = windows ? WINDOWS_PLATFORM : 'universal';
const osLabel = windows ? 'Windows (x64 installer)' : 'macOS (universal: Apple Silicon and Intel)';

const keyPath = process.env.APZUR_UPDATER_KEY_PATH ?? join(homedir(), '.tauri', 'apzur-vpn-updater.key');
if (!existsSync(keyPath)) fail(`signing key not found at ${keyPath} (see docs/RELEASING.md)`);

function signingPassword() {
	if (process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD !== undefined) return process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD;
	if (windows) fail('no signing password: set TAURI_SIGNING_PRIVATE_KEY_PASSWORD');
	const found = capture('security', ['find-generic-password', '-s', KEYCHAIN_SERVICE, '-w']);
	if (found.status !== 0) fail(`no signing password: set TAURI_SIGNING_PRIVATE_KEY_PASSWORD or add Keychain item "${KEYCHAIN_SERVICE}"`);
	return found.stdout.trim();
}

// The key compiled into the app must be the one that signs updates, or installed apps could
// never accept them.
const compiledKey = readRustOption(configSource, 'APP_UPDATE_PUBKEY');
if (!keysMatch(compiledKey, readFileSync(`${keyPath}.pub`, 'utf8'))) {
	fail('APP_UPDATE_PUBKEY in app_config.rs does not match the signing key. Shipping this would strand every installed copy.');
}
const feed = readRustOption(configSource, 'APP_UPDATE_FEED');

const baseUrl = opt['base-url'] ?? (opt.github ? githubBase(opt.github, version) : null);
if (opt.publish && !opt.github) fail('--publish needs --github owner/repo');
const notes = opt['notes-file'] ? readFileSync(resolve(opt['notes-file']), 'utf8').trim() : (opt.notes ?? `Release ${version}`);

const dirty = capture('git', ['status', '--porcelain']).stdout.trim();
if (dirty && !opt['allow-dirty'] && !opt['dry-run']) fail('the working tree has uncommitted changes (commit them or pass --allow-dirty)');

const missing = windows ? [] : missingTargets(capture('rustup', ['target', 'list', '--installed']).stdout ?? '');
console.log(`Release ${productName} ${version} for ${osLabel}`);
if (missing.length) {
	const fix = `rustup target add ${missing.join(' ')}`;
	if (!opt['dry-run']) fail(`a universal build needs more Rust targets. Run: ${fix}`);
	console.log(`  missing     : ${missing.join(', ')} (run: ${fix})`);
}
console.log(`  signing key : ${keyPath} (matches the key in app_config.rs)`);
console.log(`  download URL: ${baseUrl ?? '(none, latest.json will not be written)'}`);
console.log(`  app feed    : ${feed ?? 'not set in app_config.rs yet'}`);
if (opt.github) console.log(`  feed to set : ${githubFeed(opt.github)}`);
if (opt['dry-run']) {
	console.log('\nDry run: nothing was changed.');
	process.exit(0);
}

// ── Version, build, sign ────────────────────────────────────────────────────
if (version !== current || readCargoVersion(readFileSync(cargoPath, 'utf8')) !== version) {
	writeFileSync(confPath, setJsonVersion(readFileSync(confPath, 'utf8'), version));
	writeFileSync(packagePath, setJsonVersion(readFileSync(packagePath, 'utf8'), version));
	writeFileSync(cargoPath, setCargoVersion(readFileSync(cargoPath, 'utf8'), version));
	console.log(`Set version ${version} in tauri.conf.json, package.json and Cargo.toml`);
}

const bundleDir = windows ? file('src-tauri', 'target', 'release', 'bundle') : file('src-tauri', 'target', UNIVERSAL_TARGET, 'release', 'bundle');
// The file the updater downloads: the app archive on macOS, the installer itself on Windows.
const update = windows ? join(bundleDir, 'nsis', windowsInstallerName(productName, version)) : join(bundleDir, 'macos', `${productName}.app.tar.gz`);
// The CLI checks the signature it just made against `plugins.updater.pubkey`, so hand it the key
// from app_config.rs for this build only. tauri.conf.json keeps an empty placeholder. It goes in a
// file because quoting JSON on a Windows command line is unreliable.
const overridePath = join(tmpdir(), `kage-release-${process.pid}.json`);
writeFileSync(overridePath, JSON.stringify({ plugins: { updater: { pubkey: compiledKey } } }));
const keyOverride = ['--config', overridePath];
const signingEnv = { ...process.env, TAURI_SIGNING_PRIVATE_KEY: keyPath, TAURI_SIGNING_PRIVATE_KEY_PASSWORD: signingPassword() };
// npm is a .cmd script on Windows, which Node only starts through a shell.
const npm = (args, options = {}) => spawnSync('npm', args, { cwd: root, stdio: 'inherit', env: signingEnv, shell: windows, ...options });
if (!opt['skip-build']) {
	if (windows) {
		if (npm(['run', 'tauri', '--', 'build', '--bundles', 'nsis', ...keyOverride]).status !== 0) fail('the Windows build failed');
	} else {
		// The app bundle is all the updater needs. The .dmg (first installs only) is a separate,
		// optional step because macOS can refuse the disk-image styling when run without a desktop session.
		const target = ['--target', UNIVERSAL_TARGET];
		if (npm(['run', 'tauri', '--', 'build', ...target, '--bundles', 'app', ...keyOverride]).status !== 0) fail('the macOS build failed');
		if (opt.dmg && npm(['run', 'tauri', '--', 'build', ...target, '--bundles', 'dmg', ...keyOverride]).status !== 0) {
			console.warn('\nrelease: the .dmg could not be built, continuing without it');
		}
	}
}
rmSync(overridePath, { force: true });
if (!existsSync(update) || !existsSync(`${update}.sig`)) fail(`the build did not produce a signed update at ${update}`);

// ── Stage the files ─────────────────────────────────────────────────────────
const out = file('release', version);
mkdirSync(out, { recursive: true });
const stagedUpdate = artifactName(slug, version, platform, windows ? 'setup.exe' : 'app.tar.gz');
copyFileSync(update, join(out, stagedUpdate));
const staged = [join(out, stagedUpdate)];

const dmg = join(bundleDir, 'dmg', `${productName}_${version}_universal.dmg`);
if (!windows && existsSync(dmg)) {
	const name = artifactName(slug, version, platform, 'dmg');
	copyFileSync(dmg, join(out, name));
	staged.push(join(out, name));
}

if (baseUrl) {
	const manifestPath = join(out, 'latest.json');
	// The other OS may have published this version already (from another machine); its entries
	// are kept, so the two releases add up instead of replacing each other.
	if (!existsSync(manifestPath) && opt.github) {
		capture('gh', ['release', 'download', `v${version}`, '--repo', opt.github, '--pattern', 'latest.json', '--dir', out]);
	}
	const previous = existsSync(manifestPath) ? JSON.parse(readFileSync(manifestPath, 'utf8')) : null;
	const manifest = buildManifest({
		version,
		notes,
		platforms: windows ? [WINDOWS_PLATFORM] : UNIVERSAL_PLATFORMS,
		signature: readFileSync(`${update}.sig`, 'utf8'),
		url: `${baseUrl}/${stagedUpdate}`,
		previous
	});
	writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
	staged.push(manifestPath);
}
console.log(`\nStaged in ${out}:\n${staged.map((f) => `  ${f.replace(`${root}${sep}`, '')}`).join('\n')}`);

// ── Publish (only on request) ───────────────────────────────────────────────
if (!opt.publish) {
	console.log('\nNot published. Re-run with --publish to upload to GitHub Releases.');
	process.exit(0);
}
const tag = `v${version}`;
const exists = capture('gh', ['release', 'view', tag, '--repo', opt.github]).status === 0;
if (exists) {
	run('gh', ['release', 'upload', tag, ...staged, '--clobber', '--repo', opt.github]);
} else {
	run('gh', ['release', 'create', tag, ...staged, '--repo', opt.github, '--title', `${productName} ${version}`, '--notes', notes, ...(opt.draft ? ['--draft'] : [])]);
}
console.log(`\nPublished ${tag}. The app reads ${githubFeed(opt.github)}`);
