// Per-app routing choices, as pure functions over the rule list so they can be tested without the
// UI. An app's choice is the first enabled rule that names it; rules put in place by a data cap
// (`origin: 'cap'`) are never removed by clearing the user's own choice.

import type { Rule, RouteAction, RoutingSettings } from './api';

export type AppKind = 'app' | 'processName';
export type AppChoice = { action: RouteAction; origin: 'manual' | 'cap' };
export type Preset = 'except' | 'only';

const isAppRule = (r: Rule) => r.kind === 'app' || r.kind === 'processName';
const names = (r: Rule, kind: AppKind, value: string) => r.kind === kind && r.value === value;
const originOf = (r: Rule) => r.origin ?? 'manual';

/** What the rules currently do with this app, or null when it follows the routing default. */
export function appChoice(rules: Rule[], kind: AppKind, value: string): AppChoice | null {
	const rule = rules.find((r) => r.enabled && names(r, kind, value));
	return rule ? { action: rule.action, origin: originOf(rule) } : null;
}

/**
 * Sets (or with `null` clears) the user's own choice for an app. The new rule goes on top so it
 * wins over broader ones. Cap rules for the same app are left alone.
 */
export function setAppChoice(rules: Rule[], kind: AppKind, value: string, action: RouteAction | null, newId: () => string): Rule[] {
	const rest = rules.filter((r) => !(names(r, kind, value) && originOf(r) === 'manual'));
	if (action === null) return rest;
	return [{ id: newId(), enabled: true, kind, value, action, origin: 'manual' }, ...rest];
}

/** Rules that name an app the Apps list cannot show: process names, and bundles no longer installed. */
export function otherAppRules(rules: Rule[], installedPaths: Set<string>): Rule[] {
	return rules.filter((r) => isAppRule(r) && (r.kind === 'processName' || !installedPaths.has(r.value)));
}

/** Every app currently on Block, with who blocked it. */
export function blockedApps(rules: Rule[]): { kind: AppKind; value: string; origin: 'manual' | 'cap' }[] {
	const seen = new Set<string>();
	const out: { kind: AppKind; value: string; origin: 'manual' | 'cap' }[] = [];
	for (const r of rules) {
		if (!r.enabled || !isAppRule(r) || r.action !== 'block') continue;
		const key = `${r.kind}:${r.value}`;
		if (seen.has(key) || appChoice(rules, r.kind as AppKind, r.value)?.action !== 'block') continue;
		seen.add(key);
		out.push({ kind: r.kind as AppKind, value: r.value, origin: originOf(r) });
	}
	return out;
}

/**
 * "except": everything uses the VPN unless an app says otherwise. "only": nothing does unless an
 * app says so. Both use Rules mode, and every per-app rule is kept.
 */
export function applyPreset(settings: RoutingSettings, preset: Preset): RoutingSettings {
	return { ...settings, mode: 'rule', finalAction: preset === 'except' ? 'proxy' : 'direct' };
}

/**
 * The draft of the Routing page after a data cap changed the stored rules behind it: the stored
 * cap blocks replace the draft's own, and the user's edits stay.
 */
export function mergeCapRules(draftRules: Rule[], storedRules: Rule[]): Rule[] {
	return [...storedRules.filter((r) => originOf(r) === 'cap'), ...draftRules.filter((r) => originOf(r) !== 'cap')];
}

/**
 * Keeps a page's editable copy of the routing settings in step with the stored ones. An unedited
 * copy follows the stored settings; an edited one keeps the user's edits and only takes the stored
 * cap blocks. `synced` is the stored settings, as text, the last time they were copied in.
 */
export function reconcileDraft(
	draft: RoutingSettings | null,
	stored: RoutingSettings,
	synced: string
): { draft: RoutingSettings; synced: string; changed: boolean } {
	const storedText = JSON.stringify(stored);
	if (!draft) return { draft: structuredClone(stored), synced: storedText, changed: true };
	if (storedText === synced) return { draft, synced, changed: false };
	if (JSON.stringify(draft) === synced) return { draft: structuredClone(stored), synced: storedText, changed: true };
	return { draft: { ...draft, rules: mergeCapRules(draft.rules, stored.rules) }, synced: storedText, changed: true };
}
