# U3 summary
- `appnap.rs`: NSProcessInfo activity (UserInitiatedAllowingIdleSystemSleep) held while the status is Connected, released otherwise and on exit; hooked in `session::update`. New direct deps `objc2` and `objc2-foundation` (already in Cargo.lock via tauri).
- Popover: server option labels show measured latency. The switcher itself pre-existed.
- Manual check: `pmset -g assertions` while connected should list "Kage VPN is connected" (not run).
