# U1 Recovery tools: code summary
- Rollback: `CoreSettings.{good_version,previous_version}`, `activate`, `mark_good`, `rollback_target`, `core::remove_old`; commands `core_rollback`, `core_remove_old`; `health.rs` marks the engine good after a verified tunnel; Core page buttons.
- Repair: `src-tauri/src/repair.rs` (`repair_service`, `clear_stale`), `runner::forget_repair_attempts`; button + confirmation on the Diagnostics page.
- Diagnostics bundle: `src-tauri/src/diagnostics_bundle.rs` (redaction reuses `feedback::redact`, config key scrub, subscription URLs hidden, own stored-zip writer); saved to `~/Downloads`, revealed in Finder, summary copied.
- Tests: tests/core.rs (4 new), tests/repair.rs, tests/diagnostics_bundle.rs (zip validated by macOS `unzip`).
- Deviation: no zip crate (all 2.x-8.x yanked); Downloads instead of a save dialog (approved).
- Manual check: root-service reinstall prompt cannot be exercised without root.
