/**
 * Handles a checkbox that saves as soon as it is clicked. When the save fails the stored value
 * stays as it was, and Svelte only rewrites `checked` when that value changes, so the box would
 * keep showing the click. It is put back here instead.
 */
export async function saveToggle(event: Event, save: (on: boolean) => Promise<boolean>): Promise<void> {
	const box = event.currentTarget as HTMLInputElement;
	const on = box.checked;
	if (!(await save(on))) box.checked = !on;
}
