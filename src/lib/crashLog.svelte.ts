/** Whether the crash log dialog is open; the dialog itself is mounted once in the main page. */
class CrashLogView {
	open = $state(false);

	show() {
		this.open = true;
	}
}

export const crashLog = new CrashLogView();
