# W6 Windows service: Functional Design Plan and Questions

Round 5, Part B phase 2 (branch `feature/windows-tun`, from `dev`). Covers FR-B2.1 to FR-B2.3 and FR-B2.6 of `inception/requirements/universal-qr-windows-requirements.md`, on the service boundary in `inception/application-design/windows-platform-design.md`.

Note: the phase 1 build-and-test report stopped before phase 2 until Windows device results arrived. The user has now asked to build phase 2 (2026-11-19). Nothing in it can be run on the Mac, so it is checked with `cargo check`, `clippy` and unit tests of the pure parts only, and needs the Windows device checks listed in `docs/WINDOWS.md` before it is trusted.

## Plan
- [ ] 1. Domain entities: service, pipe, request and reply messages, installed layout in `%ProgramData%\Kage`
- [ ] 2. Business logic: install, start, stop, status, client disconnect, repair, remove, update
- [ ] 3. Business rules: config validation, pipe access, engine integrity, ownership of files
- [ ] 4. Error handling and edge cases (UAC declined, service stopped, pipe busy, engine mismatch)
- [ ] 5. Frontend components (W7 reads this): TUN choice, Diagnostics service row, repair and remove

## Questions

## Question 1
Who may talk to the service over the pipe?

A) SYSTEM, Administrators and only the user account that installed the service (the requirements' choice)

B) SYSTEM, Administrators and every interactive user on the PC

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended)

## Question 2
How is the service installed and updated?

A) One UAC prompt runs a PowerShell script that copies `kage-vpn.exe` and the engine to `%ProgramData%\Kage`, then runs `sc create` and `sc start` (the design's choice)

B) A separate installer step in the NSIS setup

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended; works from the portable exe and the installer alike)

## Question 3
Should the service start at boot or only when asked?

A) On demand (`start= demand`): the app starts it on the first TUN connect; no idle service at boot

B) Automatic at boot

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended; the requirements and design already say demand)

## Question 4
What validates the config the service is sent?

A) An allow-list: reject any `log.output`, any file path outside `%ProgramData%\Kage`, any `experimental.cache_file.path` elsewhere, and any outbound of type other than the app's known kinds; written to the service's own folder before it runs

B) Only reject `log.output`

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended)

## Question 5
When the app crashes or quits while TUN is up:

A) The service stops sing-box as soon as the pipe client disconnects (FR-B2.6)

B) The service keeps sing-box running until told to stop

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended)

## Question 6
How does the service learn the engine is the one the app installed?

A) The app records the engine's SHA-256 at install. The service re-hashes its own copy in `%ProgramData%\Kage\engine` before every start and refuses on a mismatch

B) Trust the copy once, hash only at install

X) Other (please describe after [Answer]: tag below)

[Answer]: A (recommended)
