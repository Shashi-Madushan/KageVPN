import type { Profile, Protocol } from './api';

/** What the interface needs to know about each protocol. The backend registry owns parsing and
 *  validation; to add a protocol, list it here and give ProfileDialog its fields. */
export interface ProtocolInfo {
	id: Protocol;
	label: string;
	schemes: string[];
	/** Password-style credential (everything except VLESS, which uses a UUID). */
	usesPassword: boolean;
	/** Whether the TLS/transport sections apply. */
	usesTls: boolean;
	usesTransport: boolean;
}

export const PROTOCOLS: ProtocolInfo[] = [
	{ id: 'vless', label: 'VLESS', schemes: ['vless'], usesPassword: false, usesTls: true, usesTransport: true },
	{ id: 'trojan', label: 'Trojan', schemes: ['trojan'], usesPassword: true, usesTls: true, usesTransport: true },
	{ id: 'shadowsocks', label: 'Shadowsocks', schemes: ['ss'], usesPassword: true, usesTls: false, usesTransport: false },
	{ id: 'hysteria2', label: 'Hysteria2', schemes: ['hysteria2', 'hy2'], usesPassword: true, usesTls: true, usesTransport: false }
];

export const protocolInfo = (id: Protocol): ProtocolInfo => PROTOCOLS.find((p) => p.id === id) ?? PROTOCOLS[0];

const schemes = PROTOCOLS.flatMap((p) => p.schemes);
const LINK = new RegExp(`^(${schemes.join('|')})://`, 'i');
export const ANY_LINK = new RegExp(`(^|\\s)(${schemes.join('|')})://`, 'i');

/** True when a single token looks like a server link of any supported protocol. */
export const isServerLink = (token: string) => LINK.test(token);

export const linkExamples = schemes.map((s) => `${s}://`).join(', ');

/** Short description for lists, such as "Reality · grpc" or "Hysteria2". */
export function describe(profile: Profile): string {
	const info = protocolInfo(profile.protocol);
	if (profile.protocol === 'vless') return `${profile.security === 'reality' ? 'Reality' : 'TLS'} · ${profile.transport.kind}`;
	if (profile.protocol === 'trojan') return `Trojan · ${profile.transport.kind}`;
	return info.label;
}
