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
/// A subscription ending within this long counts as "expires soon".
const SOON_SECS: u64 = 3 * 24 * 3600;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expiry {
    Soon,
    Expired,
}

/// Which expiry notice is due now, if any. `announced` is the `(date, stage)` last shown, so each
/// stage is announced once per expiry date; renewing the subscription gives a new date and starts over.
pub fn due_expiry_notice(expire: u64, now: u64, announced: (u64, u8)) -> Option<Expiry> {
    if expire == 0 {
        return None;
    }
    let stage = if expire <= now {
        2
    } else if expire - now <= SOON_SECS {
        1
    } else {
        0
    };
    let seen = if announced.0 == expire { announced.1 } else { 0 };
    (stage > seen).then_some(if stage == 2 { Expiry::Expired } else { Expiry::Soon })
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
        return error.into();
    };
    AppError::ServerUnreachable(format!(
        "The subscription server {why}. If your provider is blocked on this network, connect to a server first and refresh again."
    ))
}

async fn request(url: &str, through_tunnel: bool) -> std::result::Result<reqwest::Response, reqwest::Error> {
    let mut client = reqwest::Client::builder().user_agent(crate::app_config::user_agent()).timeout(Duration::from_secs(20));
    if through_tunnel {
        client = client.proxy(reqwest::Proxy::all(format!("socks5h://127.0.0.1:{}", crate::config::MIXED_PORT))?);
    }
    client.build()?.get(url.trim()).send().await
}

async fn fetch(url: &str) -> Result<Fetched> {
    validate_url(url)?;
    // Providers are often blocked on the very networks where people need a VPN. If the direct
    // request cannot connect and a tunnel is up, ask through it (Proxy Only mode would otherwise
    // fetch directly even while connected).
    let response = match request(url, false).await {
        Ok(response) => response,
        Err(error) if error.is_connect() || error.is_timeout() => {
            if crate::net::proxy_port_open().await {
                request(url, true).await.map_err(explain)?
            } else {
                return Err(explain(error));
            }
        }
        Err(error) => return Err(error.into()),
    }
    .error_for_status()?;

    let user_info = response
        .headers()
        .get("subscription-userinfo")
        .and_then(|v| v.to_str().ok())
        .map(parse_user_info);
    let body = response.bytes().await?;
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
    let url = state
        .store
        .lock()
        .unwrap()
        .subscriptions()
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.url.clone())
        .ok_or_else(|| AppError::Invalid("subscription not found".into()))?;

    // No lock is held while the download runs; the result is applied to the entry as it is now.
    let outcome = fetch(&url).await;
    let (result, notice) = record_refresh(&mut state.store.lock().unwrap(), id, outcome, now_secs());
    if let Some((name, expire, notice)) = notice {
        let body = match notice {
            Expiry::Expired => format!("{name} has expired."),
            Expiry::Soon => format!("{name} expires in {}.", days_left(expire, now_secs())),
        };
        notify::send(app, "Subscription", &body);
    }
    result
}

/// An expiry notice to show: the subscription's name, its expiry date and which notice is due.
type ExpiryNotice = (String, u64, Expiry);

/// Applies a download's outcome to the stored subscription. The entry is read again here rather than
/// reused from before the download, so a change made meanwhile is kept, and a subscription removed
/// meanwhile does not get its servers back as orphans nobody can delete.
fn record_refresh(store: &mut Store, id: &str, outcome: Result<Fetched>, now: u64) -> (Result<RefreshSummary>, Option<ExpiryNotice>) {
    let Some(mut current) = store.subscriptions().iter().find(|s| s.id == id).cloned() else {
        return (Err(AppError::Invalid("the subscription was removed while it was being refreshed".into())), None);
    };
    current.last_attempt = now;

    match outcome {
        Ok((profiles, user_info, skipped)) => {
            let total = profiles.len();
            let (added, removed) = match store.sync_subscription(id, profiles) {
                Ok(counts) => counts,
                Err(error) => return (Err(error), None),
            };
            current.last_updated = now;
            current.last_error = None;
            current.failures = 0;
            current.user_info = user_info;
            let notice = current
                .user_info
                .as_ref()
                .and_then(|u| due_expiry_notice(u.expire, now, (current.notified_expire, current.notified_stage)).map(|n| (current.name.clone(), u.expire, n)));
            if let Some((_, expire, notice)) = &notice {
                current.notified_expire = *expire;
                current.notified_stage = if *notice == Expiry::Expired { 2 } else { 1 };
            }
            if let Err(error) = store.update_subscription(current) {
                return (Err(error), None);
            }
            (Ok(RefreshSummary { added, removed, total, skipped: skipped.len(), skipped_reason: skipped.into_iter().next() }), notice)
        }
        Err(error) => {
            current.last_error = Some(error.to_string());
            current.failures = current.failures.saturating_add(1);
            let _ = store.update_subscription(current);
            (Err(error), None)
        }
    }
}

fn days_left(expire: u64, now: u64) -> String {
    match expire.saturating_sub(now).div_ceil(24 * 3600) {
        0 | 1 => "a day".to_string(),
        days => format!("{days} days"),
    }
}

pub fn spawn_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
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
