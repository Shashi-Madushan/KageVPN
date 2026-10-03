import { feedbackApi, type FeedbackInput, type FeedbackKind } from './api';

/** The feedback draft lives here so closing the form after a failed send never loses the text. */
class Feedback {
	open = $state(false);
	kind = $state<FeedbackKind>('bug');
	message = $state('');
	contact = $state('');
	includeDiagnostics = $state(false);
	sending = $state(false);
	/** Reference returned by the backend after a successful send. */
	sentId = $state<string | null>(null);
	sent = $state(false);
	error = $state<string | null>(null);

	input = $derived<FeedbackInput>({
		kind: this.kind,
		message: this.message,
		contact: this.contact.trim() || null,
		includeDiagnostics: this.includeDiagnostics
	});

	show(kind?: FeedbackKind) {
		if (kind) this.kind = kind;
		this.error = null;
		this.open = true;
	}

	close() {
		this.open = false;
		// A finished send starts the next time with an empty form.
		if (this.sent) this.reset();
	}

	reset() {
		this.message = '';
		this.contact = '';
		this.includeDiagnostics = false;
		this.sent = false;
		this.sentId = null;
		this.error = null;
	}

	async send() {
		if (this.sending || !this.message.trim()) return;
		this.sending = true;
		this.error = null;
		try {
			const result = await feedbackApi.send(this.input);
			this.sentId = result.id;
			this.sent = true;
		} catch (e) {
			this.error = String(e);
		} finally {
			this.sending = false;
		}
	}
}

export const feedback = new Feedback();
