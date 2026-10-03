use tauri::{AppHandle, Manager};

use super::releases::{self, Release};
use super::version::Version;
use super::{installer, Channel};
use crate::app_config::CoreSpec;
use crate::config::{self, ConfigInput, Mode};
use crate::downloads::{DownloadProvider, Kind, Reporter};
use crate::error::{AppError, Result};
use crate::session::State as Conn;
use crate::state::AppState;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Activation {
    /// Make it the active engine immediately.
    Now,
    /// Make it active now if idle, otherwise on the next connect.
    WhenIdle,
}

/// Downloads and installs one of the cores in `app_config::CORES`.
pub struct CoreProvider {
    pub spec: &'static CoreSpec,
    pub allow_unverified: bool,
    pub activation: Activation,
}

impl DownloadProvider for CoreProvider {
    type Release = Release;

    fn id(&self) -> String {
        format!("core:{}", self.spec.id)
    }

    fn kind(&self) -> Kind {
        Kind::Core
    }

    fn name(&self) -> String {
        self.spec.name.to_string()
    }

    fn version_of(&self, release: &Release) -> String {
        release.version.clone()
    }

    fn success_message(&self, version: &str) -> String {
        format!("Version {version} is installed.")
    }

    async fn resolve(&self, version: Option<&str>) -> Result<Release> {
        match version {
            Some(version) => find_release(self.spec, version).await,
            None => latest_stable(self.spec).await,
        }
    }

    async fn install(&self, app: &AppHandle, release: &Release, reporter: &Reporter) -> Result<()> {
        let spec = self.spec;
        let state = app.state::<AppState>();

        // Validate the user's current servers against the new binary before it is used.
        let check_config = {
            let store = state.store.lock().unwrap();
            let profiles = store.profiles().to_vec();
            let routing = store.routing();
            (!profiles.is_empty())
                .then(|| {
                    config::build(&ConfigInput {
                        profiles: &profiles,
                        active_id: store.active_id(),
                        auto: false,
                        mode: Mode::Proxy,
                        routing: &routing,
                        log_path: &state.paths.log_file(),
                        cache_path: &state.paths.cache_file(),
                        secret: "check",
                    })
                    .ok()
                })
                .flatten()
        };

        installer::install(reporter, &state.paths, spec, release, self.allow_unverified, check_config).await?;

        let idle = state.status.lock().unwrap().state == Conn::Disconnected;
        let mut store = state.store.lock().unwrap();
        let mut settings = store.core_settings_for(spec.id);
        if self.activation == Activation::Now || idle {
            settings.managed_version = Some(release.version.clone());
            settings.pending_version = None;
        } else {
            settings.pending_version = Some(release.version.clone());
        }
        store.set_core_settings_for(spec.id, settings)
    }
}

async fn find_release(spec: &CoreSpec, version: &str) -> Result<Release> {
    releases::fetch(spec, Channel::Prerelease)
        .await?
        .into_iter()
        .find(|r| r.version == version)
        .ok_or_else(|| AppError::Invalid(format!("version {version} was not found")))
}

async fn latest_stable(spec: &CoreSpec) -> Result<Release> {
    let found = releases::fetch(spec, Channel::Stable).await?;
    newest(found).ok_or_else(|| AppError::Invalid(format!("no stable {} release was found for this Mac", spec.name)))
}

pub(crate) fn newest(releases: Vec<Release>) -> Option<Release> {
    releases
        .into_iter()
        .filter_map(|r| Version::parse(&r.version).map(|v| (v, r)))
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, r)| r)
}
