# Network Extension mode: requirement questions

Round 3 (branch `feature/network-extension`). Please fill in each `[Answer]:` tag with a letter, or with X and your own words. Add notes under any answer if you like.

## Background (what the choices depend on)

**How TUN mode works today.** The app writes a sing-box config, then a root helper (`runner/helper.sh`, started through an administrator prompt or the optional launchd root service) runs the downloaded sing-box binary, which creates the `utun` interface. The kill switch is `pfctl` rules loaded by the same helper. Audit finding O1: the root helper trusts files a user-level process can write.

**How a Network Extension (NE) mode would work.** A packet tunnel provider (Swift) runs sing-box as a library (libbox, built from sing-box's `experimental/libbox` with gomobile into `Libbox.xcframework`). macOS creates the tunnel for it, so no root helper, no `pfctl` script and no administrator prompt. The app controls it through `NETunnelProviderManager`, which also offers VPN On Demand rules and a system-wide "VPN" entry in System Settings. This is how sing-box's own macOS app (SFM) works.

**Facts that limit the options:**
- **Signing.** NE needs a restricted entitlement, granted only through an Apple Developer Program membership and a provisioning profile. Outside the Mac App Store the provider must be a **system extension** signed with Developer ID (entitlement value `packet-tunnel-provider-systemextension`) and **notarized** to run on Macs with SIP on. Without a membership it can only run on a development Mac with SIP partly disabled.
- **First run.** A system extension is activated from an app in `/Applications`, and the user approves it once in System Settings (Privacy and Security), plus a "add VPN configuration" prompt.
- **Engine.** An NE cannot run the separately downloaded `sing-box` binary the Engine page manages; libbox is compiled into the extension, so its version is fixed per app release.
- **Licence.** sing-box and libbox are GPL-3.0-or-later, with an extra term: "no derivative work may use the name or imply association with this application without prior consent". Kage is MIT and today only downloads sing-box (it does not link it). Shipping libbox inside the app bundle changes that.
- **Swift.** The provider and the libbox bridge are Swift (an Xcode target embedded in the Tauri bundle); the Rust app talks to `NETunnelProviderManager` through a small bridge.

Sources: Apple Developer Forums on Developer ID packet tunnel providers ([1](https://developer.apple.com/forums/thread/134202), [2](https://developer.apple.com/forums/thread/664126)), [NEOnDemandRuleConnect](https://developer.apple.com/documentation/networkextension/neondemandruleconnect), [sing-box Apple clients](https://sing-box.sagernet.org/clients/apple/), [sing-box-for-apple](https://github.com/SagerNet/sing-box-for-apple) (has `SystemExtension` and `SFM.System` targets), [sing-box LICENSE](https://github.com/SagerNet/sing-box/blob/testing/LICENSE).

---

## Question 1
Do you have (or will you get) an Apple Developer Program membership for this?

A) Yes, I have one now

B) I will get one before this is built; plan and prototype assuming it

C) Not yet; plan only, and build a prototype that runs on a development Mac with SIP partly disabled

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 2
How should the NE build be distributed?

A) Direct download only (Developer ID, notarized, system extension), like today's GitHub releases

B) Mac App Store only (app extension, sandboxed app)

C) Both: direct download first, App Store later

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 3
How should NE mode relate to today's root-helper TUN mode?

A) Replace it: Full System mode always uses the NE; the root helper, root service and `pfctl` kill switch are removed

B) NE becomes the default Full System mode, and the root helper stays as a fallback for builds or Macs where the extension is not approved

C) NE is an opt-in third mode next to Full System (TUN) and Proxy Only, until it has proven itself

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 4
What happens to the Engine page (downloaded sing-box versions) once the NE embeds libbox?

A) The NE uses the bundled libbox; Proxy Only mode keeps using the downloaded engine, so the Engine page stays for Proxy Only

B) Everything uses the bundled libbox (Proxy Only too, through the extension or the app); the Engine page and engine downloads are removed

C) Keep both and let the user choose per mode

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 5
Embedding libbox (GPL-3.0-or-later) in the app bundle affects Kage's MIT licence. How do you want to handle it?

A) Relicense Kage to GPL-3.0-or-later (the source is already public)

