# U5 System proxy: business rules

## R1. When it applies
The system proxy is set only when all hold: the toggle is on, the mode is Proxy Only, the local proxy login is **off** (FR2.4), and a run is up. It does nothing in TUN mode.

## R2. When it is set (Question 1)
As soon as sing-box is listening, when the status turns Connected (before the health check result). A failed health check does not undo it; the watchdog deals with a bad tunnel.

## R3. Order of operations when setting
1. Read every **enabled** service's current settings (FR2.3). Disabled services (marked `*` by `networksetup`) are skipped.
2. Write the snapshot (atomic, private) and the restore script. If either write fails, change nothing and report `Failed { set }`.
3. For each service: set Web, Secure Web and SOCKS to `127.0.0.1:<port>` (no authentication), turn the PAC URL and auto-discovery **off** (they would take precedence over the manual proxies), and write the bypass list.
4. If a service fails half-way, restore the services already changed and report `Failed { set }`.

## R4. What a restore puts back (Question 4)
For each saved service, only while it still points at Kage, meaning its Web, Secure Web or SOCKS proxy is `127.0.0.1:<saved port>`:
- the three proxies, the PAC URL and its on/off, auto-discovery and the bypass list go back exactly as saved;
- a service you changed since (no longer pointing at Kage) is left as you set it;
- a service that no longer exists is skipped.
After a successful restore the snapshot and script are deleted. A restore that fails keeps both and reports `Failed { restore }`.

## R5. When it is restored
- The user disconnects or quits.
- The toggle is turned off, the mode leaves Proxy Only, or the proxy login is turned on, while connected (Question 7, at once).
- Reconnecting gives up (no more automatic attempts).
- **Not** on a restart (Question 2): watchdog, auto-reconnect, network change, routing change keep it set. If the restarted run uses another port, the services are set to the new port and the snapshot and script are updated.
- By the supervisor (Question 3): when it ends **without** a stop request (the app died, or sing-box died on its own), it runs the restore script, then deletes it and the snapshot. If the app is alive (sing-box died on its own) the app's reconnect sets the proxy again.
- At launch: if a snapshot is still there (power loss, killed supervisor), it is restored before anything else, with the same R4 guard.

## R6. Services that appear while connected (Question 5)
When the network watcher reports a change while the system proxy is set, enabled services not yet in the snapshot are read, added to the snapshot and script, then set. Services that disappeared are dropped from the next set but kept in the snapshot (restoring skips them if they no longer exist).

## R7. Bypass list (FR2.5)
- Built-in list first, then the user's entries, trimmed, without repeats, empty lines ignored.
- An entry is accepted if it is a host name (letters, digits, `-`, `.`, an optional leading `*.`), an IPv4 or IPv6 address, or such an address with a `/prefix`. Anything else is refused on save with "'<entry>' is not a host name or address range".
- At most 200 entries.

## R8. Proxy login (FR2.4)
- While the login is on, the toggle cannot be turned on; Settings shows why.
- Turning the login on while the system proxy is set restores at once (R5) and leaves the toggle on but inactive, with the same note.

## R9. Privileges (NFR3)
All reads, sets and restores use `/usr/sbin/networksetup` as the user, which macOS allows without a password (it is entitled for the network settings right). No root service and no administrator prompt. If a command reports that it needs authorisation anyway, the action fails with "macOS refused to change the proxy settings" and `Failed`.

## R10. Failures and Repair (FR2.9)
- `Failed { set }`: the connection stays up (apps can still be pointed at the proxy by hand); Home shows "Could not set the system proxy" with Repair.
- `Failed { restore }`: Home shows "Your proxy settings may still point at Kage" with Repair, also when disconnected.
- **Repair** runs a restore from the snapshot (R4). With no snapshot there is nothing Kage changed, and Repair says so; it never clears settings it did not save.

## R11. Testing on the live Mac (NFR2, Question 8)
Runtime checks change only one unused service ("iPhone USB"), on spare ports, never 2080 or 19090. Its settings are saved before and compared after. The first change is made only after asking.
