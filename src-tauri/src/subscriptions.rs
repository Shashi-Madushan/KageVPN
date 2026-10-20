use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio::time::sleep;

use crate::error::{AppError, Result};
use crate::notify;
use crate::profile::{contains_link, parse_many, Profile};
use crate::settings::{Subscription, UserInfo};
use crate::state::AppState;
use crate::store::Store;

const MAX_BODY_BYTES: usize = 5 * 1024 * 1024;
const SCHEDULER_TICK: Duration = Duration::from_secs(30 * 60);
const DAY_SECS: u64 = 24 * 3600;
/// After a failed refresh the first retry comes this long after; each further failure doubles it.
const RETRY_BASE_SECS: u64 = 30 * 60;

/// What a download produced: the servers, the provider's usage figures, and why links were skipped.
type Fetched = (Vec<Profile>, Option<UserInfo>, Vec<String>);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshSummary {
    pub added: usize,
    pub removed: usize,
    pub total: usize,
    pub skipped: usize,
    /// Why the first skipped link was skipped.
    pub skipped_reason: Option<String>,
}

/// How close a subscription's end is, from the first heads-up to the end itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expiry {
    /// 7 days or less left.
    Week,
    /// 3 days or less.
    Soon,
    /// 1 day or less.
    LastDay,
    Expired,
}

impl Expiry {
    /// The number stored in `notified_stage`; higher is later.
    fn rank(self) -> u8 {
        match self {
            Expiry::Week => 1,
            Expiry::Soon => 2,
            Expiry::LastDay => 3,
            Expiry::Expired => 4,
        }
    }
}

/// Which expiry notice is due now, if any. `announced` is the `(date, stage)` last shown, so each
/// stage is announced once per expiry date; renewing the subscription gives a new date and starts over.
pub fn due_expiry_notice(expire: u64, now: u64, announced: (u64, u8)) -> Option<Expiry> {
    if expire == 0 {
        return None;
    }
    let stage = if expire <= now {
        Some(Expiry::Expired)
    } else if expire - now <= DAY_SECS {
        Some(Expiry::LastDay)
    } else if expire - now <= 3 * DAY_SECS {
        Some(Expiry::Soon)
    } else if expire - now <= 7 * DAY_SECS {
        Some(Expiry::Week)
    } else {
        None
    }?;
    // Only the latest stage reached is announced, so a plan first seen with 2 days left says so once.
    let seen = if announced.0 == expire { announced.1 } else { 0 };
    (stage.rank() > seen).then_some(stage)
}

/// Applies the expiry warning that is due for a subscription's stored figures, if any, and
/// remembers it so it is not shown twice.
pub fn expiry_pass(subscription: &mut Subscription, now: u64) -> Option<Notice> {
    let expire = subscription.user_info.as_ref()?.expire;
    let stage = due_expiry_notice(expire, now, (subscription.notified_expire, subscription.notified_stage))?;
    subscription.notified_expire = expire;
    subscription.notified_stage = stage.rank();
    Some(Notice::Expiry { name: subscription.name.clone(), expire, stage })
}

pub fn validate_url(url: &str) -> Result<()> {
    let parsed = url::Url::parse(url.trim()).map_err(|_| AppError::Invalid("not a valid URL".into()))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::Invalid("subscription links must start with http:// or https://".into()));
    }
    Ok(())
}

/// What a failed request means for a person: the server is not reachable from here.
fn explain(error: reqwest::Error) -> AppError {
    let why = if error.is_timeout() {
        "did not answer in time"
    } else if error.is_connect() {
        "could not be reached"
    } else {
        return error.without_url().into();
    };
    AppError::ServerUnreachable(format!(
        "The subscription server {why}. If your provider is blocked on this network, connect to a server first and refresh again."
    ))
}

/// How a subscription asks for its list: the client name it gives and, when the user turned it
/// on, the device headers for providers that limit devices.
#[derive(Clone, Debug, Default)]
pub struct FetchOptions {
    pub user_agent: Option<String>,
    pub device_id: Option<String>,
}

/// Longest client name accepted; real ones are a few dozen characters.
const MAX_USER_AGENT: usize = 200;

