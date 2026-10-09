import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type Mode = 'tun' | 'proxy';
export type ConnState = 'disconnected' | 'connecting' | 'connected';
export type Level = 'debug' | 'info' | 'warn' | 'error';

export type Protocol = 'vless' | 'trojan' | 'shadowsocks' | 'hysteria2';

/** One server. Everything specific to its protocol is in `settings`; the editor reads and writes
 *  it by the key paths each protocol lists (see `FormField`), so no per-protocol types are needed. */
export interface Profile {
	id: string;
	protocol: Protocol;
	name: string;
	server: string;
	port: number;
	settings: Record<string, unknown>;
	group: string | null;
	favorite: boolean;
	subscriptionId: string | null;
	/** Short description for lists, such as "Reality · grpc". Added by the backend; ignored on save. */
	summary?: string;
}

export interface Status {
	state: ConnState;
	mode: Mode;
	since: number | null;
	error: string | null;
	/** Whether the tunnel carries traffic; only meaningful while connected. */
	health: Health;
	reason: { cause: 'serverUnreachable' | 'handshake' | 'noTraffic'; message: string } | null;
	latencyMs: number | null;
	/** Why the last attempt to connect failed. */
	failure: ErrorInfo | null;
}

export interface ErrorInfo {
	code: string;
	title: string;
	detail: string;
	action: 'retry' | 'installEngine' | 'diagnostics' | null;
}

export type Health = 'idle' | 'verifying' | 'healthy' | 'down';

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

export interface AutoSettings {
	enabled: boolean;
	group: string | null;
}

export interface UserInfo {
	upload: number;
	download: number;
	total: number;
	expire: number;
}

export interface Subscription {
	id: string;
	name: string;
	url: string;
	intervalHours: number;
	lastUpdated: number;
	lastError: string | null;
	userInfo: UserInfo | null;
}

export type RouteMode = 'global' | 'rule' | 'direct';
export type RouteAction = 'proxy' | 'direct' | 'block';
export type RuleKind = 'domain' | 'domainSuffix' | 'domainKeyword' | 'ipCidr' | 'processName';
export type RuleSetKind = 'geoip' | 'geosite';
export type DnsProvider = 'cloudflare' | 'google' | 'quad9' | 'custom' | 'system';

export interface Rule {
	id: string;
	enabled: boolean;
	kind: RuleKind;
	value: string;
	action: RouteAction;
}

export interface RuleSetRef {
	id: string;
	enabled: boolean;
	kind: RuleSetKind;
	country: string;
	action: RouteAction;
}

export interface RoutingSettings {
	mode: RouteMode;
	bypassLan: boolean;
	finalAction: RouteAction;
	rules: Rule[];
	ruleSets: RuleSetRef[];
	dns: { provider: DnsProvider; custom: string };
}

export interface ReliabilitySettings {
	autoReconnect: boolean;
	watchdog: boolean;
	killSwitch: boolean;
	hotkey: boolean;
}

export interface UsageSettings {
	paused: boolean;
	recordApps: boolean;
	recordDomains: boolean;
	detailRetentionDays: number;
	quotaBytes: number | null;
	cycleDay: number;
	quotaAction: 'notify' | 'disconnect';
	alertCycle: number;
	alertLevel: number;
}

export type Bucket = 'hour' | 'day' | 'week' | 'month';

export interface Filters {
	profileId?: string | null;
	mode?: string | null;
	app?: string | null;
	domain?: string | null;
}

export interface Point {
	start: number;
	up: number;
	down: number;
}

export interface UsageReport {
	series: Point[];
	total: { up: number; down: number };
	previous: { up: number; down: number } | null;
	peak: Point | null;
	connectedSecs: number;
}

export interface TopRow {
	key: string;
	up: number;
	down: number;
}

export interface QuotaStatus {
	limit: number | null;
	used: number;
	up: number;
	down: number;
	cycleStart: number;
	cycleEnd: number;
	percent: number;
}

export interface ImportSummary {
	added: number;
	duplicates: number;
	failed: string[];
}

export interface RefreshSummary {
	added: number;
	removed: number;
	total: number;
	skipped: number;
}

export interface NodeLatency {
	id: string;
	ms: number | null;
}

export interface LiveConnection {
	id: string;
	host: string;
	destinationIp: string;
	destinationPort: string;
	process: string;
	network: string;
	up: number;
	down: number;
	start: string;
	chain: string;
	rule: string;
}

export interface IpInfo {
	ip: string;
	country: string;
	city: string;
	org: string;
}

export interface Diagnostics {
	system: IpInfo | null;
	tunnel: IpInfo | null;
	dns: { ip: string; geo: string } | null;
}

export interface SpeedResult {
	downMbps: number;
	upMbps: number;
	pingMs: number;
	jitterMs: number;
}

export type SpeedPhase = 'ping' | 'download' | 'upload' | 'done';

