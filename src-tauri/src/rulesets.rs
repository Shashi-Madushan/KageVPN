//! Country and category lists (`geoip-cn`, `geosite-category-ads-all`, ...) as local files.
//!
//! sing-box can download these itself, but it refuses to start when the first download fails, and
//! the networks where people need a VPN most are often the ones that block GitHub. So the app
//! fetches them, trying a mirror too, into its own folder and hands sing-box local files. A list
//! that cannot be fetched is left out of the configuration; it never stops a connection.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use crate::app_config::urls::{GEOIP_RULE_SETS, GEOIP_RULE_SETS_MIRROR, GEOSITE_RULE_SETS, GEOSITE_RULE_SETS_MIRROR};
use crate::config::rule_set_tag;
use crate::error::{AppError, Result};
use crate::settings::{RouteMode, RoutingSettings, RuleSetKind};

/// A list older than this is refreshed in the background.
pub const MAX_AGE: Duration = Duration::from_secs(7 * 24 * 3600);
/// A list that could not be downloaded is not tried again for this long when connecting. Without
/// it every reconnect attempt would wait on the same blocked download and repeat the notification.
const RETRY_AFTER: Duration = Duration::from_secs(90);
const MAX_BYTES: usize = 20 * 1024 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const SOURCE_TIMEOUT: Duration = Duration::from_secs(8);

/// One list the current routing settings use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wanted {
    pub tag: String,
    pub kind: RuleSetKind,
    /// The country or category part of the tag, such as `cn`.
    pub name: String,
}

impl Wanted {
    pub fn file_name(&self) -> String {
        format!("{}.srs", self.tag)
    }

    pub fn path(&self, dir: &Path) -> PathBuf {
        dir.join(self.file_name())
    }
}

/// The lists the routing settings turn on. Only Rules mode uses them; duplicates count once.
pub fn wanted(routing: &RoutingSettings) -> Vec<Wanted> {
    if routing.mode != RouteMode::Rule {
        return Vec::new();
    }
    let mut found: Vec<Wanted> = Vec::new();
    for set in routing.rule_sets.iter().filter(|s| s.enabled) {
        let Some(tag) = rule_set_tag(set.kind, &set.country) else { continue };
        if found.iter().all(|w| w.tag != tag) {
            found.push(Wanted { tag, kind: set.kind, name: set.country.trim().to_lowercase() });
        }
    }
    found
}

/// Where to fetch a list, best first: GitHub, then a CDN copy.
pub fn sources(item: &Wanted) -> Vec<String> {
    let (primary, mirror) = match item.kind {
        RuleSetKind::Geoip => (GEOIP_RULE_SETS, GEOIP_RULE_SETS_MIRROR),
        RuleSetKind::Geosite => (GEOSITE_RULE_SETS, GEOSITE_RULE_SETS_MIRROR),
    };
    vec![format!("{primary}{}.srs", item.name), format!("{mirror}{}.srs", item.name)]
}

/// Splits `wanted` into lists with no file yet (needed before connecting) and lists with an old
/// file (worth refreshing, but usable as they are).
pub fn plan(dir: &Path, wanted: &[Wanted], max_age: Duration) -> (Vec<Wanted>, Vec<Wanted>) {
    let (mut missing, mut stale) = (Vec::new(), Vec::new());
    for item in wanted {
        match std::fs::metadata(item.path(dir)) {
            Ok(meta) if meta.is_file() && meta.len() > 0 => {
                let age = meta.modified().ok().and_then(|t| SystemTime::now().duration_since(t).ok()).unwrap_or_default();
                if age > max_age {
                    stale.push(item.clone());
                }
            }
            _ => missing.push(item.clone()),
        }
    }
    (missing, stale)
}

/// Whether bytes look like a compiled sing-box rule-set. A captive portal or an error page saved
/// under the same name would otherwise stop sing-box from starting.
pub fn is_rule_set(bytes: &[u8]) -> bool {
    bytes.len() > 4 && bytes.starts_with(b"SRS")
}

