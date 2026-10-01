import { invoke } from '@tauri-apps/api/core';

export type Mode = 'tun' | 'proxy';
export type ConnState = 'disconnected' | 'connecting' | 'connected';
export type Level = 'debug' | 'info' | 'warn' | 'error';

export interface Profile {
	id: string;
	name: string;
	server: string;
	port: number;
	uuid: string;
	sni: string | null;
	fingerprint: string | null;
	alpn: string[];
	flow: string | null;
	pinnedKey: string | null;
}

export interface Status {
	state: ConnState;
	mode: Mode;
	since: number | null;
	error: string | null;
}

export type Hint = { kind: 'keyMismatch'; key: string } | { kind: 'unresolved' };

export interface LogLine {
	level: Level;
	text: string;
	hint: Hint | null;
}

export interface Sample {
	up: number;
	down: number;
}

export interface Traffic {
	up: number;
	down: number;
	totalUp: number;
	totalDown: number;
}

export interface Settings {
	showSpeed: boolean;
	autoConnect: boolean;
}

export interface Snapshot {
	profiles: Profile[];
	activeId: string | null;
	mode: Mode;
	status: Status;
	logs: LogLine[];
	traffic: Traffic;
	history: Sample[];
	settings: Settings;
	launchAtLogin: boolean;
}

export const api = {
	snapshot: () => invoke<Snapshot>('snapshot'),
	importProfile: (link: string) => invoke<Profile>('import_profile', { link }),
	deleteProfile: (id: string) => invoke<void>('delete_profile', { id }),
	selectProfile: (id: string) => invoke<void>('select_profile', { id }),
	setMode: (mode: Mode) => invoke<void>('set_mode', { mode }),
	trustKey: (id: string, key: string) => invoke<void>('trust_key', { id, key }),
	clearLogs: () => invoke<void>('clear_logs'),
	connect: () => invoke<void>('connect'),
	disconnect: () => invoke<void>('disconnect'),
	pingProfile: (id: string) => invoke<number>('ping_profile', { id }),
	testConnection: () => invoke<number>('test_connection'),
	exitIp: () => invoke<string>('exit_ip'),
	renameProfile: (id: string, name: string) => invoke<void>('rename_profile', { id, name }),
	updateSettings: (settings: Settings) => invoke<void>('update_settings', { settings }),
	setLaunchAtLogin: (enabled: boolean) => invoke<void>('set_launch_at_login', { enabled }),
	openMainWindow: () => invoke<void>('open_main_window'),
	quitApp: () => invoke<void>('quit_app')
};
