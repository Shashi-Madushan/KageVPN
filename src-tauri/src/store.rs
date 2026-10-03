use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::Mode;
use crate::app_config::{core_spec, default_core, CoreSpec};
use crate::app_update::AppUpdateSettings;
use crate::core::{CoreSet, CoreSettings};
use crate::error::{AppError, Result};
use crate::fsutil;
use crate::profile::Profile;
use crate::settings::{AutoSettings, ReliabilitySettings, RoutingSettings, Subscription, UsageSettings};

const SCHEMA_VERSION: u32 = 3;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub show_speed: bool,
    pub auto_connect: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { show_speed: true, auto_connect: false }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Data {
    schema_version: u32,
    profiles: Vec<Profile>,
    active_id: Option<String>,
    mode: Mode,
    settings: Settings,
    /// Single-core layout from schema 2; read once and moved into `cores`.
    #[serde(skip_serializing_if = "Option::is_none")]
    core: Option<CoreSettings>,
    cores: CoreSet,
    app_update: AppUpdateSettings,
    usage: UsageSettings,
    routing: RoutingSettings,
    reliability: ReliabilitySettings,
    auto: AutoSettings,
    subscriptions: Vec<Subscription>,
}

pub struct Store {
    path: PathBuf,
    data: Data,
    /// Where an unreadable data file was moved so it can be recovered, if that happened at load.
    recovered: Option<PathBuf>,
}

