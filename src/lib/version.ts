function parts(version: string): { numbers: number[]; pre: string[] | null } {
	const [core, pre] = version.replace(/^v/, '').split(/-(.+)/);
	return {
		numbers: core.split('.').map((n) => Number.parseInt(n, 10) || 0),
		pre: pre ? pre.split('.') : null
	};
}

export function compareVersions(a: string, b: string): number {
	const [x, y] = [parts(a), parts(b)];
	for (let i = 0; i < 3; i++) {
		const diff = (x.numbers[i] ?? 0) - (y.numbers[i] ?? 0);
		if (diff) return Math.sign(diff);
	}
	if (!x.pre && !y.pre) return 0;
	if (!x.pre) return 1;
	if (!y.pre) return -1;
	for (let i = 0; i < Math.max(x.pre.length, y.pre.length); i++) {
		const [p, q] = [x.pre[i], y.pre[i]];
		if (p === undefined) return -1;
		if (q === undefined) return 1;
		const [pn, qn] = [Number(p), Number(q)];
		const diff = Number.isNaN(pn) || Number.isNaN(qn) ? p.localeCompare(q) : pn - qn;
		if (diff) return Math.sign(diff);
	}
	return 0;
}
