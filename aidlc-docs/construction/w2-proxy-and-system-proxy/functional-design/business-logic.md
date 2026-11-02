# W2 Proxy Only and system proxy on Windows: functional design

## Running sing-box (Proxy Only)
- `runner::start(Mode::Proxy)` deletes any old stop file, then starts `sing-box.exe run -c <config>` with no console window (`CREATE_NO_WINDOW`). Both stdout and stderr go to `sing-box.stderr.log` (cleared above 1 MB, as on macOS). The returned pid is sing-box's.
- The process goes into a Job Object with `KILL_ON_JOB_CLOSE`. The app holds the only handle for the run, so if the app exits or crashes, Windows kills sing-box. This replaces `supervisor.sh`'s parent-pid check.
- A watcher thread polls every 500 ms:
  - If the stop file appears, it kills sing-box and waits for it.
  - If sing-box exits on its own, it appends the time and exit status to the stderr log, writes the line to the app log, and puts the system proxy back (`restore_after_crash`).
  - In both cases it then drops the job and deletes the stop file.
- `request_down` is the same as a plain stop; `signal_ready` and `signal_block` do nothing (there is no kill switch on Windows yet).
- `is_alive` checks `OpenProcess` + `GetExitCodeProcess == STILL_ACTIVE`.
- TUN mode returns "TUN mode is not available on Windows yet. Use Proxy Only mode."

## System proxy
- **Store:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings`. These are per-user values, so no administrator rights are needed, and Edge, Chrome and WinINet/WinHTTP-aware apps follow them.
- **Saved values** (`system-proxy-saved.json`, owner-only): `{ port, before: { proxyEnable, proxyServer, proxyOverride, autoConfigUrl } }`. A missing value is saved as `null`, and putting it back deletes it.
- **Set:**
  - If no snapshot exists, save the current values first; on a reconnect, update only the port.
  - Then write `ProxyEnable=1`, `ProxyServer=127.0.0.1:<port>`, `ProxyOverride=<bypass>`, and delete `AutoConfigURL`, because a PAC script would win over the manual proxy.
  - Then call `InternetSetOptionW(SETTINGS_CHANGED)` and `(REFRESH)`, so running programs pick it up.
  - If writing fails, the saved values are written back and the snapshot is removed.
  - `AutoDetect` (WPAD) lives in a binary blob that Windows rewrites itself, so it is not touched.
- **Restore:** only if the current values still point at `127.0.0.1:<saved port>` with `ProxyEnable=1`. Otherwise the user changed them while connected and they are kept. Either way, the snapshot is removed.
- **Bypass list:** built-in plus the user's entries, converted to WinINet patterns (`wininet::override_list`):
  - Host names are kept as written.
  - IPv4 ranges become `a.b.*`. Prefixes that are not whole octets expand to up to 32 patterns; wider ones widen to the octet above.
  - Plain IPv6 addresses are bracketed; IPv6 ranges are dropped, because WinINet cannot express them.
  - `<local>` is always appended.
- **When it is restored:**
  1. Disconnect, quit or Repair: `restore_now`.
  2. sing-box died on its own: the runner's watcher calls `restore_after_crash`.
  3. The app died: a guard process (`kage-vpn.exe --restore-proxy-after <pid> <snapshot>`, detached, no window), started on the first Set. It polls every second and exits once the snapshot is gone; if the app's pid disappears first, it restores and exits.
  4. Anything still left behind: `restore_at_launch`.
- **State shown:** `Set { port, services: ["Windows proxy settings"] }`, `Off`, or `Failed { action, reason }`, the same type as on macOS.

## Business rules and errors
- The option applies only in Proxy Only mode with the proxy login off (shared `applies`).
- Registry errors become "Windows refused to change the proxy settings (<reason>)".
