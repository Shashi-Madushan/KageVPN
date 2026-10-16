# U2 Auto-connect by network: Functional Design Plan

## Steps
- [x] Analyze unit context (FR1; `network_watch.rs` already detects network changes and wake)
- [x] Draft the design questions below
- [x] Collect answers and resolve ambiguities (b, a, a, a, a, a; no contradictions)
- [x] Write `business-logic-model.md`, `business-rules.md`, `domain-entities.md`, `frontend-components.md`
- [x] Present completion message

## Working assumptions (change them in the answers if wrong)
- A network is identified by the default gateway's hardware address, read with `route get default` and `arp`. While the tunnel is up the default route goes through the tunnel, so the physical gateway is read from the interface-scoped route; no Location permission is needed.
- Known networks are stored in settings as `{ id: gateway address, name, action }`; action is connect, disconnect or ignore.

## Question 1: First time on a network
When the Mac joins a network it has never seen, what should happen?

A) Nothing; it is listed in Settings as "New network" with action Ignore, and the user can pick an action there

B) Ask with a notification that has Connect / Ignore buttons, and remember the choice

C) Apply a default action chosen in Settings (connect, disconnect or ignore) for every unknown network

D) Other (please describe after [Answer]: tag below)

[Answer]: b

## Question 2: Naming a network
How should a network get its name in the list?

A) Wi-Fi name when macOS exposes it, otherwise "Network <last 4 characters of the hardware address>"; always editable

B) Always "Network <last 4 characters>"; editable (never read the Wi-Fi name)

C) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 3: Manual choices
If the user connects or disconnects by hand, what should auto-connect do until the network changes?

A) Stand down until the next network change (like the reconnect loop does)

B) Stand down for a fixed time (for example 30 minutes)

C) Never override a manual choice for that network until the app restarts

D) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 4: Disconnect action and the kill switch
If a trusted network's action is "disconnect" while the kill switch is on, what should happen?

A) Disconnect and release the kill switch (the user said this network is trusted)

B) Do not disconnect while the kill switch is on; show a notice instead

C) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 5: Where it applies
Should auto-connect also work when the app has just started or the Mac just woke?

A) Yes, evaluate the current network at app start, after wake, and on every network change

B) Only on network changes and wake, not at app start (the existing "connect on launch" setting covers that)

C) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 6: Settings UI
Where should the network list live?

A) Settings, new "Networks" section: master switch, default action, list with name, action and Forget

B) A new sidebar page "Networks"

C) Other (please describe after [Answer]: tag below)

[Answer]: a