/** A live reading streamed while the speed test runs. */
export interface SpeedProgress {
	phase: SpeedPhase;
	mbps: number;
	progress: number;
	pingMs: number | null;
	jitterMs: number | null;
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
	auto: AutoSettings;
	subscriptions: Subscription[];
	routing: RoutingSettings;
	reliability: ReliabilitySettings;
	hotkey: string;
}

export const api = {
	appInfo: () => invoke<AppInfo>('app_info'),
	snapshot: () => invoke<Snapshot>('snapshot'),
	importText: (text: string) => invoke<ImportSummary>('import_text', { text }),
	newProfile: (protocol: string) => invoke<Profile>('new_profile', { protocol }),
	saveProfile: (profile: Profile) => invoke<Profile>('save_profile', { profile }),
	duplicateProfile: (id: string) => invoke<Profile>('duplicate_profile', { id }),
	setFavorite: (id: string, favorite: boolean) => invoke<void>('set_favorite', { id, favorite }),
	setGroup: (id: string, group: string | null) => invoke<void>('set_group', { id, group }),
	reorderProfiles: (ids: string[]) => invoke<void>('reorder_profiles', { ids }),
	profileLink: (id: string) => invoke<string>('profile_link', { id }),
	profileQr: (id: string) => invoke<string>('profile_qr', { id }),
	setAuto: (settings: AutoSettings) => invoke<void>('set_auto', { settings }),
	testAll: () => invoke<NodeLatency[]>('test_all'),
	setRouting: (settings: RoutingSettings) => invoke<void>('set_routing', { settings }),
	setReliability: (settings: ReliabilitySettings) => invoke<void>('set_reliability', { settings }),
	restoreNetwork: () => invoke<void>('restore_network'),
	rootServiceInstalled: () => invoke<boolean>('root_service_installed'),
	installRootService: () => invoke<void>('install_root_service'),
	uninstallRootService: () => invoke<void>('uninstall_root_service'),
	addSubscription: (name: string, url: string) => invoke<RefreshSummary>('add_subscription', { name, url }),
	refreshSubscription: (id: string) => invoke<RefreshSummary>('refresh_subscription', { id }),
	removeSubscription: (id: string) => invoke<void>('remove_subscription', { id }),
	runDiagnostics: () => invoke<Diagnostics>('run_diagnostics'),
	runSpeedTest: () => invoke<SpeedResult>('run_speed_test'),
	cancelSpeedTest: () => invoke<void>('cancel_speed_test'),
	onSpeedProgress: (handler: (progress: SpeedProgress) => void): Promise<UnlistenFn> =>
		listen<SpeedProgress>('speed-progress', (event) => handler(event.payload)),
	connections: () => invoke<LiveConnection[]>('connections_list'),
	closeConnection: (id: string) => invoke<void>('close_connection', { id }),
	closeAllConnections: () => invoke<void>('close_all_connections'),
	latencyHistory: () => invoke<Record<string, (number | null)[]>>('latency_history'),
	deleteProfile: (id: string) => invoke<void>('delete_profile', { id }),
	selectProfile: (id: string) => invoke<void>('select_profile', { id }),
	setMode: (mode: Mode) => invoke<void>('set_mode', { mode }),
	trustKey: (id: string, key: string) => invoke<void>('trust_key', { id, key }),
	clearLogs: () => invoke<void>('clear_logs'),
	crashLog: () => invoke<string>('crash_log'),
	clearCrashLog: () => invoke<void>('clear_crash_log'),
	connect: () => invoke<void>('connect'),
	disconnect: () => invoke<void>('disconnect'),
	pingProfile: (id: string) => invoke<number>('ping_profile', { id }),
	testConnection: () => invoke<number>('test_connection'),
	exitIp: () => invoke<string>('exit_ip'),
	renameProfile: (id: string, name: string) => invoke<void>('rename_profile', { id, name }),
	updateSettings: (settings: Settings) => invoke<void>('update_settings', { settings }),
	setLaunchAtLogin: (enabled: boolean) => invoke<void>('set_launch_at_login', { enabled }),
	openMainWindow: () => invoke<void>('open_main_window'),
	hideTray: () => invoke<void>('hide_tray'),
	notifyOs: (title: string, body: string) => invoke<void>('notify_os', { title, body }),
	quitApp: () => invoke<void>('quit_app')
};

export type Channel = 'stable' | 'prerelease';
export type UpdatePolicy = 'ask' | 'auto' | 'off';

export interface CoreInfo {
	path: string;
	version: string;
	tags: string[];
	source: 'system' | 'managed';
	missingTags: string[];
	outdated: boolean;
}

export interface InstalledCore {
	version: string;
	path: string;
	installedAt: number | null;
}

export interface CoreSettings {
	managedVersion: string | null;
	channel: Channel;
	autoCheck: boolean;
	lastCheck: number;
	updatePolicy: UpdatePolicy;
	pendingVersion: string | null;
}

export interface CoreRef {
	id: string;
	name: string;
	/** False for cores the app can download but cannot run connections with yet. */
	runnable: boolean;
	minVersion: string;
	requiredFeatures: string[];
}

