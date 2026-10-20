# U1 Apps tab: Code Generation Plan
Covers FR1 and FR3 of `app-controls-requirements.md`. No functional design stage (skipped in the execution plan). This plan is the single source of truth for U1.

## Findings that shape the work
- The Routing page edits a `draft` copy of the settings and applies it with its existing Save (and reconnect). The Apps tab edits the same draft, so "one confirmation per batch of changes" (FR1.5) is the page's own Save. No new apply path.
- App rules store the bundle path (`kind: 'app'`); process-name rules (`kind: 'processName'`) stay valid and show as extra rows (FR1.4).
- `AppPicker.svelte` and `api.installedApps()` already give the installed apps; there is no icon source yet.
- `set_routing` (`commands.rs`) validates DNS and providers but not rules. A block rule for Kage itself or for the engine would cut the app's own health checks and downloads (NFR2), so it needs a refusal.
- There is no front-end test runner; `npm run test:scripts` runs `node --test` over `scripts/tests/*.test.mjs`, and Node 25 can import `.ts` files that have no runtime type syntax. Rule logic is therefore written as a pure `.ts` module and tested there.

## Steps
- [ ] 1. **Rule origin** (`settings.rs`, `api.ts`): add `RuleOrigin { Manual, Cap }` (serde default `Manual`) as `Rule.origin`; unchanged saved settings must load. Used by U2/U3 to tell a cap block from a manual one (FR3.2). Test: old JSON without `origin` loads as `Manual`; round trip keeps `Cap`.
- [ ] 2. **Refuse self-blocking rules** (`config.rs`, `commands.rs::set_routing`): `validate_rules(rules)` rejects an enabled `block` rule (and a `direct` one is allowed) whose app bundle id/path, or process name, is Kage itself (`com.apzur.kage`, the running bundle) or the engine (`sing-box`, `kage-vpn`). Error text names the app and says why. Tests: Kage bundle blocked is refused, Slack blocked passes, disabled rule passes, process name `sing-box` refused.
- [ ] 3. **App icons** (`apps.rs`, `commands.rs`, `lib.rs`): `app_icon(path)` command returns a small PNG as a `data:` URL. Reads `CFBundleIconFile` from `Info.plist` (already parsed with the `plist` crate), converts the `.icns` with `sips` to 32 px PNG into `<data dir>/app-icons/<hash>.png`, caches it, returns `None` on any failure (the UI falls back to the initial letter). Pure helper `icon_file_name(&plist::Dictionary)` (adds `.icns` when missing) is unit-tested; `sips` itself is not.
- [ ] 4. **Rule logic module** (`src/lib/appRules.ts`): pure functions, no Svelte: `appChoice(rules, key)` (current VPN/Direct/Block or null), `setAppChoice(rules, key, choice)` (replace or remove the app's rule, keeping the cap origin rule untouched, new rule on top), `ruleKey(rule)`, `unknownRuleRows(rules, apps)` (rules whose app is not installed or that are process-name rules), and `applyPreset(settings, 'except' | 'only')`: "except" sets mode `rule` with `finalAction: 'proxy'`, "only" sets `finalAction: 'direct'`; existing per-app rules are kept.
- [ ] 5. **Rule logic tests** (`scripts/tests/app-rules.test.mjs`): set/replace/clear a choice, the choice is read back, other rules keep their order, presets set the default and keep overrides, process-name rows are listed, a cap-origin rule is not removed by clearing the manual choice.
- [ ] 6. **Apps tab** (`src/lib/components/AppsTab.svelte`, `RoutingPage.svelte`): new tab "Apps" with count. Search box; preset buttons; rows with icon (lazy, only for visible rows), name, current usage this period (from `api.usageTop`, hidden if recording is off) and a three-way switch (Use VPN / Direct / Block) using `Segmented.svelte`; "Follows default" shown when no rule. Rows with a rule are listed first, then A to Z; at most 80 rows rendered, with "Show more". Process-name rows are labelled with the process name. Edits change `draft.rules`; the page's Save applies them. Blocking Kage or the engine shows the backend refusal message on Save.
- [ ] 7. **Blocked apps strip** (same file): a short list on top of the tab of apps currently on Block, each with an Unblock button, so a blocked app is easy to find (FR3.2). Origin `Cap` rows are labelled "blocked by cap" (used from U2).
- [ ] 8. **Docs**: docs-site guide page for Routing (Apps tab), changelog entry (dated today), `aidlc-docs/construction/u1-apps-tab/code/summary.md`.
- [ ] 9. **Verify**: `cargo test --lib -- --test-threads=4`, `cargo clippy --all-targets -- -D warnings`, `svelte-check`, `vite build`, `npm run test:scripts`. Manual check list in the summary (switch an app, Save, reconnect, Connections page shows the route).

## Interfaces added
- Tauri command `app_icon(path: string) -> Option<string>`.
- `Rule.origin: 'manual' | 'cap'` (settings and `api.ts`).
- `set_routing` can now return an error for a rule that would block Kage or the engine.

## Out of scope for U1
Caps (U2/U3), usage bars beyond the plain usage figure, alerts (U4).
