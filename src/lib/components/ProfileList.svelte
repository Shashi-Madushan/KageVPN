<script lang="ts">
	import { app } from '$lib/app.svelte';
	import { latencyLabel, latencyTone } from '$lib/format';

	let { onadd }: { onadd: () => void } = $props();
	let editingId = $state<string | null>(null);

	function commit(id: string, event: Event) {
		app.rename(id, (event.currentTarget as HTMLInputElement).value);
		editingId = null;
	}

	function focusOnMount(node: HTMLInputElement) {
		node.focus();
		node.select();
	}
</script>

<section>
	<header>
		<span class="caption">Profiles</span>
		<button class="add" onclick={onadd}>+ Add</button>
	</header>

	{#if app.profiles.length === 0}
		<div class="empty card">
			<strong>No profiles yet</strong>
			<span>Paste a <span class="mono">vless://</span> link to connect.</span>
			<button class="primary" onclick={onadd}>Paste link</button>
		</div>
	{/if}

	<ul>
		{#each app.profiles as profile (profile.id)}
			<li class="card" class:active={profile.id === app.activeId}>
				<button class="pick" disabled={app.locked} onclick={() => app.select(profile.id)}>
					<span class="radio"></span>
					<span class="text">
						{#if editingId === profile.id}
							<input
								class="rename"
								value={profile.name}
								use:focusOnMount
								onblur={(e) => commit(profile.id, e)}
								onkeydown={(e) => {
									if (e.key === 'Enter') e.currentTarget.blur();
									if (e.key === 'Escape') editingId = null;
								}}
								onclick={(e) => e.stopPropagation()}
							/>
						{:else}
							<strong ondblclick={() => (editingId = profile.id)}>{profile.name}</strong>
						{/if}
						<small class="mono">{profile.server}:{profile.port}</small>
					</span>
				</button>
				<span class="badge">VLESS · TLS</span>
				<button
					class="ping {latencyTone(app.latency[profile.id])}"
					onclick={() => app.ping(profile.id)}
					title="Test TCP latency"
				>
					{latencyLabel(app.latency[profile.id])}
				</button>
				<button class="del" disabled={app.locked} onclick={() => app.remove(profile.id)} aria-label="Delete {profile.name}">
					✕
				</button>
			</li>
		{/each}
	</ul>
</section>

<style>
	header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 8px;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 8px;
	}
	li {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 6px 4px 4px;
		transition: border-color 0.15s;
	}
	li.active {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 9%, var(--surface));
	}
	.pick {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 10px;
		border: 0;
		background: none;
		text-align: left;
		padding: 7px 8px;
		min-width: 0;
	}
	.radio {
		width: 14px;
		height: 14px;
		border-radius: 50%;
		border: 2px solid var(--hairline);
		flex: none;
	}
	li.active .radio {
		border-color: var(--accent);
		background: radial-gradient(var(--accent) 38%, transparent 42%);
	}
	.text {
		display: grid;
		min-width: 0;
	}
	small {
		color: var(--fg-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.rename {
		font-weight: 600;
		background: var(--surface-strong);
		border: 1px solid var(--accent);
		border-radius: 6px;
		padding: 1px 5px;
		width: 100%;
		user-select: text;
	}
	.badge {
		font-size: 10px;
		padding: 2px 6px;
		border-radius: 6px;
		background: var(--hairline);
		color: var(--fg-muted);
		white-space: nowrap;
	}
	.ping,
	.del,
	.add {
		border: 0;
		background: none;
		padding: 4px 8px;
		border-radius: 8px;
	}
	.ping {
		font-size: 12px;
		min-width: 62px;
		color: var(--fg-muted);
		font-variant-numeric: tabular-nums;
	}
	.ping.ok {
		color: var(--ok);
	}
	.ping.warn {
		color: var(--warn);
	}
	.ping.bad {
		color: var(--danger);
	}
	.del {
		color: var(--fg-muted);
	}
	.add {
		color: var(--accent);
		font-weight: 600;
	}
	.empty {
		display: grid;
		justify-items: center;
		gap: 6px;
		padding: 22px 16px;
		text-align: center;
		color: var(--fg-muted);
	}
	.empty strong {
		color: var(--fg);
		font-size: 14px;
	}
	.primary {
		margin-top: 6px;
		border: 0;
		border-radius: 9px;
		padding: 7px 16px;
		background: var(--accent);
		color: white;
		font-weight: 600;
	}
</style>
