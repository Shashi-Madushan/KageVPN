use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::Mode;
use crate::core::CoreSettings;
use crate::error::{AppError, Result};
use crate::profile::Profile;
use crate::settings::{AutoSettings, ReliabilitySettings, RoutingSettings, Subscription, UsageSettings};

const SCHEMA_VERSION: u32 = 2;

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
    core: CoreSettings,
    usage: UsageSettings,
    routing: RoutingSettings,
    reliability: ReliabilitySettings,
    auto: AutoSettings,
    subscriptions: Vec<Subscription>,
}

pub struct Store {
    path: PathBuf,
    data: Data,
}

impl Store {
    pub fn load(path: PathBuf) -> Self {
        let data: Data = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        let mut store = Self { path, data };
        store.migrate();
        store
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

    pub fn core_settings(&self) -> CoreSettings {
        self.data.core.clone()
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
        self.data.core = settings;
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
        fs::write(&self.path, serde_json::to_vec_pretty(&self.data)?)?;
        Ok(())
    }
}
