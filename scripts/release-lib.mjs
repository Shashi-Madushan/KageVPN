// Pure helpers for scripts/release.mjs, kept free of side effects so they can be tested.

const SEMVER = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;

export function assertVersion(version) {
	if (!SEMVER.test(version)) throw new Error(`"${version}" is not a version like 1.2.3`);
	return version;
}

/** Reads `pub const NAME: Option<&str> = Some("…");` (or `None`) from app_config.rs. */
export function readRustOption(source, name) {
	const match = source.match(new RegExp(`pub const ${name}: Option<&str> = (None|Some\\("([^"]*)"\\));`));
	if (!match) throw new Error(`${name} was not found in app_config.rs`);
	return match[1] === 'None' ? null : match[2];
}

export function readCargoVersion(toml) {
	const match = toml.match(/^\[package\][^[]*?^version\s*=\s*"([^"]+)"/ms);
	if (!match) throw new Error('no package version in Cargo.toml');
	return match[1];
}

export function setCargoVersion(toml, version) {
	return toml.replace(/(^\[package\][^[]*?^version\s*=\s*)"[^"]+"/ms, `$1"${version}"`);
}

export function setJsonVersion(text, version) {
	const json = JSON.parse(text);
	json.version = version;
	return `${JSON.stringify(json, null, text.includes('\t') ? '\t' : 2)}\n`;
}

/** `darwin-aarch64` or `darwin-x86_64`, the keys the updater looks up. */
export function platformKey(arch = process.arch) {
	const cpu = { arm64: 'aarch64', x64: 'x86_64' }[arch];
	if (!cpu) throw new Error(`unsupported CPU ${arch}`);
	return `darwin-${cpu}`;
}

/** The Rust target that builds one app for Apple Silicon and Intel. */
export const UNIVERSAL_TARGET = 'universal-apple-darwin';
/** A universal build serves both updater keys. */
export const UNIVERSAL_PLATFORMS = ['darwin-aarch64', 'darwin-x86_64'];
const UNIVERSAL_PARTS = ['aarch64-apple-darwin', 'x86_64-apple-darwin'];

/** The targets a universal build needs that `rustup target list --installed` does not list. */
export function missingTargets(installed) {
	const have = new Set(installed.split(/\s+/).filter(Boolean));
	return UNIVERSAL_PARTS.filter((t) => !have.has(t));
}

/** The updater key for a Windows x64 build. */
export const WINDOWS_PLATFORM = 'windows-x86_64';

/** The installer Tauri's NSIS bundler writes, under `bundle/nsis/`. */
export function windowsInstallerName(productName, version) {
	return `${productName}_${version}_x64-setup.exe`;
}

/** Safe file name for a published artifact (no spaces, includes version and CPU). */
export function artifactName(slug, version, platform, extension) {
	return `${slug}_${version}_${platform.replace('darwin-', '')}.${extension}`;
}

/**
 * The update manifest the app downloads. Pass `platform` for one key or `platforms` for several
 * that share the file (a universal build). Platforms from an earlier build of the same version
 * are kept, so two machines can each add their own.
 */
export function buildManifest({ version, notes, platform, platforms, signature, url, previous, now = new Date() }) {
	const keep = previous && previous.version === version ? previous.platforms : {};
	const entry = { signature: signature.trim(), url };
	const added = Object.fromEntries((platforms ?? [platform]).map((key) => [key, entry]));
	return {
		version,
		notes,
		pub_date: now.toISOString(),
		platforms: { ...keep, ...added }
	};
}

export function githubBase(repo, version) {
	if (!/^[\w.-]+\/[\w.-]+$/.test(repo)) throw new Error(`"${repo}" should look like owner/repo`);
	return `https://github.com/${repo}/releases/download/v${version}`;
}

export function githubFeed(repo) {
	return `https://github.com/${repo}/releases/latest/download/latest.json`;
}

/** Whether the public key compiled into the app is the one belonging to the signing key. */
export function keysMatch(configured, publicKeyFile) {
	return !!configured && configured.trim() === publicKeyFile.trim();
}
