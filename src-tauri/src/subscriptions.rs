use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio::time::sleep;

use crate::error::{AppError, Result};
use crate::notify;
use crate::profile::{parse_many, Profile};
use crate::settings::UserInfo;
use crate::state::AppState;

const MAX_BODY_BYTES: usize = 5 * 1024 * 1024;
const SCHEDULER_TICK: Duration = Duration::from_secs(30 * 60);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshSummary {
    pub added: usize,
    pub removed: usize,
    pub total: usize,
    pub skipped: usize,
}

pub fn validate_url(url: &str) -> Result<()> {
    let parsed = url::Url::parse(url.trim()).map_err(|_| AppError::Invalid("not a valid URL".into()))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::Invalid("subscription links must start with http:// or https://".into()));
    }
    Ok(())
}

async fn fetch(url: &str) -> Result<(Vec<Profile>, Option<UserInfo>, usize)> {
    validate_url(url)?;
    let response = reqwest::Client::builder()
        .user_agent(concat!("singbox-gui/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(20))
        .build()?
        .get(url.trim())
        .send()
        .await?
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
    let (profiles, errors) = parse_many(&text);
    if profiles.is_empty() {
        return Err(AppError::Invalid("no VLESS links were found in this subscription".into()));
    }
    Ok((profiles, user_info, errors.len()))
}

/// Subscriptions are either a plain list of links or the same list base64-encoded.
fn decode_body(text: &str) -> String {
    if text.contains("vless://") {
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
    let subscription = state
        .store
        .lock()
        .unwrap()
        .subscriptions()
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| AppError::Invalid("subscription not found".into()))?;

    let outcome = fetch(&subscription.url).await;
    let mut store = state.store.lock().unwrap();
    let mut updated = subscription.clone();

    match outcome {
        Ok((profiles, user_info, skipped)) => {
            let total = profiles.len();
            let (added, removed) = store.sync_subscription(id, profiles)?;
            updated.last_updated = now_secs();
            updated.last_error = None;
            updated.user_info = user_info;
            store.update_subscription(updated.clone())?;
            drop(store);
            if updated.user_info.as_ref().is_some_and(|u| u.expire > 0 && u.expire < now_secs()) {
                notify::send(app, "Subscription expired", &format!("{} has expired.", updated.name));
            }
            Ok(RefreshSummary { added, removed, total, skipped })
        }
        Err(error) => {
            updated.last_error = Some(error.to_string());
            let _ = store.update_subscription(updated);
            Err(error)
        }
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
                    .filter(|s| now_secs().saturating_sub(s.last_updated) >= u64::from(s.interval_hours.max(1)) * 3600)
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
