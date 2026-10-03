<script lang="ts">
	import { feedbackApi, type FeedbackKind } from '$lib/api';
	import { feedback } from '$lib/feedback.svelte';
	import Dialog from './Dialog.svelte';
	import Segmented from './Segmented.svelte';

	const kinds: { value: FeedbackKind; label: string }[] = [
		{ value: 'bug', label: 'Problem' },
		{ value: 'idea', label: 'Idea' },
		{ value: 'question', label: 'Question' }
	];
	const placeholders: Record<FeedbackKind, string> = {
		bug: 'What happened, and what did you expect?',
		idea: 'What would make this app better for you?',
		question: 'What would you like to know?'
	};

	let showPreview = $state(false);
	let preview = $state('');
	let previewError = $state<string | null>(null);

	// Refresh the preview a moment after the draft changes, only while it is visible.
	$effect(() => {
		const { input } = feedback;
		if (!feedback.open || !showPreview) return;
		const timer = setTimeout(async () => {
			try {
				preview = await feedbackApi.preview(input);
				previewError = null;
			} catch (e) {
				preview = '';
				previewError = String(e);
			}
		}, 250);
		return () => clearTimeout(timer);
	});
</script>

<Dialog
	open={feedback.open}
	title="Send feedback"
	subtitle={feedback.sent ? undefined : 'Tell us what happened or what you would like.'}
	width={460}
	onclose={() => feedback.close()}
>
	{#if feedback.sent}
		<div class="done" role="status">
			<strong>Thanks, your feedback was sent.</strong>
			{#if feedback.sentId}<span>Reference: <span class="mono">{feedback.sentId}</span></span>{/if}
		</div>
	{:else}
		<div class="form">
			<Segmented options={kinds} value={feedback.kind} label="Type of feedback" onchange={(k) => (feedback.kind = k)} />
			<textarea
				class="input"
				rows="5"
				maxlength="5000"
				placeholder={placeholders[feedback.kind]}
				aria-label="Message"
				bind:value={feedback.message}
			></textarea>
			<input class="input" type="text" maxlength="200" placeholder="Email or handle, if you want a reply (optional)" aria-label="Contact" bind:value={feedback.contact} />

			<label class="check">
				<input type="checkbox" bind:checked={feedback.includeDiagnostics} />
				<span>
					Include diagnostics
					<small>App, engine and macOS versions, connection state and recent logs. Server addresses, IDs and keys are removed.</small>
				</span>
			</label>

			<button type="button" class="linkbtn toggle" aria-expanded={showPreview} onclick={() => (showPreview = !showPreview)}>
				{showPreview ? 'Hide' : 'Show'} exactly what will be sent
			</button>
			{#if showPreview}
				{#if previewError}
					<p class="hint">{previewError}</p>
				{:else}
					<pre class="preview mono" aria-label="Report preview">{preview}</pre>
				{/if}
			{/if}

			{#if feedback.error}<p class="error" role="alert">{feedback.error}</p>{/if}
		</div>
	{/if}

	{#snippet footer()}
		{#if feedback.sent}
			<button class="btn primary" onclick={() => feedback.close()}>Close</button>
		{:else}
			<button class="btn" onclick={() => feedback.close()}>Cancel</button>
			<button class="btn primary" disabled={!feedback.message.trim() || feedback.sending} onclick={() => feedback.send()}>
				{feedback.sending ? 'Sending…' : feedback.error ? 'Try again' : 'Send'}
			</button>
		{/if}
	{/snippet}
</Dialog>

<style>
	.form {
		display: grid;
		gap: 12px;
	}
	textarea {
		resize: vertical;
		min-height: 96px;
		font-family: inherit;
	}
	.check {
		display: flex;
		gap: 10px;
		align-items: flex-start;
		font-size: 13px;
		cursor: pointer;
	}
	.check input {
		margin-top: 2px;
	}
	.check small {
		display: block;
		margin-top: 2px;
		color: var(--fg-muted);
		font-size: 12px;
		line-height: 1.45;
	}
	.toggle {
		justify-self: start;
	}
	.preview {
		margin: 0;
		padding: 10px;
		max-height: 200px;
		overflow: auto;
		border-radius: 8px;
		background: var(--hairline);
		font-size: 11px;
		line-height: 1.45;
		white-space: pre-wrap;
		word-break: break-word;
		user-select: text;
	}
	.hint {
		margin: 0;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.error {
		margin: 0;
		font-size: 12px;
		color: var(--danger);
	}
	.done {
		display: grid;
		gap: 6px;
		padding: 18px 4px;
		font-size: 13px;
		line-height: 1.5;
	}
	.done span {
		color: var(--fg-muted);
	}
</style>