/// A client name typed by the user, tidied: `None` for "use the app's own".
pub fn clean_user_agent(text: Option<&str>) -> Result<Option<String>> {
    let Some(text) = text.map(str::trim).filter(|t| !t.is_empty()) else { return Ok(None) };
    if text.len() > MAX_USER_AGENT || !text.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
        return Err(AppError::Invalid(format!("the client name must be plain text (letters, digits and punctuation), at most {MAX_USER_AGENT} characters")));
    }
    Ok(Some(text.to_string()))
}

async fn request(url: &str, options: &FetchOptions, through_tunnel: bool) -> std::result::Result<reqwest::Response, reqwest::Error> {
    let user_agent = options.user_agent.clone().unwrap_or_else(crate::app_config::user_agent);
    let mut client = reqwest::Client::builder().user_agent(user_agent).timeout(Duration::from_secs(20));
    if through_tunnel {
        client = client.proxy(reqwest::Proxy::all(crate::config::proxy_url())?);
    }
    let mut request = client.build()?.get(url.trim());
    if let Some(id) = &options.device_id {
        for (name, value) in crate::device::subscription_headers(id) {
            request = request.header(name, value);
        }
    }
    request.send().await
}

/// Why a provider refused the request, in words that say what to do. Device-limited panels mark
/// their answers with `x-hwid-*` headers; without the device ID they answer 404, which on its own
/// looks like a wrong link.
pub fn refusal(status: u16, header: impl Fn(&str) -> bool, sent_device_id: bool) -> Option<String> {
    if header("x-hwid-max-devices-reached") || header("x-hwid-limit") {
        return Some("This subscription has reached its device limit. Remove a device you no longer use in your provider's panel (or ask your provider), then refresh.".into());
    }
    if !sent_device_id && (header("x-hwid-not-supported") || (status == 404 && header("x-hwid-active"))) {
        return Some("Your provider limits how many devices can use this subscription and needs this Mac's device ID. Edit the subscription, turn on “Send device ID”, then refresh.".into());
    }
    if (200..300).contains(&status) {
        return None;
    }
    let hint = match status {
        401 | 403 => " The link may have expired or been replaced; copy a fresh one from your provider.",
        404 if !sent_device_id => " Check the link. If your provider limits devices, edit the subscription and turn on “Send device ID”.",
        404 => " Check that the link is complete and still valid.",
        429 => " The provider is limiting requests; try again later.",
        500..=599 => " The problem is on the provider's side; try again later.",
        _ => "",
    };
    // The link is not quoted: it usually carries the account's token.
    Some(format!("The subscription server replied {status}.{hint}"))
}

async fn fetch(url: &str, options: &FetchOptions) -> Result<Fetched> {
    validate_url(url)?;
    // Providers are often blocked on the very networks where people need a VPN. If the direct
    // request cannot connect and a tunnel is up, ask through it (Proxy Only mode would otherwise
    // fetch directly even while connected).
    let response = match request(url, options, false).await {
        Ok(response) => response,
        Err(error) if error.is_connect() || error.is_timeout() => {
            if crate::net::proxy_port_open().await {
                request(url, options, true).await.map_err(explain)?
            } else {
                return Err(explain(error));
            }
        }
        Err(error) => return Err(error.without_url().into()),
    };
    let header = |name: &str| response.headers().get(name).and_then(|v| v.to_str().ok()).is_some_and(|v| v.trim().eq_ignore_ascii_case("true"));
    if let Some(why) = refusal(response.status().as_u16(), header, options.device_id.is_some()) {
        return Err(AppError::Invalid(why));
    }

    let user_info = response
        .headers()
        .get("subscription-userinfo")
        .and_then(|v| v.to_str().ok())
        .map(parse_user_info);
    let body = response.bytes().await.map_err(reqwest::Error::without_url)?;
    if body.len() > MAX_BODY_BYTES {
        return Err(AppError::Invalid("the subscription response is too large".into()));
    }
    let text = decode_body(&String::from_utf8_lossy(&body));
    let (profiles, mut errors) = parse_many(&text);
    let (profiles, placeholders) = drop_placeholders(profiles);
    errors.extend(placeholders);
    if profiles.is_empty() {
        let why = errors.first().map(|reason| format!(" ({reason})")).unwrap_or_default();
        return Err(AppError::Invalid(format!("no supported server links were found in this subscription{why}")));
    }
    Ok((profiles, user_info, errors))
}

