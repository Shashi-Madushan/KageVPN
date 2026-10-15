import { api, type RuleSetsStatus } from './api';

/** What is downloaded for the lists in the saved routing settings, shared by the pages that show it. */
class RuleLists {
	status = $state<RuleSetsStatus | null>(null);

	async reload() {
		this.status = await api.ruleSetsStatus().catch(() => null);
	}

	present(tag: string): boolean | null {
		const item = this.status?.lists.find((l) => l.tag === tag);
		return item ? item.present : null;
	}
}

export const ruleLists = new RuleLists();
