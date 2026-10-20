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

use serde::Serialize;

use crate::app_config::urls::{GEOIP_RULE_SETS, GEOSITE_RULE_SETS};
use crate::config::rule_set_tag;
use crate::error::{AppError, Result};
use crate::settings::{RouteMode, RoutingSettings, RuleProvider, RuleSetKind};

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
    enabled_lists(routing)
}

/// Every enabled list, whatever the routing mode, so the manager can show and fetch them before
/// the user switches to Rules mode.
pub fn enabled_lists(routing: &RoutingSettings) -> Vec<Wanted> {
    let mut found: Vec<Wanted> = Vec::new();
    for set in routing.rule_sets.iter().filter(|s| s.enabled) {
        let Some(tag) = rule_set_tag(set.kind, &set.country) else { continue };
        if found.iter().all(|w| w.tag != tag) {
            found.push(Wanted { tag, kind: set.kind, name: set.country.trim().to_lowercase() });
        }
    }
    found
}

pub const DEFAULT_PROVIDER: &str = "sagernet";

fn builtin_provider(id: &str, name: &str, geoip: String, geosite: String, note: &str) -> RuleProvider {
    RuleProvider { id: id.into(), name: name.into(), geoip, geosite, note: note.into() }
}

/// The sources that come with the app. All of them publish sing-box `.srs` files; the V2Ray
/// `.dat` files some projects (Loyalsoldier's `v2ray-rules-dat`) are known for cannot be read by
/// sing-box, so only their sing-box-format repositories are listed.
pub fn builtin_providers() -> Vec<RuleProvider> {
    let raw = "https://raw.githubusercontent.com";
    vec![
        builtin_provider(
            DEFAULT_PROVIDER,
            "SagerNet",
            format!("{GEOIP_RULE_SETS}{{name}}.srs"),
            format!("{GEOSITE_RULE_SETS}{{name}}.srs"),
            "The sing-box authors' own lists: countries and V2Ray's domain categories.",
        ),
        builtin_provider(
            "metacubex",
            "MetaCubeX",
            format!("{raw}/MetaCubeX/meta-rules-dat/sing/geo/geoip/{{name}}.srs"),
            format!("{raw}/MetaCubeX/meta-rules-dat/sing/geo/geosite/{{name}}.srs"),
            "meta-rules-dat, the lists Clash Meta uses. Many more categories than SagerNet.",
        ),
        builtin_provider(
            "v2rayn",
            "v2rayN",
            format!("{raw}/2dust/sing-box-rules/rule-set-geoip/geoip-{{name}}.srs"),
            format!("{raw}/2dust/sing-box-rules/rule-set-geosite/geosite-{{name}}.srs"),
            "The lists v2rayN downloads for sing-box.",
        ),
        builtin_provider(
            "loyalsoldier",
            "Loyalsoldier (IP lists)",
            format!("{raw}/Loyalsoldier/geoip/release/srs/{{name}}.srs"),
            String::new(),
            "Loyalsoldier's enhanced country IP lists. Its site lists are V2Ray .dat files, which sing-box cannot read, so there are none here.",
        ),
        builtin_provider(
            "runetfreedom",
            "runetfreedom (Russia)",
            format!("{raw}/runetfreedom/russia-v2ray-rules-dat/release/sing-box/rule-set-geoip/geoip-{{name}}.srs"),
            format!("{raw}/runetfreedom/russia-v2ray-rules-dat/release/sing-box/rule-set-geosite/geosite-{{name}}.srs"),
            "Lists for Russia, including blocked-site categories (for example geosite ru-blocked).",
        ),
    ]
}

/// The provider the routing settings select, a built-in or a custom one. An unknown id (a custom
/// provider that was removed) falls back to the default.
pub fn provider_of(routing: &RoutingSettings) -> RuleProvider {
    let builtin = builtin_providers();
    builtin
        .iter()
        .chain(routing.custom_providers.iter())
        .find(|p| p.id == routing.rule_provider)
        .cloned()
        .unwrap_or_else(|| builtin.into_iter().next().unwrap())
}

/// Refuses custom providers that cannot work, so a typo shows up when saving, not as a list that
/// silently never downloads.
pub fn validate_providers(providers: &[RuleProvider]) -> Result<()> {
    if providers.len() > 20 {
        return Err(AppError::Invalid("there are too many list sources; remove some".into()));
    }
    let builtin = builtin_providers();
    for (i, provider) in providers.iter().enumerate() {
        let name = provider.name.trim();
        if provider.id.trim().is_empty() || name.is_empty() {
            return Err(AppError::Invalid("a list source needs a name".into()));
        }
        if builtin.iter().any(|b| b.id == provider.id) || providers[..i].iter().any(|p| p.id == provider.id) {
            return Err(AppError::Invalid(format!("the list source {name} is a duplicate")));
        }
        if provider.geoip.trim().is_empty() && provider.geosite.trim().is_empty() {
            return Err(AppError::Invalid(format!("{name} needs an address for IP lists, site lists or both")));
        }
        for (label, template) in [("IP list", &provider.geoip), ("site list", &provider.geosite)] {
            let template = template.trim();
            if template.is_empty() {
                continue;
            }
            if !template.starts_with("https://") || !template.contains("{name}") || template.contains(char::is_whitespace) {
                return Err(AppError::Invalid(format!(
                    "{name}: the {label} address must be a link starting with https:// and containing {{name}}, for example https://example.com/geoip-{{name}}.srs"
                )));
            }
        }
    }
    Ok(())
}

