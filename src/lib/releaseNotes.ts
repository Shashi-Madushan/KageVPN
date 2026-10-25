// What's New content, newest first. Keep entries short and user-facing; the full history is in the docs changelog.
// To announce a release: add an entry on top whose `version` matches package.json, so the sidebar badge appears.

export type NoteKind = 'new' | 'improved' | 'fixed';

export type Note = { kind: NoteKind; title: string; text: string; page?: 'apps' | 'settings' | 'profiles' | 'routing' | 'connections' | 'home' };

export type Release = { version: string; date: string; headline: string; notes: Note[] };

export const releaseNotes: Release[] = [
	{
		version: '0.3.0',
		date: '2026-10-25',
		headline: 'More protocols, per-app controls and a fresh Home',
		notes: [
			{ kind: 'new', title: 'A redesigned Home and menu bar popover', text: 'A larger mascot, a Connect/Disconnect button, a server and mode card, a wide activity chart and health chips.', page: 'home' },
			{ kind: 'new', title: 'App controls and data caps', text: 'Send each app through the VPN, direct, or block it, and give any app a daily, weekly or monthly data limit.', page: 'apps' },
			{ kind: 'new', title: 'WireGuard, TUIC, Hysteria and SSH servers', text: 'Import links, WireGuard .conf files and subscription entries for four more protocols.', page: 'profiles' },
			{ kind: 'new', title: 'Set system proxy', text: 'In Proxy Only mode the macOS proxy settings follow the VPN and are restored when you disconnect.', page: 'settings' },
			{ kind: 'improved', title: 'Earlier subscription expiry warnings', text: 'You are told 7 days, 3 days and 1 day before a plan ends, and when it ends.' },
			{ kind: 'improved', title: 'IPv6 is off by default', text: 'New installs carry IPv4 only, so sites no longer half-load on servers without IPv6.', page: 'routing' },
			{ kind: 'fixed', title: 'App icons now appear', text: 'Icons for every installed app show up, and retry or fall back to the initial if one cannot be read.', page: 'apps' }
		]
	},
	{
		version: '0.2.0',
		date: '2026-10-20',
		headline: 'A clearer Connections page and safer updates',
		notes: [
			{ kind: 'improved', title: 'Connections redesigned', text: 'Wider columns, per-app connection counts, a busy bar and sortable headings.', page: 'connections' },
			{ kind: 'improved', title: 'Updates go through the VPN', text: 'A network that blocks GitHub no longer stops update checks and downloads.' },
			{ kind: 'fixed', title: 'Tidy speed readings', text: 'Speeds under 1 KB are rounded instead of showing long raw numbers.' }
		]
	}
];
