# Component Inventory (condensed)
- Backend (src-tauri/src): session, config, runner, store, settings, subscriptions, profile/*, rulesets, health, reliability, network_watch, diagnostics, feedback, crashlog, core (engine download/manager), usage/*, tray, traffic, apps (installed-app listing), clash (controller API), backup, notify.
- Frontend (src/lib/components, src/routes): Home, Profiles, Routing (with AppPicker), Connections, Usage, Diagnostics, Core, Settings, tray popover page.
- Existing: QrDialog (QR generation only), Hysteria2 protocol, redaction in feedback.rs, app picker, kill switch rework, 90% data warning and days left in popover.
- Missing vs wish-list: gateway-based auto-connect, repair/rollback/diagnostics bundle, TUIC and WireGuard, system-proxy mode, App Nap assertion, QR scan and popover server switcher, NetworkExtension.
