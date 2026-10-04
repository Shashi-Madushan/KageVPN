type Bag = Record<string, unknown>;

/** Reads `a.b.c` from nested objects; anything missing along the way is undefined. */
export function getPath(source: unknown, path: string): unknown {
	let value: unknown = source;
	for (const key of path.split('.')) {
		if (value === null || typeof value !== 'object') return undefined;
		value = (value as Bag)[key];
	}
	return value;
}

/** Writes `a.b.c`, creating objects along the way. */
export function setPath(target: Bag, path: string, value: unknown): void {
	const keys = path.split('.');
	let node = target;
	for (const key of keys.slice(0, -1)) {
		if (node[key] === null || typeof node[key] !== 'object') node[key] = {};
		node = node[key] as Bag;
	}
	node[keys[keys.length - 1]] = value;
}
