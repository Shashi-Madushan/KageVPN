# Requirement Verification Questions

Remaining wish-list items. Done already: per-app picker, kill switch while reconnecting, 90% data warning + days left in popover, Hysteria2, IPv6 switch (implemented before the workflow).

Fill each [Answer]: with a letter, then tell me you are done.

## Question 1: Scope
Which items should be built in this round?

A) All except native NetworkExtension (items 3, 4, 5, 8, 10, 11)

B) Only the reliability group: repair service, engine rollback, diagnostics bundle, App Nap (4, 10)

C) Reliability group plus popover server switcher and QR scanning (4, 10, 11)

D) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 2: Auto-connect by network
How should a network be recognised and what should happen?

A) Gateway hardware address (no permissions); per-network action: connect, disconnect, or ignore

B) Gateway hardware address, connect-only on networks the user marks untrusted

C) Skip this item

D) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 3: Diagnostics bundle
What should the exported bundle contain?

A) Redacted app log, sing-box log tail, config with secrets and server addresses removed, versions, settings summary, as a .zip saved where the user picks

B) Same as A, plus a copy to clipboard of a short summary

C) Other (please describe after [Answer]: tag below)

[Answer]: b

## Question 4: Engine rollback
How should rollback to the last working engine work?

A) Keep the previous engine version on disk; mark a version good after a verified connection; one-click rollback on the Core page

B) Do not keep old binaries; re-download a chosen older version on demand

C) Other (please describe after [Answer]: tag below)

[Answer]: a and also adda option to remove them 

## Question 5: Repair VPN service
What should Repair do?

A) Reinstall the root service/helper (one password prompt), clear stale start/stop/pf state, then re-check

B) Only clear stale state, never reinstall

C) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 6: New protocols
Which should be added (each is one module)?

A) TUIC and WireGuard

B) TUIC only

C) WireGuard only

D) None this round

E) Other (please describe after [Answer]: tag below)

[Answer]: d

## Question 7: System-proxy mode
How should it behave?

A) Third mode next to TUN and Proxy-only: sets macOS system proxy via networksetup on all services, restores the previous values on disconnect, and on next launch after a crash

B) Skip this item

C) Other (please describe after [Answer]: tag below)

[Answer]: b

## Question 8: QR scanning
Where should QR codes be read from?

A) Dropped or chosen image file, and a screen capture of the user's screen

B) Dropped or chosen image file only (no screen-recording permission needed)

C) Skip QR scanning, do only the popover server switcher

D) Other (please describe after [Answer]: tag below)

[Answer]: c

## Question 9: Popover server switcher
How should switching appear in the menu-bar popover?

A) A list of the servers (name, latency) of the active group; clicking one switches without a full reconnect when possible

B) Previous/next buttons only

C) Other (please describe after [Answer]: tag below)

[Answer]: a and i think we can chage servers whn vpn is disconected in the menu popover

## Question 10: Delivery
How should the work be delivered?

A) Commits on the dev branch, one per feature, no PR yet

B) One feature branch and a pull request to dev

C) Leave uncommitted in the working tree

D) Other (please describe after [Answer]: tag below)

[Answer]: b

## Question: Security Extensions
Should security extension rules be enforced for this project?

A) Yes — enforce all SECURITY rules as blocking constraints (recommended for production-grade applications)

B) No — skip all SECURITY rules (suitable for PoCs, prototypes, and experimental projects)

X) Other (please describe after [Answer]: tag below)

[Answer]: b

## Question: Property-Based Testing Extension
Should property-based testing (PBT) rules be enforced for this project?

A) Yes — enforce all PBT rules as blocking constraints

B) Partial — enforce PBT rules only for pure functions and serialization round-trips (fits the link parsers)

C) No — skip all PBT rules

X) Other (please describe after [Answer]: tag below)

[Answer]: c

## Question: Resiliency Extensions
Should the resiliency baseline (directional AWS Well-Architected reliability practices, not a certification) be applied?

A) Yes — apply as design-time guidance

B) No — skip (this is a desktop app, not a cloud workload)

X) Other (please describe after [Answer]: tag below)

[Answer]: b