B) Keep the app MIT and ship the extension as a separate GPL component with its source, and get advice on whether that is enough

C) Get legal advice first; treat licensing as a blocking open question in the plan

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 6
sing-box's licence asks that derivative works not use its name or imply association without consent. The README and app say "powered by sing-box". What should the plan assume?

A) Ask the sing-box author for consent to keep the attribution

B) Keep a neutral credit only (for example the licence notice and a link), with no "powered by" wording

C) Leave as is for now and list it as an open question

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 7
What should VPN On Demand do in the first NE release?

A) Nothing yet; only manual connect, with On Demand later

B) An "Always on" switch: macOS reconnects the VPN on any network whenever something needs the internet

C) Tie it to the existing networks feature: connect on networks marked Connect (and unknown ones, if chosen), stay off on networks marked Disconnect

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 8
How should the kill switch work in NE mode?

A) Use the system's built-in protection (`includeAllNetworks`, traffic outside the tunnel is dropped by macOS), and drop the `pfctl` version in NE mode

B) Keep a `pfctl` kill switch alongside the NE

C) No kill switch in the first NE release

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 9
Which features must work in NE mode before it ships? (Several letters allowed, for example A, B, D)

A) Connect, disconnect, server switching, Auto (fastest) and failover

B) Routing rules and ready-made lists, including per-app rules and data caps (needs process matching inside the extension, which has to be proven)

C) Live traffic, Connections page, usage tracking and health checks (through the sing-box controller API)

D) Reconnect, dead-tunnel restart and connect by network

E) Logs and the crash log

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 10
What is the lowest macOS version the NE build should support? (sing-box's own macOS app needs 13.)

A) macOS 13 Ventura

B) macOS 14 Sonoma

C) macOS 15 Sequoia

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 11
What should happen for existing users who installed the root service?

A) On the first run of the NE build, explain the change and offer to remove the root service (one password prompt)

B) Leave the root service installed and unused; they can remove it in Settings

C) Remove it automatically once the extension is approved

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 12
What should this round deliver?

A) A plan only: requirements, design and units, no code

B) A plan plus a spike that proves the riskiest parts first: libbox in a system extension embedded in the Tauri bundle, started from the app, with process matching checked

C) A plan and the full implementation

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question: Security Extensions
Should security extension rules be enforced for this project?

A) Yes — enforce all SECURITY rules as blocking constraints (recommended for production-grade applications)

B) No — skip all SECURITY rules (suitable for PoCs, prototypes, and experimental projects)

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question: Resiliency Extensions
Should the resiliency baseline be applied to this project?

**What this extension is.** Enabling it applies a set of **directional, design-time best practices** for building resilient systems, derived from the **AWS Well-Architected Framework (Reliability Pillar)** and resilience-review guidance. It steers requirements, design, and code toward fault tolerance, high availability, observability, and recoverability — covering 15 practice areas across business goals, change management, observability, high availability, disaster recovery, and continuous improvement.

**What this extension is NOT.** Enabling it does **not** make your workload production-ready, nor does it certify or guarantee any availability, RTO, or RPO target. It is a **starting point** that scaffolds good resiliency decisions early — it is not a substitute for a formal **AWS Well-Architected Review** of the built system.

Treat the output as a well-grounded **first draft of your resiliency posture** to build on and validate — not a finished, production-certified result.

A) Yes — apply the resiliency baseline as directional best practices and design-time guidance (recommended for business-critical workloads, as an informed starting point that you can validate and harden before go-live)

B) No — skip the resiliency baseline (suitable for PoCs, prototypes, and experimental projects where rapid iteration matters more than reliability)

X) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question: Property-Based Testing Extension
Should property-based testing (PBT) rules be enforced for this project?

A) Yes — enforce all PBT rules as blocking constraints (recommended for projects with business logic, data transformations, serialization, or stateful components)

B) Partial — enforce PBT rules only for pure functions and serialization round-trips (suitable for projects with limited algorithmic complexity)

C) No — skip all PBT rules (suitable for simple CRUD applications, UI-only projects, or thin integration layers with no significant business logic)

X) Other (please describe after [Answer]: tag below)

[Answer]: 