impl Store {
    pub fn load(path: PathBuf) -> Self {
        let (data, recovered) = match fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice::<Data>(&bytes) {
                Ok(data) => (data, None),
                // Never start over silently: keep the unreadable file next to the new one.
                Err(_) => (Data::default(), quarantine(&path)),
            },
            Err(_) => (Data::default(), None),
        };
        let mut store = Self { path, data, recovered };
        store.migrate();
        store
    }

    /// The saved copy of a data file that could not be read at startup, once.
    pub fn take_recovery(&mut self) -> Option<PathBuf> {
        self.recovered.take()
    }

    /// Upgrades older files in place, keeping a copy of the file as it was.
    fn migrate(&mut self) {
        if self.data.schema_version >= SCHEMA_VERSION {
            return;
        }
        if self.path.is_file() {
            let backup = format!("data.v{}.json", self.data.schema_version.max(1));
            let _ = fs::copy(&self.path, self.path.with_file_name(backup));
        }
        if let Some(legacy) = self.data.core.take() {
            self.data.cores.entries.entry(default_core().id.to_string()).or_insert(legacy);
        }
        self.data.schema_version = SCHEMA_VERSION;
        let _ = self.save();
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.data.profiles
    }

    pub fn active_id(&self) -> Option<&str> {
        self.data.active_id.as_deref()
    }

    pub fn active_profile(&self) -> Option<&Profile> {
        self.active_id().and_then(|id| self.get(id))
    }

    pub fn get(&self, id: &str) -> Option<&Profile> {
        self.data.profiles.iter().find(|p| p.id == id)
    }

    pub fn mode(&self) -> Mode {
        self.data.mode
    }

    pub fn settings(&self) -> Settings {
        self.data.settings
    }

    /// The core that runs connections. An unknown saved id falls back to the default core.
    pub fn active_core(&self) -> &'static CoreSpec {
        core_spec(&self.data.cores.active).unwrap_or_else(default_core)
    }

    pub fn set_active_core(&mut self, id: &str) -> Result<()> {
        let spec = core_spec(id).ok_or_else(|| AppError::Invalid(format!("unknown core {id}")))?;
        self.data.cores.active = spec.id.to_string();
        self.save()
    }

    /// The active core's settings.
    pub fn core_settings(&self) -> CoreSettings {
        self.core_settings_for(self.active_core().id)
    }

    pub fn core_settings_for(&self, id: &str) -> CoreSettings {
        self.data.cores.entries.get(id).cloned().unwrap_or_default()
    }

    pub fn app_update(&self) -> AppUpdateSettings {
        self.data.app_update.clone()
    }

    pub fn set_app_update(&mut self, settings: AppUpdateSettings) -> Result<()> {
        self.data.app_update = settings;
        self.save()
    }

    pub fn usage_settings(&self) -> UsageSettings {
        self.data.usage.clone()
    }

    pub fn routing(&self) -> RoutingSettings {
        self.data.routing.clone()
    }

    pub fn reliability(&self) -> ReliabilitySettings {
        self.data.reliability.clone()
    }

    pub fn auto(&self) -> AutoSettings {
        self.data.auto.clone()
    }

    pub fn subscriptions(&self) -> &[Subscription] {
        &self.data.subscriptions
    }

    pub fn set_settings(&mut self, settings: Settings) -> Result<()> {
        self.data.settings = settings;
        self.save()
    }

    pub fn set_core_settings(&mut self, settings: CoreSettings) -> Result<()> {
        self.set_core_settings_for(self.active_core().id, settings)
    }

    pub fn set_core_settings_for(&mut self, id: &str, settings: CoreSettings) -> Result<()> {
        self.data.cores.entries.insert(id.to_string(), settings);
        self.save()
    }

    pub fn set_usage_settings(&mut self, settings: UsageSettings) -> Result<()> {
        self.data.usage = settings;
        self.save()
    }

    pub fn set_routing(&mut self, settings: RoutingSettings) -> Result<()> {
        self.data.routing = settings;
        self.save()
    }

    pub fn set_reliability(&mut self, settings: ReliabilitySettings) -> Result<()> {
        self.data.reliability = settings;
        self.save()
    }

    pub fn set_auto(&mut self, settings: AutoSettings) -> Result<()> {
        self.data.auto = settings;
        self.save()
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<()> {
        self.data.mode = mode;
        self.save()
    }

    pub fn add(&mut self, profile: Profile) -> Result<()> {
        self.add_many(vec![profile]).map(|_| ())
    }

    pub fn add_many(&mut self, profiles: Vec<Profile>) -> Result<usize> {
        let count = profiles.len();
        for profile in profiles {
            if self.data.active_id.is_none() {
                self.data.active_id = Some(profile.id.clone());
            }
            self.data.profiles.push(profile);
        }
        self.save()?;
        Ok(count)
    }

    /// Adds profiles that are not already saved (same server, port, credentials and transport).
    /// Returns how many were added and how many were skipped as duplicates.
    pub fn import_unique(&mut self, profiles: Vec<Profile>) -> Result<(usize, usize)> {
        let total = profiles.len();
        let mut fresh: Vec<Profile> = Vec::new();
        for profile in profiles {
            let known = self.data.profiles.iter().chain(fresh.iter()).any(|p| p.same_endpoint(&profile));
            if !known {
                fresh.push(profile);
            }
        }
        let added = fresh.len();
        if added > 0 {
            self.add_many(fresh)?;
        }
        Ok((added, total - added))
    }

    /// Replaces the profile with the same id, or appends it when new.
    pub fn upsert(&mut self, profile: Profile) -> Result<()> {
        match self.data.profiles.iter_mut().find(|p| p.id == profile.id) {
            Some(existing) => *existing = profile,
            None => return self.add(profile),
        }
        self.save()
    }

    pub fn remove(&mut self, id: &str) -> Result<()> {
        self.data.profiles.retain(|p| p.id != id);
        self.fix_active();
        self.save()
    }

    pub fn select(&mut self, id: &str) -> Result<()> {
        self.require(id)?;
        self.data.active_id = Some(id.to_string());
        self.save()
    }

    pub fn trust_key(&mut self, id: &str, key: String) -> Result<()> {
        self.edit(id, |p| p.pinned_key = Some(key))
    }

    pub fn edit(&mut self, id: &str, change: impl FnOnce(&mut Profile)) -> Result<()> {
        let profile = self.data.profiles.iter_mut().find(|p| p.id == id).ok_or(AppError::ProfileNotFound)?;
        change(profile);
        self.save()
    }

    pub fn reorder(&mut self, ids: &[String]) -> Result<()> {
        let mut ordered: Vec<Profile> = ids
            .iter()
            .filter_map(|id| self.data.profiles.iter().find(|p| p.id == *id).cloned())
            .collect();
        ordered.extend(self.data.profiles.iter().filter(|p| !ids.contains(&p.id)).cloned());
        self.data.profiles = ordered;
        self.save()
    }

    pub fn add_subscription(&mut self, subscription: Subscription) -> Result<()> {
        self.data.subscriptions.push(subscription);
        self.save()
    }

    pub fn update_subscription(&mut self, subscription: Subscription) -> Result<()> {
        if let Some(existing) = self.data.subscriptions.iter_mut().find(|s| s.id == subscription.id) {
            *existing = subscription;
        }
        self.save()
    }

    pub fn remove_subscription(&mut self, id: &str) -> Result<()> {
        self.data.subscriptions.retain(|s| s.id != id);
        self.data.profiles.retain(|p| p.subscription_id.as_deref() != Some(id));
        self.fix_active();
        self.save()
    }

    /// Syncs a subscription's profiles with a fresh list. Profiles are matched by
    /// server, port and UUID so ids, names, groups and favorites survive a refresh.
    pub fn sync_subscription(&mut self, subscription_id: &str, fresh: Vec<Profile>) -> Result<(usize, usize)> {
        let key = |p: &Profile| (p.server.clone(), p.port, p.uuid.clone());
        let (mut added, mut kept) = (0, Vec::new());

        for mut profile in fresh {
            profile.subscription_id = Some(subscription_id.to_string());
            match self.data.profiles.iter_mut().find(|p| p.subscription_id.as_deref() == Some(subscription_id) && key(p) == key(&profile)) {
                Some(existing) => {
                    let (id, name, group, favorite) =
                        (existing.id.clone(), existing.name.clone(), existing.group.clone(), existing.favorite);
                    profile.id = id;
                    profile.name = name;
                    profile.group = group;
                    profile.favorite = favorite;
                    *existing = profile.clone();
                }
                None => {
                    added += 1;
                    self.data.profiles.push(profile.clone());
                }
            }
            kept.push(key(&profile));
        }

        let before = self.data.profiles.len();
        self.data
            .profiles
            .retain(|p| p.subscription_id.as_deref() != Some(subscription_id) || kept.contains(&key(p)));
        let removed = before - self.data.profiles.len();
        self.fix_active();
        self.save()?;
        Ok((added, removed))
    }

    fn fix_active(&mut self) {
        let valid = self.data.active_id.as_deref().is_some_and(|id| self.data.profiles.iter().any(|p| p.id == id));
        if !valid {
            self.data.active_id = self.data.profiles.first().map(|p| p.id.clone());
        }
    }

    fn require(&self, id: &str) -> Result<()> {
        self.get(id).map(|_| ()).ok_or(AppError::ProfileNotFound)
    }

    fn save(&self) -> Result<()> {
        fsutil::write_private_atomic(&self.path, &serde_json::to_vec_pretty(&self.data)?)?;
        Ok(())
    }
}

/// Moves an unreadable data file aside as `data.corrupt.json` (or a numbered variant).
fn quarantine(path: &std::path::Path) -> Option<PathBuf> {
    let stem = path.file_stem()?.to_string_lossy().into_owned();
    let mut target = path.with_file_name(format!("{stem}.corrupt.json"));
    let mut n = 2;
    while target.exists() {
        target = path.with_file_name(format!("{stem}.corrupt-{n}.json"));
        n += 1;
    }
    fs::rename(path, &target).ok().map(|()| target)
}

#[cfg(test)]
#[path = "tests/store.rs"]
mod tests;
