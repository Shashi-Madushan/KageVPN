# Round 5: universal macOS build, QR import from images, Windows support

Please fill in each `[Answer]:` tag with a letter, or with X and your own words. **(Recommended)** marks my suggestion; "use the recommended answers" is a fine reply.

## What the survey found

**Universal macOS build.** `scripts/release.mjs` builds for the host CPU only and names the `.dmg` after `process.arch`. Engine downloads already pick `arm64` or `amd64` at run time (`core/releases.rs`), so the app side needs no change.

**QR import.** Nothing decodes a QR code yet; `qrcode` only draws them. The webview can read an image file, a dropped image or a clipboard image through a canvas, so decoding can happen in the front end without adding Rust image crates (the size worry noted in PRD-v2).

**Windows.** Most of the app is already portable: profiles, config generation, subscriptions, usage storage, routing, the Clash API client, the engine download and verify, the UI. These parts are macOS-only and need a Windows version or must be hidden on Windows:

| Area | macOS today | Windows equivalent |
|---|---|---|
| Starting sing-box as admin for TUN | `osascript` admin prompt + `helper.sh`, or a launchd root service (`runner/`) | UAC prompt, or a Windows service installed once |
| Kill switch | `pfctl` rules in `helper.sh` | Windows Firewall / WFP rules, or sing-box `strict_route` |
| System proxy | `networksetup` per network service (`sysproxy.rs`) | WinINet registry keys + refresh call |
| Engine archive | `.tar.gz`, `darwin-*` | `.zip`, `windows-amd64` / `windows-arm64`, `sing-box.exe` |
| Network change / gateway (`network_watch.rs`, `networks.rs`) | `route`, `scutil`, `arp` | `GetAdaptersAddresses` / `NotifyIpInterfaceChange` |
| Installed apps picker (`apps.rs`) | `.app` bundles, `plist`, `sips` icons | Start Menu shortcuts / uninstall registry, `.exe` icons |
| Process stats (`sampler.rs`) | `ps`, `proc_pidinfo` | `sysinfo` crate or Win32 |
| App Nap (`appnap.rs`) | `NSProcessInfo` activity | not needed |
| Window chrome | overlay title bar with traffic lights, vibrancy "sidebar" | own title bar with Windows min/max/close, Mica |
| Tray popover | under the menu bar, top right | above the taskbar, usually bottom right |
| Shortcuts in the UI | `⌘1`… | `Ctrl+1`… |
| Installer and updates | `.app` / `.dmg`, updater | NSIS or MSI, updater |
| Deep links `kagevpn://` | Info.plist | registry (the deep-link plugin does this) |

Windows code cannot be run on this Mac. It can be compiled and unit-tested on a GitHub Actions Windows runner; anything that touches the network adapter, UAC or the firewall needs a real Windows machine or VM.

---

## Question 1
How should the work be split?

A) Two rounds: first a small PR with the universal build and QR import; then Windows support as its own branch and PRs, in phases (Recommended)

B) One branch with everything

C) Windows first, the other two later

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 2
Where should a QR code be read from?

A) An image file (picker), an image dropped on the import dialog, and an image on the clipboard (Recommended)

B) A, plus "scan the screen" (macOS asks for Screen Recording permission the first time)

C) Image file only

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 3
Which macOS release builds should be produced?

A) One universal app and `.dmg` that runs natively on Apple Silicon and Intel (about twice the size) (Recommended)

B) Two separate builds, Apple Silicon and Intel

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 4
How far should Windows support go in the first version?

A) Phased: phase 1 is everything that does not need admin rights (Proxy Only, system proxy, engine manager, title bar, tray, installer); phase 2 adds TUN mode; phase 3 the kill switch (Recommended)

B) Full parity with macOS before any Windows release

C) Proxy Only and system proxy only, no TUN

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 5
How should TUN mode get admin rights on Windows?

A) A small Windows service installed once with a UAC prompt, then connects need no prompt (like the macOS root service) (Recommended)

B) A UAC prompt on every TUN connect, sing-box started elevated

C) Ask the user to run the whole app as administrator

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 6
What should the kill switch use on Windows?

A) sing-box's own `strict_route` in TUN mode now, a firewall-based kill switch (WFP) later in phase 3 (Recommended)

B) Windows Firewall rules through `netsh advfirewall`, like the `pfctl` design

C) Hide the kill switch on Windows

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 7
How should the main window look on Windows?

A) A custom title bar drawn by the app with Windows 11-style minimize, maximize and close buttons on the right, Mica background, same sidebar layout (Recommended)

B) The standard Windows title bar above the app

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 8
Which Windows installer and CPUs?

A) NSIS `.exe` installer (per-user, works with the updater), x64 only to start (Recommended)

B) NSIS for x64 and ARM64

C) MSI

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 9
How will Windows builds be tested?

A) I have a Windows PC or VM: compile and unit-test in GitHub Actions, then I run the manual checks on Windows (Recommended)

B) No Windows machine: GitHub Actions only (compile, unit tests, installer build); live behaviour stays unverified

X) Other (please describe after [Answer]: tag below)

[Answer]: X - No GitHub Actions for now. The user builds the Windows version and tests it on their own Windows device (changed at requirements review).

## Question: Security Extensions
Should security extension rules be enforced for this project?

A) Yes — enforce all SECURITY rules as blocking constraints (recommended for production-grade applications)

B) No — skip all SECURITY rules (suitable for PoCs, prototypes, and experimental projects)

X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question: Property-Based Testing Extension
Should property-based testing (PBT) rules be enforced for this project?

A) Yes — enforce all PBT rules as blocking constraints (recommended for projects with business logic, data transformations, serialization, or stateful components)

B) Partial — enforce PBT rules only for pure functions and serialization round-trips (suitable for projects with limited algorithmic complexity)

C) No — skip all PBT rules (suitable for simple CRUD applications, UI-only projects, or thin integration layers with no significant business logic)

X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question: Resiliency Extensions
Should the resiliency baseline be applied to this project?

A) Yes — apply the resiliency baseline as directional best practices and design-time guidance

B) No — skip the resiliency baseline (earlier rounds chose No)

X) Other (please describe after [Answer]: tag below)

[Answer]: B
