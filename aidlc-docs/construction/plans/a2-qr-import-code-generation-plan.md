# A2 QR import: Code Generation Plan
Round 5, Part A. Covers FR-A2.1 to FR-A2.6. No functional design stage. This plan is the single source of truth for A2.

## Findings that shape the work
- `ImportDialog.svelte` already accepts a dropped or chosen text file, and pastes links from the clipboard through the clipboard plugin. This avoids the floating "Paste" bubble that macOS shows for the web clipboard API.
- `import_text` reads links and configuration files only, not subscription URLs. Many providers put the subscription URL in their QR code, so a decoded `http(s)` address needs a route to the subscription form on the Profiles page.
- **Decoder:** jsQR 1.4.0 (Apache-2.0, about 250 KB unminified, no dependencies). It reads one code per image, so FR-A2.4 takes the first code. WKWebView and WebView2 have no `BarcodeDetector`, so a library is needed.
- The clipboard plugin has `readImage()`, which returns RGBA bytes. It needs the `clipboard-manager:allow-read-image` permission. Pasting with Cmd/Ctrl+V in the text box gives the image through the paste event, with no extra permission.
- No CSP change: decoding uses `createImageBitmap` on the `File` and a canvas, with no `blob:` image URL.

## Steps
- [x] 1. **Decoder module** (`src/lib/qr.ts`). It holds pure helpers plus the browser glue:
  - `fitWithin(w, h, max)`: the scaled size.
  - `decodeRgba(data, w, h)`: jsQR with `inversionAttempts: 'attemptBoth'`, so white-on-dark codes work.
  - `isImageFile(file)`: by MIME type or extension.
  - `decodeImage(blob)`: tries at a maximum of 1600 px, then at full size up to 4000 px if the first try fails, so small codes in big screenshots are found.
  - Returns the text, or `null` when no code is found.
- [x] 2. **Tests** (`scripts/tests/qr.test.mjs`), using a QR matrix built in the test with the `qrcode` generator:
  - `fitWithin` scales large images and leaves small ones alone.
  - `decodeRgba` reads a generated code with a VLESS link, reads an inverted one, and returns `null` on a blank image.
  - `isImageFile` cases.
- [x] 3. **Import dialog** (`ImportDialog.svelte`):
  - A dropped or chosen image is decoded, and its text goes into the box, so the user sees it before adding. The usual count and validation follow.
  - "Choose file…" also accepts `image/*`.
  - Pasting an image into the box decodes it.
  - A new "Read QR from clipboard" link button uses the plugin's `readImage()`.
  - Errors: "No QR code found in this image", and "The clipboard has no image".
  - The subtitle mentions QR images.
- [x] 4. **Subscription URLs.** When the box holds a single `http(s)` URL and no links, the dialog says "This is a web address, likely a subscription" and shows **Add as subscription**. That button calls a new `onsubscription(url)` prop. `ProfilesPage` handles it by closing the dialog and opening the subscription form with the URL filled in; `SubscriptionForm` gains an `initialUrl` prop.
- [x] 5. **Permission** (`src-tauri/capabilities/*.json`): add `clipboard-manager:allow-read-image`.
- [x] 6. **Docs.** Update the docs-site user guide (adding servers), the changelog entry, the in-app What's new entry and the README feature line. Add `aidlc-docs/construction/a2-qr-import/code/summary.md`.
- [x] 7. **Verify:**
  - `npm run test:scripts`, `svelte-check` and `vite build`.
  - The mock UI (`vite.mock.config.js`) in the browser pane: drop and choose a generated QR PNG, check the box fills in, and check that a QR holding a URL shows the subscription button.
  - The live app is not run against the real VPN.

## Interfaces
- `ImportDialog` gains an `onsubscription?: (url: string) => void` prop.
- `SubscriptionForm` gains an `initialUrl?: string` prop.
- New capability permission: `clipboard-manager:allow-read-image`.
- New dependency: `jsqr`.
