# Kill switch redesign (audit item O2)

Status: steps 1 and 2 (G2, G4, G5) are implemented and tested against a stub `pfctl` (see `runner/helper.sh`, `session::holds_block`, `tests/runner.rs`); the rest is design only. Nothing has been run against the real firewall. The kill switch edits `pf` rules as root, so every step below needs a test machine that is not running a live tunnel.

## What happens today

`helper.sh` (root service and per-launch helper) and `supervisor.sh` (proxy-style fallback) share the same logic:

1. At connect, the app resolves each server name once (`session::resolve_servers`) and passes the IPs on line 1 of `helper.start`.
2. While sing-box is alive, no firewall rules exist.
3. If the sing-box process dies on its own, the helper loads an anchor (`com.apple/250.KageVPN`) that passes loopback, private ranges and the server IPs, then blocks everything else.
4. The block stays until a `stop` or `helper.start` file appears, or the owner app exits.

## Gaps and how to close them

| Gap | Cause | Fix |
| --- | --- | --- |
| G1. No protection while sing-box is alive but the tunnel is dead | Block engages only on process exit | The app already detects a dead tunnel (`health.rs`, watchdog). When it restarts the tunnel, it writes a `block` file; the helper engages the block and keeps it until the new run is verified healthy. |
| G2. A failed reconnect releases the block | `session.rs:102,110` write the `stop` file on a failed connect, and the helper treats `stop` as "release" | Split the signals. `stop` stays "release". Add `down` = "stop sing-box, keep the block". Failed reconnects and watchdog restarts use `down`. Only a user disconnect, a quit or "Restore internet access" write `stop`. |
| G3. Block exists only after a death, so the pre-connect DNS check works but a reconnect behind a block cannot resolve names | The block drops DNS to public resolvers | Keep the allow-list persistent and give it server IPs **plus** a DNS pass for the resolvers the app uses for the pre-connect check; skip that check while the block is up and connect straight to the last known IPs. |
| G4. Server IPs are resolved once | `resolve_servers` runs only at connect | Re-resolve on every connect and on every reconnect attempt. Keep the union of the last N results per server (N = 3) in the allow-list so a CDN rotating addresses is still reachable. Send the list through a new `allow` file the helper re-reads, so no re-prompt is needed. |
| G5. IPv6 neighbour discovery is blocked | The ruleset has no `inet6` ICMPv6 pass | Add `pass quick inet6 proto icmp6 all icmp6-type { routersol, routeradv, neighbrsol, neighbradv }` and `pass quick inet6 from fe80::/10 to fe80::/10`, plus the server's IPv6 addresses. |
| G6. Never tested | | See the test plan. |

## Resulting state machine (in the helper)

```
OFF ──connect──► RUNNING (no rules)
RUNNING ──sing-box exits / "block" file──► BLOCKED (rules loaded)
BLOCKED ──"start" file (new run)──► STARTING (rules stay)
STARTING ──app writes "ready" after health check passes──► RUNNING (rules released)
STARTING ──"down" file──► BLOCKED
any state ──"stop" / app quits / quit file──► OFF (rules released)
```

The key change: rules are released only on `ready` or an explicit `stop`, never because a run ended.

## Changes by file

- `runner/helper.sh`, `runner/supervisor.sh`: new files `block`, `down`, `ready`, `allow`; the state machine above; extended ruleset (G5). Keep the two scripts in step; a unit test already compares the installed script with the app's copy.
- `runner/mod.rs`: `request_down`, `request_block`, `signal_ready`; the install check (F14) re-installs the service so the new script lands.
- `session.rs`: failed connect and the reconnect loop use `down` while the kill switch is on; `ready` after the first healthy check; `resolve_servers` runs per attempt and writes `allow`.
- `health.rs`: when the watchdog restarts a tunnel, request the block first (G1).
- `reliability.rs` and `SettingsView.svelte`: the "Restore internet access" button stays. Add a visible "Blocked, reconnecting" state on Home so a user whose traffic is blocked knows why. Copy change: the block now persists across failed reconnects.

## Risks

- A bug leaves the user without internet. Mitigations: the quit/crash path always releases (owner-alive check stays), the manual restore button stays, and `ready`/`stop` have a 10-minute watchdog in the helper that releases if the app is gone silent.
- Behaviour change for existing users: failed reconnects stay blocked. Say so in the setting text and the changelog.

## Test plan (on a spare Mac or VM, not the machine running the live tunnel)

1. Unit tests for the new file protocol in `runner` (pure functions where possible) and for the ruleset text.
2. Shell tests: run the script with a stub `pfctl` on `PATH` that records its input; drive each transition by creating files; assert rules loaded and released at the right moments.
3. Manual, on a test machine: kill sing-box (`kill -9`), cut the server (firewall it), restart watchdog, fail a reconnect, quit the app, hard-kill the app. After each, check `pfctl -a com.apple/250.KageVPN -sr` and that `curl` fails or works as expected.
4. IPv6 network: confirm no neighbour-discovery loss while blocked.

## Step 1 as built

- New `down` file next to `stop`. The helper ends the run on either; `down` additionally puts up (or keeps) the block when the run had kill-switch IPs. `stop` always wins and releases.
- The app writes `down` for a restart (watchdog or lost tunnel with auto-reconnect) and for a failed attempt inside a reconnect loop, and `stop` for everything else. A failed first connect still releases, so a bad config never blocks the internet.
- When a reconnect loop ends with nothing left to try (non-retryable error, or the user took over and is disconnected), the app writes `stop` so the block does not outlive the loop. When the loop gives up because the network is down, the block stays and the notification says so.
- Not yet done: the block is still released the moment a new run is *requested*, so there is a short exposed window while sing-box starts (that is the `ready` signal in step 3's state machine). `supervisor.sh` (Proxy Only) never receives kill-switch IPs, so it was left alone.

## Step 2 as built

- `allowlist.rs` looks every server up on each connect and every 5 minutes while connected (kill switch on, TUN), remembers the last 3 answers per server and writes the union (capped at 64) to `<data dir>/allow`. A failed or blocked lookup changes nothing and is cut off after 3 seconds, so a reconnect behind a block is not held up by DNS.
- The helper reads that file when it puts the block up, together with the addresses named for the run. Only entries shaped like an IPv4 or IPv6 address are used: both files are user-writable, and one bad word would make `pfctl` reject the whole rule set and leave traffic unblocked.
- The rule set now passes outgoing router/neighbour solicitation and advertisement and link-local to link-local traffic (G5).
- Not verified: the `pf` syntax of the ICMPv6 rules has only been exercised against a stub. Check it once on a test machine with `pfctl -n -f` on the generated rules (parse only, loads nothing) before relying on it.

## Order of work

1. G2 (separate `down` from `stop`): the biggest real exposure, small change.
2. G4 and G5: allow-list refresh and ICMPv6.
3. G1: block while the tunnel is dead but sing-box lives.
4. G3 and the UI state.