/// Providers pad their lists with entries that are not servers ("Remaining traffic: 80 GB") and
/// point them at 127.0.0.1 or 0.0.0.0. Importing them only adds dead rows. Returns the real servers
/// and one reason for each entry dropped.
pub fn drop_placeholders(profiles: Vec<Profile>) -> (Vec<Profile>, Vec<String>) {
    let (mut real, mut reasons) = (Vec::new(), Vec::new());
    for profile in profiles {
        if is_placeholder(&profile.server) {
            reasons.push(format!("not a server ({}): {}", profile.server, profile.name));
        } else {
            real.push(profile);
        }
    }
    (real, reasons)
}

fn is_placeholder(server: &str) -> bool {
    let host = server.trim().trim_matches(['[', ']']);
    host.eq_ignore_ascii_case("localhost") || host.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback() || ip.is_unspecified())
}

/// When a subscription is next due. A healthy one waits its interval. One that just failed is tried
/// again sooner, with the wait doubling for each failure in a row (30 minutes, 1 hour, 2 hours, ...)
/// up to its normal interval, so a dead link is neither left alone for half a day after a hiccup
/// nor hit every half hour for ever.
pub fn next_due(subscription: &Subscription) -> u64 {
    let interval = u64::from(subscription.interval_hours.max(1)) * 3600;
    if subscription.failures == 0 {
        return subscription.last_updated + interval;
    }
    let wait = (RETRY_BASE_SECS << subscription.failures.saturating_sub(1).min(6)).min(interval);
    subscription.last_attempt.max(subscription.last_updated) + wait
}

/// Subscriptions are either a plain list of links or the same list base64-encoded.
fn decode_body(text: &str) -> String {
    if contains_link(text) {
        return text.to_string();
    }
    let compact: String = text.split_whitespace().collect();
    [&STANDARD, &STANDARD_NO_PAD, &URL_SAFE, &URL_SAFE_NO_PAD]
        .iter()
        .find_map(|engine| engine.decode(compact.trim_end_matches('=')).ok().or_else(|| engine.decode(&compact).ok()))
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_else(|| text.to_string())
}

fn parse_user_info(header: &str) -> UserInfo {
    let mut info = UserInfo::default();
    for part in header.split(';') {
        let Some((key, value)) = part.split_once('=') else { continue };
        let value = value.trim().parse::<u64>().unwrap_or(0);
        match key.trim() {
            "upload" => info.upload = value,
            "download" => info.download = value,
            "total" => info.total = value,
            "expire" => info.expire = value,
            _ => {}
        }
    }
    info
}

pub async fn refresh(app: &AppHandle, id: &str) -> Result<RefreshSummary> {
    let state = app.state::<AppState>();
    let (url, options) = {
        let mut store = state.store.lock().unwrap();
        let subscription = store
            .subscriptions()
            .iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or_else(|| AppError::Invalid("subscription not found".into()))?;
        let device_id = if subscription.send_device_id { Some(store.device_id()?) } else { None };
        (subscription.url, FetchOptions { user_agent: subscription.user_agent, device_id })
    };

    // No lock is held while the download runs; the result is applied to the entry as it is now.
    let outcome = fetch(&url, &options).await;
    let (result, notices) = record_refresh(&mut state.store.lock().unwrap(), id, outcome, now_secs());
    for notice in notices {
        notify::send(app, "Subscription", &notice.message(now_secs()));
    }
    result
}

/// Something to tell the user after a refresh, once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notice {
    Expiry { name: String, expire: u64, stage: Expiry },
    Data { name: String, stage: DataStage, left: u64 },
}

