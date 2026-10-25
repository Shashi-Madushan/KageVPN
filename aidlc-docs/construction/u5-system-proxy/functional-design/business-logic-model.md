# U5 System proxy: business logic model

## Component
`sysproxy` (new backend module) is split so the decisions are pure and testable, and only a thin layer runs commands:
- **Pure:** parse `networksetup` output into `ServiceProxy`; decide which services are "still Kage's" (R4); build the bypass list (R7); build the commands that set a service and the commands that restore one; build the restore script text.
- **Effects:** run `networksetup` (a small runner that can be swapped in tests); read and write the snapshot and script files; update `SystemProxyState` and emit it.

## F1. Connect (Proxy Only, toggle on, login off)
```
sing-box listening -> status Connected
  -> sysproxy::apply(port)
       services = enabled services
       for new ones: read current settings
       snapshot += their settings; write snapshot; write restore script (port)
       for each service: set web/secure/socks to 127.0.0.1:port, PAC off, WPAD off, bypass list
       on a failure: restore the ones changed; state Failed{set}
       else: state Set{port, services}
```

## F2. Restart (watchdog, auto-reconnect, network or routing change)
```
stop for restart: leave the system proxy as it is
next run listening:
  same port      -> nothing to do (F6 handles new services)
  different port -> set each service to the new port; rewrite snapshot port and script
reconnecting gives up -> F3
```

## F3. Disconnect, quit, toggle off, leaving Proxy Only, login on
```
sysproxy::restore()
  snapshot = read; if none -> state Off; done
  for each saved service still pointing at 127.0.0.1:<snapshot port> and still present:
      put back its saved proxies, PAC, WPAD and bypass list
  all done   -> delete snapshot and script; state Off
  any failed -> keep both; state Failed{restore}
```
Quit runs F3 before the app exits, then the usual stop. The supervisor sees the stop request and does not run the script.

## F4. The app or sing-box dies (supervisor)
```
supervisor loop ends:
  stop file present -> normal stop, script not run
  otherwise (app gone, or sing-box exited) -> if the restore script exists: run it, delete it and the snapshot
```
The script contains, per saved service, a guard (`networksetup -getwebproxy` shows 127.0.0.1 and the saved port) and the put-back commands. Supervisor change: about five lines; no new logic in shell.

## F5. Launch
```
before auto-connect: if a snapshot exists -> F3 (restore), log what was restored
```

## F6. Network change while set
```
network watcher change event and state Set:
  enabled services not in snapshot -> read, add to snapshot, rewrite script, set them
```

## F7. Turning on while connected
```
toggle on (or mode becomes Proxy Only, or login turned off) while connected in Proxy Only -> F1 with the current port
```
Mode changes already reconnect; F1 runs after the new run is listening.

## F8. Repair
```
snapshot exists -> F3
no snapshot    -> "Nothing to restore: Kage has not changed your proxy settings"
```

## Data flow summary
Settings (`ReliabilitySettings.systemProxy`, `bypass`) -> session hooks (connect, stop, restart, mode, toggle) -> `sysproxy` -> `networksetup`; snapshot and script on disk -> supervisor and launch; `SystemProxyState` -> `Status.systemProxy` -> Home.
