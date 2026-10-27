use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::read::GzDecoder;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::releases::Release;
use super::version::Version;
use super::{inspect, Manifest, Source, MANIFEST_NAME};
use crate::app_config::CoreSpec;
use crate::error::{AppError, Result};
use crate::paths::AppPaths;
use crate::downloads::{read_body, Reporter, Stage, STALL_AFTER};
use crate::singbox;

const PROGRESS_STEP: u64 = 256 * 1024;
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
const ARCHIVE_NAME: &str = "archive.tar.gz";
/// A sing-box archive is about 30 MB. These limits stop a damaged or hostile download from filling
/// the disk, and a small archive that unpacks into something huge (a decompression bomb) from doing
/// the same through the extraction.
const MAX_ARCHIVE_BYTES: u64 = 300 * 1024 * 1024;
#[cfg(not(test))]
const MAX_BINARY_BYTES: u64 = 1024 * 1024 * 1024;
/// Small enough for a test to build an archive that exceeds it.
#[cfg(test)]
const MAX_BINARY_BYTES: u64 = 1024 * 1024;

pub async fn install(
    reporter: &Reporter,
    paths: &AppPaths,
    spec: &CoreSpec,
    release: &Release,
    allow_unverified: bool,
    check_config: Option<Value>,
) -> Result<()> {
    check_release(release)?;
    if release.sha256.is_none() && !allow_unverified {
        return Err(AppError::Invalid(
            "this release has no published checksum, confirm to install it anyway".into(),
        ));
    }

    let staging = paths.core_dir_for(spec.id).join(format!(".staging-{}", release.version));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging)?;

    let result = run(reporter, paths, spec, release, &staging, check_config).await;
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

/// Refuses a release whose version could name a folder outside the engine folder, or whose download
/// address is not https. Both come from a web service, so they are checked before they are used.
fn check_release(release: &Release) -> Result<()> {
    if Version::parse(&release.version).is_none() {
        return Err(AppError::Invalid("the release has a version this app cannot use".into()));
    }
    if !release.url.starts_with("https://") {
        return Err(AppError::Invalid("the release is not offered over https".into()));
    }
    Ok(())
}

async fn run(
    reporter: &Reporter,
    paths: &AppPaths,
    spec: &CoreSpec,
    release: &Release,
    staging: &Path,
    check_config: Option<Value>,
) -> Result<()> {
    let archive = staging.join(ARCHIVE_NAME);
    let archive_sha256 = download(reporter, release, &archive).await?;

    reporter.report(Stage::Verify, 0, 0);
    if let Some(expected) = &release.sha256 {
        if !expected.eq_ignore_ascii_case(&archive_sha256) {
            return Err(AppError::Launch("checksum mismatch, the download was rejected".into()));
        }
    }

    reporter.report(Stage::Extract, 0, 0);
    let binary = staging.join(spec.binary_name);
    let (archive_path, binary_path, name) = (archive.clone(), binary.clone(), spec.binary_name);
    tauri::async_runtime::spawn_blocking(move || extract_binary(name, &archive_path, &binary_path))
        .await
        .map_err(|e| AppError::Launch(e.to_string()))??;

    reporter.report(Stage::Check, 0, 0);
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
    reporter.report(Stage::Done, 0, 0);
    Ok(())
}

async fn download(reporter: &Reporter, release: &Release, destination: &Path) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent(crate::app_config::user_agent())
        .connect_timeout(CONNECT_TIMEOUT)
        .redirect(crate::downloads::https_only_redirects())
        .build()?;
    // Connecting is bounded above; this bounds the wait for the server's first answer.
    let response = tokio::time::timeout(STALL_AFTER, client.get(&release.url).send())
        .await
        .map_err(|_| AppError::Launch("the download server did not respond, try again later".into()))??
        .error_for_status()?;
    if response.content_length().is_some_and(|promised| promised > MAX_ARCHIVE_BYTES) {
        return Err(AppError::Launch("the download is larger than an engine archive can be, it was rejected".into()));
    }

    let mut file = File::create(destination)?;
    let mut hasher = Sha256::new();
    let (mut received, mut last_emit) = (0u64, 0u64);

    read_body(response, STALL_AFTER, || reporter.cancelled(), |chunk| {
        hasher.update(chunk);
        file.write_all(chunk)?;
        received += chunk.len() as u64;
        if received > MAX_ARCHIVE_BYTES {
            return Err(AppError::Launch("the download is larger than an engine archive can be, it was rejected".into()));
        }
        if received - last_emit >= PROGRESS_STEP {
            last_emit = received;
            reporter.report(Stage::Download, received, release.size);
        }
        Ok(())
    })
    .await?;
    reporter.report(Stage::Download, received, release.size);
    Ok(hex(&hasher.finalize()))
}

pub fn extract_binary(binary_name: &str, archive: &Path, destination: &Path) -> Result<()> {
    let mut tar = tar::Archive::new(GzDecoder::new(File::open(archive)?));
    for entry in tar.entries()? {
        let mut entry = entry?;
        let is_binary = entry.header().entry_type().is_file()
            && entry.path()?.file_name().is_some_and(|name| name == binary_name);
        if is_binary {
            if entry.size() > MAX_BINARY_BYTES {
                return Err(AppError::Launch(format!("the {binary_name} binary in the archive is larger than expected")));
            }
            let mut out = File::create(destination)?;
            // The header's size is only a claim; the copy itself is capped too.
            let copied = std::io::copy(&mut (&mut entry).take(MAX_BINARY_BYTES + 1), &mut out)?;
            if copied > MAX_BINARY_BYTES {
                let _ = fs::remove_file(destination);
                return Err(AppError::Launch(format!("the {binary_name} binary in the archive is larger than expected")));
            }
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

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
#[path = "../tests/core_installer.rs"]
mod tests;
