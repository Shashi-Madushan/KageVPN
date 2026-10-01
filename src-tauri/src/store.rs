use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::Mode;
use crate::error::{AppError, Result};
use crate::profile::Profile;

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
    profiles: Vec<Profile>,
    active_id: Option<String>,
    mode: Mode,
    settings: Settings,
}

pub struct Store {
    path: PathBuf,
    data: Data,
}

impl Store {
    pub fn load(path: PathBuf) -> Self {
        let data = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self { path, data }
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

    pub fn set_settings(&mut self, settings: Settings) -> Result<()> {
        self.data.settings = settings;
        self.save()
    }

    pub fn rename(&mut self, id: &str, name: String) -> Result<()> {
        self.require(id)?;
        if let Some(profile) = self.data.profiles.iter_mut().find(|p| p.id == id) {
            profile.name = name;
        }
        self.save()
    }

    pub fn add(&mut self, profile: Profile) -> Result<()> {
        if self.data.active_id.is_none() {
            self.data.active_id = Some(profile.id.clone());
        }
        self.data.profiles.push(profile);
        self.save()
    }

    pub fn remove(&mut self, id: &str) -> Result<()> {
        self.data.profiles.retain(|p| p.id != id);
        if self.data.active_id.as_deref() == Some(id) {
            self.data.active_id = self.data.profiles.first().map(|p| p.id.clone());
        }
        self.save()
    }

    pub fn select(&mut self, id: &str) -> Result<()> {
        self.require(id)?;
        self.data.active_id = Some(id.to_string());
        self.save()
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<()> {
        self.data.mode = mode;
        self.save()
    }

    pub fn trust_key(&mut self, id: &str, key: String) -> Result<()> {
        self.require(id)?;
        if let Some(profile) = self.data.profiles.iter_mut().find(|p| p.id == id) {
            profile.pinned_key = Some(key);
        }
        self.save()
    }

    fn require(&self, id: &str) -> Result<()> {
        self.get(id).map(|_| ()).ok_or(AppError::ProfileNotFound)
    }

    fn save(&self) -> Result<()> {
        fs::write(&self.path, serde_json::to_vec_pretty(&self.data)?)?;
        Ok(())
    }
}
