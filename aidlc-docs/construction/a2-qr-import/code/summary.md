# A2 QR import: summary

## Changed
- `src/lib/qr.ts` (new): `fitWithin`, `decodeRgba` (jsQR, light and dark codes), `isImageFile`, `decodeImage` (1600 px pass, then up to 4000 px), `webAddress`.
- `ImportDialog.svelte`: images dropped, chosen or pasted are decoded into the box; **QR from clipboard** reads the clipboard image through the plugin; a lone web address shows "A web address, likely a subscription" and **Add as subscription** (new `onsubscription` prop).
- `SubscriptionForm.svelte`: `initialUrl` prop. `ProfilesPage.svelte`: opens the form with the address (keyed so an open form takes the new one).
- `src-tauri/capabilities/default.json`: `clipboard-manager:allow-read-image`.
- `package.json`: `jsqr` 1.4.0 (Apache-2.0); dev `qrcode-generator` 1.4.4 (MIT) for tests.
- Tests: `scripts/tests/qr.test.mjs` (6).
- Docs: user guide, README, changelog, What's new (0.4.0 entry, shown as new once the version is bumped).

## Verified
- Script tests 49 passed, `svelte-check` 0 errors, `vite build` ok, `cargo test --lib` 450 passed.
- Mock UI in the browser pane: a dropped PNG with a VLESS link fills the box ("1 link ready to add"); a PNG with an https address shows **Add as subscription**, which opens the subscription form with the address filled in.

## Not verified here
- **QR from clipboard** in the real app (needs the Tauri clipboard plugin; the mock has none). Check: copy a QR image (for example Cmd+Ctrl+Shift+4 on a code), open Add server, click QR from clipboard.
- HEIC images: WKWebView decodes them; WebView2 on Windows may not.
