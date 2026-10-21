import { test } from 'node:test';
import assert from 'node:assert/strict';
import { dnsNote } from '../../src/lib/dnsNote.ts';

test('in Proxy Only mode the DNS choice says it is not used', () => {
	for (const provider of ['cloudflare', 'custom', 'system']) {
		for (const route of ['global', 'rule', 'direct']) assert.match(dnsNote(provider, route, 'proxy'), /Not used in Proxy Only mode/);
	}
});

test('in Full System mode it says where lookups go', () => {
	assert.match(dnsNote('system', 'global', 'tun'), /internet provider can see/);
	assert.match(dnsNote('quad9', 'direct', 'tun'), /normal connection/);
	assert.match(dnsNote('cloudflare', 'rule', 'tun'), /through the VPN/);
});
