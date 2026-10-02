import type { Snippet } from 'svelte';

/** Page-level actions rendered in the shell header; pages register them via <HeaderActions>. */
export const header = $state<{ actions: Snippet | null }>({ actions: null });