/// The same file through jsDelivr, for a GitHub raw address: `raw.githubusercontent.com/o/r/branch/path`
/// becomes `cdn.jsdelivr.net/gh/o/r@branch/path`. Other addresses have no mirror.
pub fn jsdelivr_mirror(url: &str) -> Option<String> {
    let rest = url.strip_prefix("https://raw.githubusercontent.com/")?;
    let mut parts = rest.splitn(4, '/');
    let (owner, repo, branch, path) = (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
    if [owner, repo, branch, path].iter().any(|p| p.is_empty()) {
        return None;
    }
    Some(format!("https://cdn.jsdelivr.net/gh/{owner}/{repo}@{branch}/{path}"))
}

/// Where to fetch a list, best first: the provider's own address, then a CDN copy when there is one.
/// Empty when the provider has no lists of that kind.
pub fn sources(item: &Wanted, provider: &RuleProvider) -> Vec<String> {
    let template = match item.kind {
        RuleSetKind::Geoip => &provider.geoip,
        RuleSetKind::Geosite => &provider.geosite,
    };
    if template.trim().is_empty() {
        return Vec::new();
    }
    let url = template.trim().replace("{name}", &item.name);
    let mut urls = vec![url.clone()];
    urls.extend(jsdelivr_mirror(&url));
    urls
}

/// How a batch of downloads is made: from which provider, and through which local proxy.
#[derive(Clone)]
pub struct Fetch {
    pub provider: RuleProvider,
    /// The tunnel's local proxy while connected. Tried first, so a network that blocks GitHub
    /// does not matter once the VPN is up; direct access is the fallback.
    pub proxy: Option<String>,
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
/// failed or partial download never damages the copy already there. With a `proxy` every source is
/// tried through it first, then directly.
pub async fn download(dir: &Path, item: &Wanted, sources: &[String], proxy: Option<&str>) -> Result<()> {
    let builder = || {
        reqwest::Client::builder().user_agent(crate::app_config::user_agent()).connect_timeout(CONNECT_TIMEOUT).timeout(SOURCE_TIMEOUT)
    };
    let mut clients = Vec::new();
    if let Some(proxy) = proxy {
        clients.push(builder().proxy(reqwest::Proxy::all(proxy)?).build()?);
    }
    clients.push(builder().no_proxy().build()?);

    let mut last = AppError::Invalid("this list source has no address for that kind of list".into());
    for client in &clients {
        for url in sources {
            match fetch_one(client, url).await {
                Ok(bytes) => {
                    std::fs::create_dir_all(dir)?;
                    crate::fsutil::write_private_atomic(&item.path(dir), &bytes)?;
                    return Ok(());
                }
                Err(error) => last = error,
            }
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
pub async fn fetch_all(dir: &Path, items: Vec<Wanted>, fetch: &Fetch) -> Vec<(Wanted, AppError)> {
    let provider = fetch.provider.clone();
    fetch_all_from(dir, items, Arc::new(move |item: &Wanted| sources(item, &provider)), fetch.proxy.clone()).await
}

type SourcesOf = Arc<dyn Fn(&Wanted) -> Vec<String> + Send + Sync>;

async fn fetch_all_from(dir: &Path, items: Vec<Wanted>, sources_of: SourcesOf, proxy: Option<String>) -> Vec<(Wanted, AppError)> {
    let handles: Vec<_> = items
        .into_iter()
        .map(|item| {
            let dir = dir.to_path_buf();
            let sources_of = sources_of.clone();
            let proxy = proxy.clone();
            tauri::async_runtime::spawn(async move {
                let result = download(&dir, &item, &sources_of(&item), proxy.as_deref()).await;
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

const PROVIDER_MARKER: &str = "provider.txt";

/// The id of the provider the files on disk came from, if known.
pub fn downloaded_from(dir: &Path) -> Option<String> {
    std::fs::read_to_string(dir.join(PROVIDER_MARKER)).ok().map(|t| t.trim().to_string()).filter(|t| !t.is_empty())
}

pub fn remember_provider(dir: &Path, id: &str) {
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::write(dir.join(PROVIDER_MARKER), id);
}

/// One list as the manager shows it.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListStatus {
    pub tag: String,
    pub kind: RuleSetKind,
    pub name: String,
    pub present: bool,
    pub bytes: u64,
    /// When the file was last written, in seconds since the epoch.
    pub modified: Option<u64>,
}

/// What is on disk for each enabled list.
pub fn status(dir: &Path, routing: &RoutingSettings) -> Vec<ListStatus> {
    enabled_lists(routing)
        .into_iter()
        .map(|item| {
            let meta = std::fs::metadata(item.path(dir)).ok().filter(|m| m.is_file() && m.len() > 0);
            ListStatus {
                modified: meta.as_ref().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
                present: meta.is_some(),
                bytes: meta.map_or(0, |m| m.len()),
                tag: item.tag,
                kind: item.kind,
                name: item.name,
            }
        })
        .collect()
}

/// Deletes the downloaded file of one list, or of every list. Only `.srs` files this module
/// writes (and the provider marker) are touched.
pub fn remove(dir: &Path, tag: Option<&str>) -> Result<()> {
    match tag {
        Some(tag) => {
            let valid = !tag.is_empty() && tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
            if !valid {
                return Err(AppError::Invalid("that is not a list name".into()));
            }
            let _ = std::fs::remove_file(dir.join(format!("{tag}.srs")));
        }
        None => {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "srs") {
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
            let _ = std::fs::remove_file(dir.join(PROVIDER_MARKER));
        }
    }
    Ok(())
}

/// Plain words for a notification about lists that were left out.
pub fn skipped_message(tags: &[String]) -> String {
    format!(
        "{} could not be downloaded, so {} not applied. Kage tries again the next time you connect.",
        tags.join(", "),
        if tags.len() == 1 { "it is" } else { "they are" }
    )
}

#[cfg(test)]
#[path = "tests/rulesets.rs"]
mod tests;
