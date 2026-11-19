# W6 Windows service: Functional Design

Answers used: the recommended option (A) for all six questions in `plans/w6-windows-service-functional-design-plan.md`. They match the approved requirements (FR-B2.1 to B2.3, B2.6) and the service boundary in the application design.

## Entities
- **Service "KageHelper"**: the same `kage-vpn.exe`, copied to `%ProgramData%\Kage\kage-vpn.exe` and started by the Service Control Manager with `--service --user-sid <SID>`. Runs as LocalSystem, start type demand.
- **Install folder `%ProgramData%\Kage`**: writable only by SYSTEM and Administrators (the install script sets the ACL and removes inherited rights). Holds `kage-vpn.exe`, `engine\sing-box.exe`, `engine\sing-box.sha256`, `run\config.json`, `run\sing-box.log`.
- **Pipe `\\.\pipe\com.apzur.kage.helper`**: byte pipe, one client at a time. DACL (SDDL): `D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GRGW;;;<installing user SID>)`. Remote clients are rejected.
- **Messages** (one JSON object per line, UTF-8):
  - Request: `{"op":"start","config":<object>,"kill_switch_ips":null|"..."}`, `{"op":"stop"}`, `{"op":"status"}`, `{"op":"version"}`.
  - Reply: `{"ok":true,"pid":123}` or `{"ok":true,"running":bool,"pid":..,"version":".."}` or `{"ok":false,"error":"..."}`.
- **Run**: at most one sing-box child inside a Job Object (`KILL_ON_JOB_CLOSE`), started from the engine copy, with `CREATE_NO_WINDOW`.

## Logic
1. **Install / update / repair** (`daemon::install`, one UAC prompt): the app builds a PowerShell script and runs it elevated (`Start-Process powershell -Verb RunAs -Wait`). The script creates the folder, sets its ACL, copies `kage-vpn.exe` and the engine, writes the engine's SHA-256, runs `sc stop`/`sc delete` for an old service, then `sc create KageHelper ... start= demand obj= LocalSystem`, `sc description`, and `sc start`. A declined prompt returns `Cancelled`. After it, `installed()` must be true or the install reports failure.
2. **Installed?** `installed()` asks the SCM whether service `KageHelper` exists. `binary_current(engine)` compares the SHA-256 of the app's engine with `engine\sing-box.sha256`. `script_current()` compares the service exe's size/hash with the running app's exe (the service is the app, so "current" = same version).
3. **Start (TUN connect)**: the app ensures the service is installed and current (install/update otherwise), starts it if stopped (`sc start` needs admin, so the service is set to be startable by the installing user via its security descriptor `sc sdset`, see rules), connects to the pipe, sends `start`, and keeps the pipe open for the whole run. The reply carries sing-box's pid.
4. **Service side start**: validate the config (rules below), re-hash `engine\sing-box.exe`, refuse on a mismatch, write the config to `run\config.json`, spawn sing-box into the Job, reply with the pid. A second `start` while running stops the first run first.
5. **Stop**: `stop` request (or disconnect) kills the child and drops the Job. `request_stop`/`request_down` from the app send `stop` over the held pipe; with no pipe they reconnect and send it.
6. **Client gone (FR-B2.6)**: when the pipe read fails or hits EOF, the service kills the child. The app crashing or quitting closes the handle, so this needs no app cooperation.
7. **Status / diagnostics**: `status` reports running and pid; `version` reports the service's app version. The app's Diagnostics row shows: not installed, installed and stopped, running, out of date (offer update), engine mismatch (offer repair).
8. **Remove**: elevated script: `sc stop`, `sc delete`, delete `%ProgramData%\Kage`.

## Rules
- **R1 Pipe access**: only SYSTEM, Administrators and the installing user's SID. The SID is passed on the service command line by the install script and is read once at service start; it is not read from any user-writable place.
- **R2 Config validation** (service refuses with an error, never edits to "fix"): valid JSON object; must contain `inbounds` with a `tun` inbound; the keys `external_ui`, `external_ui_download_url`, `external_ui_download_detour`, `cache_file`, `v2ray_api`, `certificate_path`, `certificate_directory_path`, `key_path`, `private_key_path`, `config_path`, `state_directory`, `plugin`, `plugin_opts`, `ntp` may not appear anywhere in it (same list as the macOS helper); `log.output` is replaced by the service with `run\sing-box.log` (the app's own value is ignored); `route.rule_set` entries of type `local` need an absolute `.srs` path, others of type `remote` are refused; config size at most 4 MiB.
- **R3 Engine integrity**: the service only ever runs `%ProgramData%\Kage\engine\sing-box.exe`, and only if its SHA-256 equals the recorded one at every start.
- **R4 Folder ownership**: the install folder is admin-only; the service writes only inside it.
- **R5 Startable by the user**: after `sc create`, `sc sdset KageHelper` grants the installing user `RPWPLCRC` (start, stop-not-needed, query) so TUN connects never need a UAC prompt. Stopping is done through the pipe.
- **R6 One client**: a second pipe client while one is connected is refused with `busy`.
- **R7 No console windows**: every process uses `CREATE_NO_WINDOW`.

## Errors
- UAC declined: `Cancelled`, nothing changed.
- Service missing, stopped and cannot start: "the Kage helper service did not start; use Repair in Diagnostics".
- Pipe busy / not found after start: retried for up to 10 s, then a launch error.
- Engine hash mismatch: refused; Diagnostics offers Repair.
- sing-box exits early: the reply is an error with the last lines of `run\sing-box.log`.

## Frontend (for W7)
- Mode choice shows TUN when `platform.tun` (now true) and the engine is installed.
- First TUN connect with no service: confirm dialog "Kage needs to install a helper service. Windows will ask for permission once."
- Diagnostics: a "Helper service" row with state, Repair and Remove buttons (existing root-service UI on macOS is reused).
