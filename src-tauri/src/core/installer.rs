use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::read::GzDecoder;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};

use super::releases::Release;
use super::{inspect, Manifest, Source, MANIFEST_NAME};
use crate::app_config::CoreSpec;
use crate::error::{AppError, Result};
use crate::paths::AppPaths;
use crate::state::AppState;
use crate::singbox;

const PROGRESS_STEP: u64 = 256 * 1024;
const ARCHIVE_NAME: &str = "archive.tar.gz";

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Download,
    Verify,
    Extract,
    Check,
    Done,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub core: String,
    pub version: String,
    pub stage: Stage,
    pub received: u64,
    pub total: u64,
}

pub async fn install(
    app: &AppHandle,
    paths: &AppPaths,
    spec: &CoreSpec,
    release: &Release,
    allow_unverified: bool,
    check_config: Option<Value>,
) -> Result<()> {
    if release.sha256.is_none() && !allow_unverified {
        return Err(AppError::Invalid(
            "this release has no published checksum, confirm to install it anyway".into(),
        ));
    }

    let staging = paths.core_dir_for(spec.id).join(format!(".staging-{}", release.version));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging)?;

    let result = run(app, paths, spec, release, &staging, check_config).await;
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

async fn run(
    app: &AppHandle,
    paths: &AppPaths,
    spec: &CoreSpec,
    release: &Release,
    staging: &Path,
    check_config: Option<Value>,
) -> Result<()> {
    let archive = staging.join(ARCHIVE_NAME);
    let archive_sha256 = download(app, spec, release, &archive).await?;

    emit(app, spec, release, Stage::Verify, 0, 0);
    if let Some(expected) = &release.sha256 {
        if !expected.eq_ignore_ascii_case(&archive_sha256) {
            return Err(AppError::Launch("checksum mismatch, the download was rejected".into()));
        }
    }

    emit(app, spec, release, Stage::Extract, 0, 0);
    let binary = staging.join(spec.binary_name);
    let (archive_path, binary_path, name) = (archive.clone(), binary.clone(), spec.binary_name);
    tauri::async_runtime::spawn_blocking(move || extract_binary(name, &archive_path, &binary_path))
        .await
        .map_err(|e| AppError::Launch(e.to_string()))??;

    emit(app, spec, release, Stage::Check, 0, 0);
    verify_binary(spec, &binary, staging, check_config)?;

    let manifest = Manifest {
        version: release.version.clone(),
        archive_sha256,
        binary_sha256: sha256_file(&binary)?,
        installed_at: now_secs(),
    };
    fs::write(staging.join(MANIFEST_NAME), serde_json::to_vec_pretty(&manifest)?)?;
    fs::remove_file(&archive)?;

    let target = paths.core_dir_for(spec.id).join(&release.version);
    let _ = fs::remove_dir_all(&target);
    fs::rename(staging, &target)?;
    emit(app, spec, release, Stage::Done, 0, 0);
    Ok(())
}

async fn download(app: &AppHandle, spec: &CoreSpec, release: &Release, destination: &Path) -> Result<String> {
    let mut response = reqwest::Client::builder()
        .user_agent(crate::app_config::user_agent())
        .build()?
        .get(&release.url)
        .send()
        .await?
        .error_for_status()?;

    let mut file = File::create(destination)?;
    let mut hasher = Sha256::new();
    let (mut received, mut last_emit) = (0u64, 0u64);

    let cancel = &app.state::<AppState>().core_cancel;
    while let Some(chunk) = response.chunk().await? {
        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(AppError::Launch("download cancelled".into()));
        }
        hasher.update(&chunk);
        file.write_all(&chunk)?;
        received += chunk.len() as u64;
        if received - last_emit >= PROGRESS_STEP {
            last_emit = received;
            emit(app, spec, release, Stage::Download, received, release.size);
        }
    }
    emit(app, spec, release, Stage::Download, received, release.size);
    Ok(hex(&hasher.finalize()))
}

pub fn extract_binary(binary_name: &str, archive: &Path, destination: &Path) -> Result<()> {
    let mut tar = tar::Archive::new(GzDecoder::new(File::open(archive)?));
    for entry in tar.entries()? {
        let mut entry = entry?;
        let is_binary = entry.header().entry_type().is_file()
            && entry.path()?.file_name().is_some_and(|name| name == binary_name);
        if is_binary {
            let mut out = File::create(destination)?;
            std::io::copy(&mut entry, &mut out)?;
            set_executable(destination)?;
            return Ok(());
        }
    }
    Err(AppError::Launch(format!("the archive does not contain a {binary_name} binary")))
}

fn verify_binary(spec: &CoreSpec, binary: &Path, staging: &Path, check_config: Option<Value>) -> Result<()> {
    let info = inspect(spec, binary, Source::Managed)?;
    if !info.missing_tags.is_empty() {
        return Err(AppError::Launch(format!("missing build features: {}", info.missing_tags.join(", "))));
    }
    if info.outdated {
        return Err(AppError::Launch(format!("version {} is older than the supported minimum", info.version)));
    }
    // Validating a config is a sing-box subcommand, so only cores with a runtime adapter do it.
    if let Some(config) = check_config.filter(|_| spec.runnable) {
        let path = staging.join("check.json");
        fs::write(&path, serde_json::to_vec(&config)?)?;
        let result = singbox::check(binary, &path);
        let _ = fs::remove_file(&path);
        result?;
    }
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn emit(app: &AppHandle, spec: &CoreSpec, release: &Release, stage: Stage, received: u64, total: u64) {
    let progress = Progress { core: spec.name.to_string(), version: release.version.clone(), stage, received, total };
    let _ = app.emit("core-progress", progress);
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
#[path = "../tests/core_installer.rs"]
mod tests;
