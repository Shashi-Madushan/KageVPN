# Integration (manual, on a real Mac)
1. **IPv6 off**: Routing > Allow IPv6 off, connect in TUN mode; `curl -6 https://api64.ipify.org` must fail, IPv4 sites load.
2. **Networks**: Settings > Networks on; join a new Wi-Fi: the dialog appears once; choose Connect; leave and rejoin: it connects by itself. Set Disconnect on another network with the kill switch on: the VPN turns off and the block lifts. Connect by hand there: nothing overrides it until the network changes.
3. **Rollback**: install a second engine version, connect once (marks it good), switch to the other, press Roll back, connect.
4. **Repair**: disconnect, Diagnostics > Repair VPN service: one password prompt; connecting in TUN mode works afterwards.
5. **Diagnostics bundle**: Export diagnostics; open the zip; search for your server host, UUID, subscription link: none present.
6. **App Nap**: while connected run `pmset -g assertions`; "Kage VPN is connected" is listed.
7. **Popover**: server list shows latency after Test.
