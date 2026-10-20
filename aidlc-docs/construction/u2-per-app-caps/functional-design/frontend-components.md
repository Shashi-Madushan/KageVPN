# U2 frontend components

## CapEditor (inside an Apps row, expands under it)
- **State:** limit number, unit (MB or GB), period (Day, Week, Month), at-limit (Notify only, Offer to block), Save, Remove cap.
- **Shows** when a cap exists: a progress bar with "1.4 GB of 2 GB today (approximate)", resets "at midnight" or the date, the alert level colour (normal, 80 percent amber, over red).
- **States:** `ok`, `warn80`, `over` with **Block now** and **Not now**, `blocked` ("Blocked by a data cap" with **Unblock**), `dismissed`, `notCounting` ("App usage is off" with **Turn on**), "not installed" with **Remove**.
- **Validation:** limit between 1 MB and 10 TB; Save disabled otherwise; backend errors shown inline.
- **API:** `caps_list`, `cap_set`, `cap_remove`, `cap_block`, `cap_not_now`, `cap_unblock`, plus the existing usage `settings`/`setSettings` for the recording switch.

## Apps row changes (AppsTab)
- A small "Cap" button on each row (a bar replaces it when a cap exists). A cap block shows "blocked by a data cap" and the switch stays disabled, as in U1.
- The Blocked strip gets an Unblock button for cap blocks (calls `cap_unblock`).

## CapBanner (Home page)
- One banner per app with `offer = pending`: "Slack reached its 2 GB daily cap. Block it?" with **Block now** and **Not now**. Hidden when the list is empty. Also shows a one-line note when a cap block was just lifted.

## Menu-bar popover
Not changed in U2 (alerts for the popover belong to U4).

## State management
A `caps` store (`src/lib/caps.svelte.ts`) holds the `CapStatus` list, refreshed every 5 seconds while Home or the Apps tab is open and on `caps-changed`. Components only read it and call its actions.
