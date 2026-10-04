import type { ProtocolInfo } from './api';

/** The protocols the backend can run, loaded once at startup. The backend owns every detail;
 *  this only answers "which links do we accept" and "what is it called". */
class Protocols {
	list = $state<ProtocolInfo[]>([]);

	set(list: ProtocolInfo[]) {
		this.list = list;
	}

	/** Matches a URL scheme of any supported protocol at the start of a link. */
	private get schemes() {
		return this.list.flatMap((p) => p.schemes);
	}

	isLink = (token: string) => {
		const schemes = this.schemes;
		return schemes.length > 0 && new RegExp(`^(${schemes.join('|')})://`, 'i').test(token);
	};

	/** True when text contains a supported link anywhere. */
	hasLink = (text: string) => {
		const schemes = this.schemes;
		return schemes.length > 0 && new RegExp(`(^|\\s)(${schemes.join('|')})://`, 'i').test(text);
	};

	label = (id: string) => this.list.find((p) => p.id === id)?.label ?? id;

	find = (id: string) => this.list.find((p) => p.id === id);
}

export const protocols = new Protocols();