impl Notice {
    pub fn message(&self, now: u64) -> String {
        match self {
            Notice::Expiry { name, stage: Expiry::Expired, .. } => format!("{name} has expired."),
            Notice::Expiry { name, expire, .. } => format!("{name} expires in {}.", days_left(*expire, now)),
            Notice::Data { name, stage: DataStage::Low, left } => format!("{name}: 90% of your data is used, {} left.", format_bytes(*left)),
            Notice::Data { name, stage: DataStage::UsedUp, .. } => format!("{name}: your data allowance is used up."),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataStage {
    Low = 1,
    UsedUp = 2,
}

/// Which data warning is due for the provider's figures, and the stage to remember. `total` 0
/// means the plan has no data limit. A stage is announced once; dropping below 90 percent (a new
/// period, or a top-up) resets it so the next period warns again.
pub fn due_data_notice(info: &UserInfo, announced: u8) -> (Option<DataStage>, u8) {
    if info.total == 0 {
        return (None, 0);
    }
    let used = info.upload.saturating_add(info.download);
    let stage = if used >= info.total {
        2
    } else if u128::from(used) * 10 >= u128::from(info.total) * 9 {
        1
    } else {
        0
    };
    let due = (stage > announced).then_some(if stage == 2 { DataStage::UsedUp } else { DataStage::Low });
    (due, stage)
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 { format!("{bytes} B") } else { format!("{value:.1} {}", UNITS[unit]) }
}

/// Applies a download's outcome to the stored subscription. The entry is read again here rather than
/// reused from before the download, so a change made meanwhile is kept, and a subscription removed
/// meanwhile does not get its servers back as orphans nobody can delete.
fn record_refresh(store: &mut Store, id: &str, outcome: Result<Fetched>, now: u64) -> (Result<RefreshSummary>, Vec<Notice>) {
    let Some(mut current) = store.subscriptions().iter().find(|s| s.id == id).cloned() else {
        return (Err(AppError::Invalid("the subscription was removed while it was being refreshed".into())), Vec::new());
    };
    current.last_attempt = now;

    match outcome {
        Ok((profiles, user_info, skipped)) => {
            let total = profiles.len();
            let (added, removed) = match store.sync_subscription(id, profiles) {
                Ok(counts) => counts,
                Err(error) => return (Err(error), Vec::new()),
            };
            current.last_updated = now;
            current.last_error = None;
            current.failures = 0;
            current.user_info = user_info;
            let mut notices = Vec::new();
            notices.extend(expiry_pass(&mut current, now));
            if let Some(info) = current.user_info.clone() {
                let (due, stage) = due_data_notice(&info, current.notified_quota);
                current.notified_quota = stage;
                if let Some(stage) = due {
                    let left = info.total.saturating_sub(info.upload.saturating_add(info.download));
                    notices.push(Notice::Data { name: current.name.clone(), stage, left });
                }
            }
            if let Err(error) = store.update_subscription(current) {
                return (Err(error), Vec::new());
            }
            (Ok(RefreshSummary { added, removed, total, skipped: skipped.len(), skipped_reason: skipped.into_iter().next() }), notices)
        }
        Err(error) => {
            current.last_error = Some(error.to_string());
            current.failures = current.failures.saturating_add(1);
            let _ = store.update_subscription(current);
            (Err(error), Vec::new())
        }
    }
}

fn days_left(expire: u64, now: u64) -> String {
    match expire.saturating_sub(now).div_ceil(24 * 3600) {
        0 | 1 => "a day".to_string(),
        days => format!("{days} days"),
    }
}

/// Expiry is a matter of time, not of a download: check the stored dates on every scheduler tick,
/// so the warning still comes when refreshes are failing or far apart.
fn check_expiry(app: &AppHandle) {
    let state = app.state::<AppState>();
    let now = now_secs();
    let notices: Vec<Notice> = {
        let mut store = state.store.lock().unwrap();
        let all: Vec<Subscription> = store.subscriptions().to_vec();
        let mut out = Vec::new();
        for mut subscription in all {
            if let Some(notice) = expiry_pass(&mut subscription, now) {
                if store.update_subscription(subscription).is_ok() {
                    out.push(notice);
                }
            }
        }
        out
    };
    for notice in notices {
        notify::send(app, "Subscription", &notice.message(now));
    }
}

pub fn spawn_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            check_expiry(&app);
            let due: Vec<String> = {
                let state = app.state::<AppState>();
                let store = state.store.lock().unwrap();
                store
                    .subscriptions()
                    .iter()
                    .filter(|s| now_secs() >= next_due(s))
                    .map(|s| s.id.clone())
                    .collect()
            };
            for id in due {
                let _ = refresh(&app, &id).await;
            }
            sleep(SCHEDULER_TICK).await;
        }
    });
}

pub fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
#[path = "tests/subscriptions.rs"]
mod tests;
