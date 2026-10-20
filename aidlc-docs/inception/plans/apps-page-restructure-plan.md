# Move per-app controls to their own sidebar page: plan

Request: the data caps (and the per-app list around them) do not belong to routing rules, so they should be a new item in the sidebar instead of a tab on the Routing page.

## Is the user right?
Mostly. Caps, usage and "block this app" are about **an app's data and access**, not about routing rules (domains, address ranges, country lists). Two things still tie them to routing, and the plan has to respect both:
- The per-app **VPN / Direct / Block** choice *is* a routing rule underneath (`kind: app`), and a cap block is a routing rule with `origin: cap`. The storage stays as it is; only where it is shown changes.
- Saving a per-app choice reconnects the VPN and today uses the Routing page's draft and **Save changes**. A separate page needs its own way to apply changes (see question 3).

## What the code looks like now
- Sidebar (`Sidebar.svelte`): sections VPN (Home, Servers, Data usage), Protection (Routing, Connections, Diagnostics), System (Engine, Logs, Settings). Nine pages; shortcuts are Cmd+1 to Cmd+9 by `pageOrder`, so a tenth page would have no number and inserting in the middle renumbers the pages after it.
- `+page.svelte` has `titles`, `subtitles` and the page switch; `Icon.svelte` has a fixed icon list.
- `AppsTab.svelte`, `CapEditor.svelte`, `caps.svelte.ts`, `capView.ts`, `appRules.ts` are self-contained. `AppsTab` takes the Routing page's `draft` as a prop, which is the only real coupling.
- The Home banner and the Connections page already work without the Routing page.

## Questions

## Question 1
What should the new page hold?

A) Everything per-app: the app list with the VPN / Direct / Block switch, usage, caps, the Blocked list and the two presets. The Routing page keeps domain rules, ready-made lists and the test tool

B) Only data and access: usage, caps and a plain Block switch. The VPN / Direct choice and the presets stay on a smaller Apps tab in Routing

C) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 2
Where should it sit in the sidebar, and what should it be called?

A) "Apps", in the Protection section right after Routing

B) "Apps", in the VPN section right after Data usage (it is about data)

C) "App controls", in the Protection section right after Routing

D) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 3
Per-app choices change routing and reconnect the VPN. How should the new page apply them?

A) Same as today: a "Save changes" bar appears on the page when something changed, and saving reconnects once (several edits, one reconnect)

B) Apply each change at once and reconnect each time

C) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 4
The page order grows to ten and the shortcuts are Cmd+1 to Cmd+9. What should happen?

A) Keep Cmd+1 to 9 for the first nine in sidebar order and give the new page no shortcut (Settings, the last, loses its number)

B) Keep every existing number as it is and give the new page the next free one, even though its sidebar position differs

C) Add Cmd+0 as the tenth

D) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 5
What should happen to the old Apps tab on the Routing page?

A) Remove it, and leave a short line on the Routing page pointing to the new Apps page ("Per-app choices and data caps are on the Apps page")

B) Remove it silently

C) Other (please describe after [Answer]: tag below)

[Answer]: 

## Proposed plan (assuming A for questions 1, 3 and 5, and A for 2)

### Steps
- [ ] 1. **Page and navigation**: add `'apps'` to `Page` and `pageOrder`, an icon (`apps`, a four-square grid) in `Icon.svelte`, an entry in `Sidebar.svelte`, titles and subtitles in `+page.svelte` ("Apps", "Choose how each app uses the VPN, and set data limits"), and the page switch. Home banners and other `onnavigate` links keep working (the Block now card can link to the page).
- [ ] 2. **AppsPage**: new `AppsPage.svelte` holding what `AppsTab.svelte` holds today. It owns a draft of the routing settings (cloned from `app.routing`, resynced with the same unedited-vs-edited rule the Routing page uses) and a floating "Save changes" bar like the Routing page's. Extract the draft sync into a small shared helper so both pages use one implementation (`src/lib/routingDraft.svelte.ts`).
- [ ] 3. **Routing page**: remove the Apps tab and `AppsTab` import; keep the rule editor's app picker (rules by app still work there); add the pointer line from question 5.
- [ ] 4. **Rename and tidy**: `AppsTab.svelte` becomes `AppsPage.svelte`; its panel styles become page styles; the "Other app rules" list links to the Routing page's rule editor (`onnavigate('routing')`).
- [ ] 5. **Shortcuts**: apply the choice for question 4 in `pageOrder` and the key handler; update the shortcut text in Settings and the user guide if they list numbers.
- [ ] 6. **Docs**: the user guide section "Choosing by App" and "Data Caps for Apps" move to a new "Apps page" section; the changelog entry for 20 Oct is reworded (it is not released yet); AI-DLC state and audit updated.
- [ ] 7. **Tests**: a script test for the page order and shortcut mapping (extract the mapping to a pure function in `Sidebar.svelte`'s module script or a small `.ts`), and the existing 26 script tests and 344 Rust tests stay green. No backend change is needed.
- [ ] 8. **Verify**: `svelte-check`, `vite build`, `npm run test:scripts`, `cargo test --lib -- --test-threads=4`, `cargo clippy`.

### Scope and effort
Front-end only, about 4 files changed and 2 added, no new commands, no data migration (rules and caps are stored exactly as now). Small to medium.

### Risks
- The draft logic exists in two pages unless it is shared (step 2 shares it).
- If both pages are open in turn with unsaved edits in one, the other page resyncs from stored settings; unsaved edits on the page being left are kept only while the app runs (same as today when switching tabs).
- Existing screenshots in `docs/` and `docs-site` that show the Routing tabs may need replacing; I cannot capture screenshots here.
