use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RouteMode {
    #[default]
    Global,
    Rule,
    Direct,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    #[default]
    Proxy,
    Direct,
    Block,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuleKind {
    #[default]
    Domain,
    DomainSuffix,
    DomainKeyword,
    IpCidr,
    ProcessName,
    /// An installed app, stored as its bundle path (`/Applications/Slack.app`). Matches every
    /// process inside the bundle, so helpers such as "Slack Helper (Renderer)" follow the app.
    App,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Rule {
    pub id: String,
    pub enabled: bool,
    pub kind: RuleKind,
    pub value: String,
    pub action: Action,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleSetKind {
    #[default]
    Geoip,
    Geosite,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RuleSetRef {
    pub id: String,
    pub enabled: bool,
    pub kind: RuleSetKind,
    pub country: String,
    pub action: Action,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DnsProvider {
    #[default]
    Cloudflare,
    Google,
    Quad9,
    Custom,
    System,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DnsSettings {
    pub provider: DnsProvider,
    pub custom: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RoutingSettings {
    pub mode: RouteMode,
    pub bypass_lan: bool,
    pub final_action: Action,
    pub rules: Vec<Rule>,
    pub rule_sets: Vec<RuleSetRef>,
    pub dns: DnsSettings,
}

impl Default for RoutingSettings {
    fn default() -> Self {
        Self {
            mode: RouteMode::Global,
            bypass_lan: true,
            final_action: Action::Proxy,
            rules: Vec::new(),
            rule_sets: Vec::new(),
            dns: DnsSettings::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuotaAction {
    #[default]
    Notify,
    Disconnect,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UsageSettings {
    pub paused: bool,
    pub record_apps: bool,
    pub record_domains: bool,
    pub detail_retention_days: u32,
    pub quota_bytes: Option<u64>,
    pub cycle_day: u8,
    pub quota_action: QuotaAction,
    pub alert_cycle: i64,
    pub alert_level: u8,
}

impl Default for UsageSettings {
    fn default() -> Self {
        Self {
            paused: false,
            record_apps: true,
            record_domains: false,
            detail_retention_days: 90,
            quota_bytes: None,
            cycle_day: 1,
            quota_action: QuotaAction::Notify,
            alert_cycle: 0,
            alert_level: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReliabilitySettings {
    pub auto_reconnect: bool,
    pub watchdog: bool,
    pub kill_switch: bool,
    pub hotkey: bool,
}

impl Default for ReliabilitySettings {
    fn default() -> Self {
        Self { auto_reconnect: true, watchdog: false, kill_switch: false, hotkey: false }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoSettings {
    pub enabled: bool,
    pub group: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UserInfo {
    pub upload: u64,
    pub download: u64,
    pub total: u64,
    pub expire: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Subscription {
    pub id: String,
    pub name: String,
    pub url: String,
    pub interval_hours: u32,
    pub last_updated: u64,
    pub last_error: Option<String>,
    pub user_info: Option<UserInfo>,
    /// The expiry date the last expiry notice was for, and which one it was (1 soon, 2 expired),
    /// so each notice appears once per date instead of on every refresh.
    pub notified_expire: u64,
    pub notified_stage: u8,
    /// When the last refresh was tried (successful or not), and how many in a row failed.
    pub last_attempt: u64,
    pub failures: u32,
    /// Sent instead of the app's own user agent. Providers choose the list format, or whether to
    /// answer at all, by the client name.
    pub user_agent: Option<String>,
    /// Send this Mac's device ID with each refresh, for providers that limit how many devices can
    /// use one subscription. Off unless the user turns it on.
    pub send_device_id: bool,
    /// The data warning last shown for the provider's allowance (1 at 90 percent, 2 used up), so
    /// each is shown once; it drops back to 0 when a new period starts.
    pub notified_quota: u8,
}

impl Default for Subscription {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            url: String::new(),
            interval_hours: 12,
            last_updated: 0,
            last_error: None,
            user_info: None,
            notified_expire: 0,
            notified_stage: 0,
            last_attempt: 0,
            failures: 0,
            user_agent: None,
            send_device_id: false,
            notified_quota: 0,
        }
    }
}
