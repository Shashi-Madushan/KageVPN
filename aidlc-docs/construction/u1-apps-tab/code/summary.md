# U1 Apps tab: code summary
- `settings.rs`: `RuleOrigin` (manual or cap) on `Rule`; old files load as manual.
- `config.rs`, `commands.rs`: `validate_rules` refuses an enabled Block rule for this app or the engine; called from `set_routing`.
- `apps.rs`, `paths.rs`, `commands.rs`, `lib.rs`: `app_icon` command (`sips` converts the bundle's `.icns` to a 48 px PNG, cached in `app-icons/`).
- `src/lib/appRules.ts`: pure rule logic (`appChoice`, `setAppChoice`, `otherAppRules`, `blockedApps`, presets).
- `src/lib/components/AppsTab.svelte`, `RoutingPage.svelte`: the Apps tab; edits the page's draft, applied by Save changes.
- Tests: 3 Rust tests in `tests/config.rs`, 3 in `tests/apps.rs`, 7 in `scripts/tests/app-rules.test.mjs`.

## Manual check (not run here)
1. Routing, Apps: switch Slack to Block, Save changes, reconnect; Slack has no network, Connections shows it as blocked.
2. Try to block Kage VPN: Save shows the refusal.
3. Choose "VPN only for..." and pick one app; only that app's traffic goes through the VPN.
4. With app tracking on, a row shows "x MB this period".