async fn fetch_one(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    let bytes = client.get(url).send().await?.error_for_status()?.bytes().await?;
    if bytes.len() > MAX_BYTES || !is_rule_set(&bytes) {
        return Err(AppError::Invalid(format!("{url} did not return a rule-set file")));
    }
    Ok(bytes.to_vec())
}

/// Downloads one list from the first source that answers and replaces the file atomically, so a
/// failed or partial download never damages the copy already there.
pub async fn download(dir: &Path, item: &Wanted, sources: &[String]) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(crate::app_config::user_agent())
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(SOURCE_TIMEOUT)
        .build()?;
    let mut last = AppError::Launch("no address to download the list from".into());
    for url in sources {
        match fetch_one(&client, url).await {
            Ok(bytes) => {
                std::fs::create_dir_all(dir)?;
                crate::fsutil::write_private_atomic(&item.path(dir), &bytes)?;
                return Ok(());
            }
            Err(error) => last = error,
        }
    }
    Err(last)
}

/// Remembers which lists failed to download and when.
#[derive(Default)]
struct Cooldown(HashMap<String, Instant>);

impl Cooldown {
    fn failed(&mut self, tag: &str, now: Instant) {
        self.0.insert(tag.to_string(), now);
    }

    fn succeeded(&mut self, tag: &str) {
        self.0.remove(tag);
    }

    fn cooling(&self, tag: &str, now: Instant) -> bool {
        self.0.get(tag).is_some_and(|at| now.saturating_duration_since(*at) < RETRY_AFTER)
    }
}

static COOLDOWN: Mutex<Option<Cooldown>> = Mutex::new(None);

fn with_cooldown<T>(action: impl FnOnce(&mut Cooldown) -> T) -> T {
    let mut guard = COOLDOWN.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    action(guard.get_or_insert_with(Cooldown::default))
}

/// Drops the lists that failed to download a moment ago, for callers that retry on a loop.
pub fn skip_recent_failures(items: Vec<Wanted>) -> Vec<Wanted> {
    let now = Instant::now();
    with_cooldown(|cooldown| items.into_iter().filter(|item| !cooldown.cooling(&item.tag, now)).collect())
}

/// Downloads every list in `items` at once, returning the ones that could not be fetched.
pub async fn fetch_all(dir: &Path, items: Vec<Wanted>) -> Vec<(Wanted, AppError)> {
    fetch_all_from(dir, items, Arc::new(sources)).await
}

type SourcesOf = Arc<dyn Fn(&Wanted) -> Vec<String> + Send + Sync>;

async fn fetch_all_from(dir: &Path, items: Vec<Wanted>, sources_of: SourcesOf) -> Vec<(Wanted, AppError)> {
    let handles: Vec<_> = items
        .into_iter()
        .map(|item| {
            let dir = dir.to_path_buf();
            let sources_of = sources_of.clone();
            tauri::async_runtime::spawn(async move {
                let result = download(&dir, &item, &sources_of(&item)).await;
                (item, result)
            })
        })
        .collect();
    let mut failed = Vec::new();
    for handle in handles {
        match handle.await {
            Ok((item, Ok(()))) => with_cooldown(|cooldown| cooldown.succeeded(&item.tag)),
            Ok((item, Err(error))) => {
                with_cooldown(|cooldown| cooldown.failed(&item.tag, Instant::now()));
                failed.push((item, error));
            }
            Err(_) => {}
        }
    }
    failed
}

/// Plain words for a notification about lists that were left out.
pub fn skipped_message(tags: &[String]) -> String {
    format!(
        "{} could not be downloaded, so {} not applied. Kage VPN tries again the next time you connect.",
        tags.join(", "),
        if tags.len() == 1 { "it is" } else { "they are" }
    )
}

#[cfg(test)]
#[path = "tests/rulesets.rs"]
mod tests;
