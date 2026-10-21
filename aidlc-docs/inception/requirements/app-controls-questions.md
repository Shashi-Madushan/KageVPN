# Per-app controls: clarifying questions

Request: a per-app picker for bypass and proxy rules, plus data-usage caps and "disable usage" for chosen apps.

What the app already has (from the code): a rule editor on the Routing page with an app picker (`AppPicker.svelte`, installed apps from `apps.rs`), "Use the VPN / Go direct / Block" per app rule, a Route buttons row on the Connections page, per-app usage totals in the usage database, and one global data cap with 80% and 100% alerts (`usage/quota.rs`). Please answer with the letter after each [Answer]: tag.

## Question 1
What should the per-app picker be, given the rule editor already has an app dropdown?

A) A new "Apps" tab on the Routing page: a searchable list of every installed app, each with a three-way switch (VPN / Direct / Block), instead of adding rules one by one

B) Keep the rule editor as it is and only add icons and a clearer search to the existing dropdown

C) A new "Apps" tab as in A, and also show the app's icon and current usage on each row

D) Other (please describe after [Answer]: tag below)

[Answer]: C (not answered by the user; chosen by the assistant: the Apps tab with icons and usage, and presets plus per-app overrides give the most complete result using what already exists)

## Question 2
In "VPN for everything except these apps" setups, how should the default and the per-app choice relate?

A) Per-app choice is an override of the current Routing mode (Everything else stays as set); apps with no choice follow the default

B) Offer two simple presets at the top: "Everything through VPN, except the apps I pick" and "Only the apps I pick through VPN"

C) Both: presets at the top that set the default, and per-app overrides below

D) Other (please describe after [Answer]: tag below)

[Answer]: C (not answered by the user; chosen by the assistant)

## Question 3
What does a per-app data cap measure?

A) Bytes the app moved through the VPN only (what the app already counts in the usage database)

B) Bytes through the VPN and direct, so the cap covers all of the app's traffic

C) Through the VPN only for now, with direct counting left for later

D) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 4
Over what period does each app's cap reset?

A) The same billing cycle day as the existing global data cap

B) Each app picks its own: daily, weekly or monthly

C) Monthly only, on the first day of the month

D) Other (please describe after [Answer]: tag below)

[Answer]: b

## Question 5
When an app reaches its cap, what should happen?

A) Notify only (at 80% and 100%), like the global cap

B) Choose per app: notify only, or block the app until the cycle resets

C) Choose per app: notify only, block the app, or send the app direct (outside the VPN) so it stops using VPN data

D) Other (please describe after [Answer]: tag below)

[Answer]: b

## Question 6
Blocking or rerouting an app on reaching its cap needs a routing rule, and a routing change makes the VPN reconnect today. How should that be handled?

A) Add a temporary rule and reconnect automatically, with a notification saying why (simple, a brief interruption)

B) Ask first: the notification has a "Block now" action; nothing changes without a click

C) Apply the rule without a full reconnect, if sing-box can reload rules live (needs a spike to check; fall back to A if not)

D) Other (please describe after [Answer]: tag below)

[Answer]: b

## Question 7
What should "disable usage for selected apps" mean?

A) Block the app's network access completely (the Block action), switchable on and off from the Apps tab

B) Block, plus a schedule (for example "no network from 22:00 to 07:00" or "weekdays only")

C) Block it only while on certain networks (reusing the Networks feature)

D) Other (please describe after [Answer]: tag below)

[Answer]: a like how the data eye app work 

## Question 8
When a cap is reached or an app is blocked this way, how should the app be undone?

A) Automatically when the cycle resets, and manually any time with an "Unblock" button on the app's row

B) Only manually

C) Automatically at reset only; no manual override while the cap is hit

D) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 9
Where should caps and usage be visible?

A) On each app's row in the Apps tab (a small progress bar), plus the notifications

B) Also in the Usage page (per-app cap progress) and the menu-bar popover (apps near their cap)

C) Only in the Apps tab; no popover or Usage page changes

D) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 10
How accurate must per-app counting be?

A) Good enough for a soft cap: the app is counted from the connections list every 2 seconds, so it may overshoot by a few seconds of traffic

B) Strict: stop the traffic the moment the cap is reached (would need a different enforcement mechanism and is much larger)

C) Soft cap now, mention the limitation in the UI and docs

D) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 11
Existing rules by process name (for apps with no `.app` bundle, such as `curl` or command-line tools) should:

A) Keep working in the rule editor, and show in the Apps tab as extra rows labelled with the process name

B) Stay in the rule editor only; the Apps tab lists installed `.app` bundles only

C) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 12
Which features from the earlier list should be in this round?

A) Only this per-app work (picker, caps, disable)

B) Also low-data and expiry alerts in the menu bar

C) Also low-data and expiry alerts, and the universal (Intel and Apple Silicon) build

D) Other (please describe after [Answer]: tag below)

[Answer]: a and b 

## Question 13
Extension rules (security baseline, property-based testing, resiliency baseline) were all left off for this project earlier. For this work:

A) Keep them off, as before

B) Turn on the security baseline for this work (the per-app block and cap logic change what traffic is allowed)

C) Other (please describe after [Answer]: tag below)

[Answer]: a