export interface AppInfo {
	name: string;
	brand: string;
	version: string;
	updatesConfigured: boolean;
	feedbackConfigured: boolean;
	protocols: ProtocolInfo[];
}

export type FieldKind = 'text' | 'secret' | 'select' | 'segmented' | 'toggle' | 'list';
export type FieldSection = 'connection' | 'security' | 'transport' | 'advanced';

export interface FieldChoice {
	value: string;
	label: string;
}

/** One editable setting of a protocol, described by the backend. */
export interface FormField {
	/** Where the value lives in the profile, such as `sni` or `transport.path`. */
	key: string;
	label: string;
	kind: FieldKind;
	section: FieldSection;
	choices: FieldChoice[];
	optional: boolean;
	mono: boolean;
	placeholder: string | null;
	warning: string | null;
	showWhen: { key: string; values: string[] } | null;
	pattern: { regex: string; message: string } | null;
}

export interface ProtocolInfo {
	id: Protocol;
	label: string;
	schemes: string[];
	fields: FormField[];
}

export interface CoreStatus {
	core: CoreRef;
	cores: CoreRef[];
	active: CoreInfo | null;
	error: string | null;
	system: CoreInfo | null;
	installed: InstalledCore[];
	settings: CoreSettings;
	busy: boolean;
}

export interface Release {
	version: string;
	prerelease: boolean;
	publishedAt: string | null;
	notes: string;
	size: number;
	url: string;
	sha256: string | null;
}

export type DownloadKind = 'app' | 'core';

export interface DownloadProgress {
	id: string;
	kind: DownloadKind;
	title: string;
	version: string;
	stage: 'download' | 'verify' | 'extract' | 'check' | 'install' | 'done';
	received: number;
	total: number;
}

export interface DownloadFinished {
	id: string;
	kind: DownloadKind;
	title: string;
	version: string | null;
	error: string | null;
}

export interface UpdateAvailable {
	id: string;
	kind: DownloadKind;
	name: string;
	version: string;
}

export const downloadsApi = {
	cancel: () => invoke<void>('download_cancel')
};

export const coreApi = {
	status: () => invoke<CoreStatus>('core_status'),
	check: () => invoke<Release[]>('core_check'),
	use: (version: string | null) => invoke<void>('core_use', { version }),
	remove: (version: string) => invoke<void>('core_remove', { version }),
	installBackground: (version: string | null, allowUnverified = false) =>
		invoke<void>('core_install_background', { version, allowUnverified }),
	setActive: (id: string) => invoke<void>('core_set_active', { id }),
	setChannel: (channel: Channel) => invoke<void>('set_core_channel', { channel }),
	setPolicy: (policy: UpdatePolicy) => invoke<void>('set_core_update_policy', { policy }),
	brewUpgrade: () => invoke<string>('core_brew_upgrade')
};

export const usageApi = {
	report: (args: { from: number; to: number; bucket: Bucket; filters: Filters; compare: boolean }) =>
		invoke<UsageReport>('usage_report', { args }),
	top: (args: { kind: 'app' | 'domain' | 'profile'; from: number; to: number; limit: number; filters: Filters }) =>
		invoke<TopRow[]>('usage_top', { args }),
	exportCsv: (args: { from: number; to: number; bucket: Bucket; filters: Filters; compare: boolean }) =>
		invoke<string>('usage_export', { args }),
	settings: () => invoke<UsageSettings>('usage_get_settings'),
	setSettings: (settings: UsageSettings) => invoke<void>('usage_set_settings', { settings }),
	quota: () => invoke<QuotaStatus>('usage_quota'),
	reset: () => invoke<void>('usage_reset'),
	clearDetails: () => invoke<void>('usage_clear_details')
};

export type FeedbackKind = 'bug' | 'idea' | 'question';

export interface FeedbackInput {
	kind: FeedbackKind;
	message: string;
	contact: string | null;
	includeDiagnostics: boolean;
}

export interface FeedbackResult {
	/** Reference number the backend returned, if any. */
	id: string | null;
}

export const feedbackApi = {
	/** The exact JSON that would be sent, after redaction. */
	preview: (input: FeedbackInput) => invoke<string>('feedback_preview', { input }),
	send: (input: FeedbackInput) => invoke<FeedbackResult>('feedback_send', { input })
};

export interface AppUpdateInfo {
	version: string;
	notes: string | null;
}

export interface AppUpdateSettings {
	policy: UpdatePolicy;
	lastCheck: number;
}

export const appUpdateApi = {
	check: () => invoke<AppUpdateInfo | null>('app_update_check'),
	install: () => invoke<void>('app_update_install'),
	settings: () => invoke<AppUpdateSettings>('app_update_settings'),
	setPolicy: (policy: UpdatePolicy) => invoke<void>('set_app_update_policy', { policy }),
	restart: (force: boolean) => invoke<void>('app_restart', { force })
};
