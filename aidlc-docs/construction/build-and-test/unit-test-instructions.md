# Unit tests
```bash
cd src-tauri && cargo test --lib -- --test-threads=4
npm run test:scripts
```
New tests: core (rollback), repair, diagnostics_bundle (needs macOS `unzip`), networks, appnap, config (IPv6). Use `--test-threads=4`: with the default thread count the run stalled twice in this session, cause not found.
