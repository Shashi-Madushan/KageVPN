export function clickOutside(node: HTMLElement, handler: () => void) {
	const listener = (event: MouseEvent) => {
		if (!node.contains(event.target as Node)) handler();
	};
	document.addEventListener('mousedown', listener, true);
	return {
		destroy() {
			document.removeEventListener('mousedown', listener, true);
		}
	};
}

export function focusOnMount(node: HTMLInputElement) {
	node.focus();
	node.select();
}
