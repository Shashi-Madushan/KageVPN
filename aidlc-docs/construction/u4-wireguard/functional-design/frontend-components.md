# U4 WireGuard: frontend components

The editor fields come from the backend; only the Add server dialog changes.

## Server editor (`ProfileDialog.svelte`, no code change)
Fields declared by the WireGuard module (Question 8):

| Section | Field | Kind | Notes |
|---|---|---|---|
| Connection | Private key | secret | required, pattern: 44-character base64 |
| Connection | Server public key | text, mono | required, same pattern |
| Connection | Local addresses | list | required, placeholder `172.16.0.2/32, fd01::2/128` |
| Security | Pre-shared key | secret | optional |
| Advanced | MTU | text, mono | default 1408, pattern 1280 to 1500 |
| Advanced | Keepalive (seconds) | text, mono | optional |
| Advanced | Reserved bytes | text, mono | optional, placeholder `0,0,0`, hint: "Cloudflare WARP needs these; other servers do not" |

Port defaults to 51820 for a new WireGuard server (the untouched-port rule from U3 moves it when switching protocol). Numbers typed as text are read by the backend (the Hysteria speed approach).

## Add server dialog (`ImportDialog.svelte`)
- **Recognises wg-quick text:** `isConfig` also matches text with an `[Interface]` and a `[Peer]` section, so Import is enabled for a pasted or dropped `.conf`.
- **Keeps the dropped file's name:** `loadFile` stores `file.name`; typing or pasting clears it. Import calls `app.importText(text, fileName)`.
- **Accepts `.conf` in the file picker** if the dialog has an `accept` list.
- Placeholder text mentions `wireguard://` and `.conf`.

## API
- `api.importText(text, fileName?)` → Tauri `import_text { text, fileName }`; `fileName` optional, so existing callers (onboarding) are unchanged.
- `ImportSummary.notes` (from U2) shows the ignored-lines note in the toast.

## Connect error
The R9 message reaches the user through the existing connect error path (no new component).

## Interaction flows
1. Drop `warp-tokyo.conf` → text fills, Import enabled → toast "Imported 1 profile, note (warp-tokyo: DNS and AllowedIPs from the file are not used; …)" → list shows "warp-tokyo · WireGuard".
2. Paste a `wireguard://` link → imported like any link.
3. New server → Protocol: WireGuard → port 51820, MTU 1408 → enter keys and address → save.
4. Connect with an engine lacking `with_wireguard` and a WireGuard server chosen → error names the Engine page.
