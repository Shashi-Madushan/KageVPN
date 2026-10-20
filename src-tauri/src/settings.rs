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

/// Where country and category lists are downloaded from. The two addresses are templates: `{name}`
/// stands for the country or category (`cn`, `category-ads-all`). An empty one means the source
/// has no lists of that kind.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RuleProvider {
    pub id: String,
    pub name: String,
    pub geoip: String,
    pub geosite: String,
    /// A line about the source, for the built-in ones.
    pub note: String,
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
    /// Hand out made-up addresses for lookups (full-system mode only), so sites are routed by name.
    pub fake_ip: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RoutingSettings {
    pub mode: RouteMode,
    pub bypass_lan: bool,
    /// Whether IPv6 is allowed. Off blocks IPv6 traffic and never hands out IPv6 addresses, for servers without IPv6.
    pub ipv6: bool,
    pub final_action: Action,
    pub rules: Vec<Rule>,
    pub rule_sets: Vec<RuleSetRef>,
    /// The id of the source the lists come from; a built-in one or one of `custom_providers`.
    pub rule_provider: String,
    pub custom_providers: Vec<RuleProvider>,
    pub dns: DnsSettings,
}

impl Default for RoutingSettings {
    fn default() -> Self {
        Self {
            mode: RouteMode::Global,
            bypass_lan: true,
            ipv6: true,
            final_action: Action::Proxy,
            rules: Vec::new(),
            rule_sets: Vec::new(),
            rule_provider: crate::rulesets::DEFAULT_PROVIDER.into(),
            custom_providers: Vec::new(),
            dns: DnsSettings::default(),
        }
    }
}

/// How much sing-box writes to its log. Anything above `Errors` lists the sites that were visited
/// (checked against sing-box 1.14: `info` and `debug` log every connection's destination; `warn`
/// only the ones that failed).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Errors,
    #[default]
    Normal,
    Detailed,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Errors => "warn",
            LogLevel::Normal => "info",
            LogLevel::Detailed => "debug",
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
    /// Protect the local proxy with a login (user `kage`). Off by default so Proxy Only mode
    /// keeps working for apps that cannot send a login.
    pub proxy_auth: bool,
    pub proxy_password: String,
}

impl Default for ReliabilitySettings {
    fn default() -> Self {
        Self { auto_reconnect: true, watchdog: true, kill_switch: false, hotkey: false, proxy_auth: false, proxy_password: String::new() }
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

/// What to do when the Mac joins a known network.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkAction {
    Connect,
    Disconnect,
    #[default]
    Ignore,
}

/// A network the app has seen, identified by its gateway's hardware address.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KnownNetwork {
    pub id: String,
    pub name: String,
    pub action: NetworkAction,
    pub last_seen: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NetworkSettings {
    pub enabled: bool,
    pub networks: Vec<KnownNetwork>,
}
