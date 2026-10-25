# U2 business logic model

## Counting and alerts (every 2 seconds while connected)
```
poll tick
  settings = usage settings; if paused or recording off -> stop
  for each connection with an app bundle that has a cap:
      tally = load or create tally for the app
      if tally.periodStart != periodStart(now, cap.period): rollover(app)
      tally += connection byte change
  for each capped app whose tally changed:
      level = level(tally, cap)
      if level > tally.alertLevel: send notification; tally.alertLevel = level
      if level == 100 and cap.atLimit == offerBlock and tally.offer == none: tally.offer = pending
  also: for every cap, if its period has rolled (even with no traffic): rollover(app)
  save changed tallies; tell the UI if anything changed
```

## Rollover(app)
```
reset tally (zero, periodStart = current, alertLevel 0, offer none)
if a cap rule exists for the app:
    remove it
    if connected: reconnect once, notify "<app> can use the VPN again"
```
At app start and before every connect, rollover runs for all caps first (no reconnect needed when connecting).

## Block now(app)
```
require offer == pending and app is not Kage or the engine
add cap rule on top; set routing mode to Rules if needed
offer = taken
reconnect if connected
```

## Not now(app): offer = dismissed.   Unblock(app): remove cap rule, offer = dismissed, reconnect if connected.

## Set or edit a cap
```
validate limit and path
period changed -> restart tally; limit changed -> keep tally
atLimit == notify and a cap rule exists -> lift it
re-evaluate the level (never lowers alertLevel)
```

## Remove a cap: lift its rule if any, delete its tally and cap.

## Data flow
UI (Apps tab, Home banner) -> cap commands -> cap module (settings, tally table, routing store) -> existing reconnect path. The poll only writes tallies and raises alerts. The UI polls `caps_list` every 5 seconds while the page is open and reacts to a `caps-changed` event.
