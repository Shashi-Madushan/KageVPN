import type { DnsProvider, Mode, RouteMode } from './api';

/**
 * The line under the DNS provider choice: where lookups really go for this connection mode and
 * routing mode. In Proxy Only mode apps hand site names to the local proxy, which passes them to
 * the server, so the choice is not used there.
 */
export function dnsNote(provider: DnsProvider, route: RouteMode, connection: Mode): string {
	if (connection === 'proxy') return 'Not used in Proxy Only mode: apps send site names to the server, which looks them up. This applies in Full System mode.';
	if (provider === 'system') return 'Your internet provider can see which sites you look up.';
	if (route === 'direct') return 'While the VPN is idle, lookups go to this provider over your normal connection.';
	return 'Lookups go through the VPN, so they stay private.';
}
