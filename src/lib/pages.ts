// The pages of the main window, in sidebar order, and the keyboard shortcuts that open them.

export type Page = 'home' | 'profiles' | 'usage' | 'routing' | 'apps' | 'connections' | 'diagnostics' | 'core' | 'logs' | 'settings';

export const pageOrder: Page[] = ['home', 'profiles', 'usage', 'routing', 'diagnostics', 'apps', 'connections', 'core', 'logs', 'settings'];

/** Cmd+1 to Cmd+9 open the first nine pages in sidebar order; later pages have no number. */
export function shortcutFor(page: Page): number | null {
	const index = pageOrder.indexOf(page);
	return index >= 0 && index < 9 ? index + 1 : null;
}

/** The page a pressed digit key opens, or null. */
export function pageForKey(key: string): Page | null {
	const index = Number(key) - 1;
	return Number.isInteger(index) && index >= 0 && index < 9 ? (pageOrder[index] ?? null) : null;
}
