# Integration (manual, on a real Mac)
1. **IPv6 off**: Routing > Allow IPv6 off, connect in TUN mode; `curl -6 https://api64.ipify.org` must fail, IPv4 sites load.
2. **Networks**: Settings > Networks on; join a new Wi-Fi: the dialog appears once; choose Connect; leave and rejoin: it connects by itself. Set Disconnect on another network with the kill switch on: the VPN turns off and the block lifts. Connect by hand there: nothing overrides it until the network changes.
3. **Rollback**: install a second engine version, connect once (marks it good), switch to the other, press Roll back, connect.
4. **Repair**: disconnect, Diagnostics > Repair VPN service: one password prompt; connecting in TUN mode works afterwards.
5. **Diagnostics bundle**: Export diagnostics; open the zip; search for your server host, UUID, subscription link: none present.
6. **App Nap**: while connected run `pmset -g assertions`; "Kage VPN is connected" is listed.
7. **Popover**: server list shows latency after Test.

# Round 2 integration (manual, on a real Mac; none of these has been run)
1. **Apps tab**: Routing > Apps: icons appear as you scroll; set Slack to Block, Save changes, reconnect: Slack has no network and the Connections page shows it blocked. Reset arrow returns it to the default.
2. **Self-block refusal**: set Kage VPN to Block and Save: refused with a clear message.
3. **Presets**: "VPN only for..." then pick one app: only that app uses the VPN (check its exit IP).
4. **Cap set**: Cap on an app: 1 MB, Day, Offer to block; use the app through the VPN. 80% and 100% notifications once each; a Block now card on Home.
5. **Block now**: the VPN reconnects once; the app has no network; the row says blocked by a data cap; Unblock lifts it and the offer does not return the same day.
6. **Rollover**: set the Mac's clock past midnight (or wait): the block lifts, the VPN reconnects, a "can use the VPN again" notification arrives. With the VPN off at midnight, the block is gone at the next connect.
7. **Stale page**: open Routing, edit a rule without saving, press Block now from Home: the page keeps your edit and shows the cap block; Save keeps the block.
8. **Usage off**: turn app usage off: the cap editor says nothing is counted; Turn on restores counting. Clear details: the cap's usage is unchanged. Save any Usage setting: the cap remains.
9. **Expiry**: a test subscription with an end date 6 days away gives one "expires in 6 days" notice within half an hour, also when the provider is unreachable; the popover plan line is in the warning colour.

# Round 4 integration, pull request 1 (protocols)
## Run by the assistant (spare ports, never 2080 or 19090)
A. **All protocols in one real sing-box:** the config the app builds from every protocol's sample (written by `generated_configs_pass_singbox_check` to `$TMPDIR/kage-vpn-test-Proxy.json`) was moved to ports 21080 and 29090 and run with sing-box 1.14.2. The Clash API listed all 10 nodes (9 protocols, WireGuard as an endpoint) in the `proxy` selector. Selecting the TUIC, Hysteria, SSH and WireGuard nodes through `PUT /proxies/proxy` returned 204 and became `now`. Delay tests answered with an error (TUIC, Hysteria, SSH: the sample hosts do not resolve) or `Timeout` (WireGuard: a documentation address), which the app shows as a failed test. sing-box was stopped and both ports were free afterwards.
B. **WireGuard endpoint alone:** same result (listed, delay answers).

## To run on a real Mac with real servers (not run)
1. **TUIC v5:** import a provider's `tuic://` link; connect in Proxy Only mode; open a site; the list shows "TUIC · BBR".
2. **Hysteria v1:** import a link without speeds: the toast names the defaults; set the real speeds in the editor; connect.
3. **SSH:** add a server with a pasted private key and its host key (from `ssh-keyscan`); connect; then change one character of the host key and connect again: it must fail.
4. **WireGuard:** drop a WARP or provider `.conf`; connect in Proxy Only mode, then TUN mode; Test in the server list gives a latency.
5. **Engine without a feature:** point the Engine page at a sing-box built without `with_wireguard`, choose a WireGuard server, connect: the Engine page message appears and nothing starts.
6. **Subscription refresh:** a provider subscription with WireGuard entries keeps the same servers (and favourites) after a refresh that changes the local address.
