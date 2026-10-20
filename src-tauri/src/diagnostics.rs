use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::error::{AppError, Result};

use crate::app_config::urls::{CONNECTION_PROBE as PING_URL, DNS_LEAK as DNS_URL, IP_DETAILS as IP_URL, SPEED_DOWNLOAD as DOWN_URL, SPEED_UPLOAD as UP_URL};

/// Each of the download and upload phases is time-boxed so slow and fast links both finish quickly.
const PHASE_SECONDS: f64 = 8.0;
const DOWN_STREAMS: usize = 4;
const UP_STREAMS: usize = 3;
/// The HTTP status of the last request the speed test server refused, or 0.
static REFUSED_STATUS: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);

const UP_CHUNK: usize = 192 * 1024;
const PING_SAMPLES: usize = 6;
const SAMPLE_EVERY: Duration = Duration::from_millis(150);
/// How quickly the live reading follows the instantaneous rate (higher is twitchier).
const LIVE_SMOOTHING: f64 = 0.35;

static CANCELLED: AtomicBool = AtomicBool::new(false);

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawIp {
    ip: String,
    country: String,
    city: String,
    connection: RawConnection,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawConnection {
    org: String,
}

#[derive(Serialize, Clone)]
pub struct IpInfo {
    pub ip: String,
    pub country: String,
    pub city: String,
    pub org: String,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawDns {
    dns: RawDnsServer,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawDnsServer {
    ip: String,
    geo: String,
}

#[derive(Serialize)]
pub struct DnsInfo {
    pub ip: String,
    pub geo: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub system: Option<IpInfo>,
    pub tunnel: Option<IpInfo>,
    pub dns: Option<DnsInfo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeedResult {
    pub down_mbps: f64,
    pub up_mbps: f64,
    pub ping_ms: f64,
    pub jitter_ms: f64,
}

/// A live reading streamed to the UI while the test runs.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpeedProgress {
    /// `ping`, `download`, `upload` or `done`.
    pub phase: &'static str,
    /// Current speed in Mbps (zero during the ping phase).
    pub mbps: f64,
    /// How far through the current phase, from 0 to 1.
    pub progress: f64,
    pub ping_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
}

fn direct_client() -> Result<Client> {
    Ok(Client::builder().no_proxy().timeout(Duration::from_secs(12)).build()?)
}

fn tunnel_client(timeout: u64) -> Result<Client> {
    let proxy = Proxy::all(crate::config::proxy_url())?;
    Ok(Client::builder().proxy(proxy).timeout(Duration::from_secs(timeout)).build()?)
}

async fn ip_info(client: &Client) -> Option<IpInfo> {
    let raw: RawIp = client.get(IP_URL).send().await.ok()?.json().await.ok()?;
    (!raw.ip.is_empty()).then_some(IpInfo { ip: raw.ip, country: raw.country, city: raw.city, org: raw.connection.org })
}

/// Compares what the internet sees through the system route, the tunnel and the DNS resolver.
pub async fn run(connected: bool) -> Diagnostics {
    let system = match direct_client() {
        Ok(client) => ip_info(&client).await,
        Err(_) => None,
    };
    let tunnel = match (connected, tunnel_client(12)) {
        (true, Ok(client)) => ip_info(&client).await,
        _ => None,
    };
    let dns = match direct_client() {
        Ok(client) => match client.get(DNS_URL).send().await {
            Ok(response) => response.json::<RawDns>().await.ok().filter(|d| !d.dns.ip.is_empty()),
            Err(_) => None,
        },
        Err(_) => None,
    }
    .map(|d| DnsInfo { ip: d.dns.ip, geo: d.dns.geo });
    Diagnostics { system, tunnel, dns }
}

/// Asks a running speed test to stop at its next reading.
pub fn cancel_speed_test() {
    CANCELLED.store(true, Ordering::Relaxed);
}

fn emit(app: &AppHandle, phase: &'static str, mbps: f64, progress: f64, ping: Option<f64>, jitter: Option<f64>) {
    let _ = app.emit("speed-progress", SpeedProgress { phase, mbps, progress, ping_ms: ping, jitter_ms: jitter });
}

/// Latency through the tunnel: the median of a few tiny requests, plus the jitter between them.
async fn measure_ping(app: &AppHandle) -> Result<(f64, f64)> {
    let client = tunnel_client(6)?;
    // The first request pays for opening the connection, so it is not counted.
    let _ = client.get(PING_URL).send().await;

    let mut samples = Vec::new();
    for i in 0..PING_SAMPLES {
        if CANCELLED.load(Ordering::Relaxed) {
            return Err(AppError::Invalid("speed test cancelled".into()));
        }
        let started = Instant::now();
        if client.get(PING_URL).send().await.is_ok() {
            samples.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        let (median, jitter) = latency_stats(&samples);
        emit(app, "ping", 0.0, (i + 1) as f64 / PING_SAMPLES as f64, Some(median), Some(jitter));
    }
    if samples.is_empty() {
        return Err(AppError::Invalid("the speed test server could not be reached through the tunnel".into()));
    }
    Ok(latency_stats(&samples))
}

/// Runs `workers` parallel streams for a fixed time, streaming a live reading while they go.
/// Returns the average speed in Mbps over the whole phase.
async fn measure_phase<F, Fut>(app: &AppHandle, phase: &'static str, workers: usize, ping: (f64, f64), spawn_worker: F) -> Result<f64>
where
    F: Fn(Arc<AtomicU64>, Arc<AtomicBool>) -> Fut,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let bytes = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let handles: Vec<_> = (0..workers)
        .map(|_| tauri::async_runtime::spawn(spawn_worker(bytes.clone(), stop.clone())))
        .collect();

    let started = Instant::now();
    let (mut last_at, mut last_bytes, mut live) = (started, 0u64, 0.0f64);
    let cancelled = loop {
        tokio::time::sleep(SAMPLE_EVERY).await;
        let (now, total) = (Instant::now(), bytes.load(Ordering::Relaxed));
        let instant = mbps_between(total - last_bytes, now - last_at);
        live = live + (instant - live) * LIVE_SMOOTHING;
        (last_at, last_bytes) = (now, total);

        let progress = (started.elapsed().as_secs_f64() / PHASE_SECONDS).min(1.0);
        emit(app, phase, live, progress, Some(ping.0), Some(ping.1));
        let cancelled = CANCELLED.load(Ordering::Relaxed);
        if progress >= 1.0 || cancelled || handles.iter().all(|h| h.inner().is_finished()) {
            break cancelled;
        }
    };
    stop.store(true, Ordering::Relaxed);
    for handle in &handles {
        handle.abort();
    }
    if cancelled {
        return Err(AppError::Invalid("speed test cancelled".into()));
    }
    let total = bytes.load(Ordering::Relaxed);
    if total == 0 {
        return Err(AppError::Invalid(no_data_message(REFUSED_STATUS.load(Ordering::Relaxed))));
    }
    Ok(mbps_between(total, started.elapsed()))
}

pub async fn speed_test(app: &AppHandle) -> Result<SpeedResult> {
    CANCELLED.store(false, Ordering::Relaxed);
    REFUSED_STATUS.store(0, Ordering::Relaxed);
    emit(app, "ping", 0.0, 0.0, None, None);
    let ping = measure_ping(app).await?;

    let down_client = tunnel_client(30)?;
    let down_mbps = measure_phase(app, "download", DOWN_STREAMS, ping, move |bytes, stop| {
        let client = down_client.clone();
        async move {
            while !stop.load(Ordering::Relaxed) {
                let Ok(response) = client.get(DOWN_URL).send().await else { return };
                let mut response = match response.error_for_status() {
                    Ok(r) => r,
                    Err(error) => {
                        note_refusal(&error);
                        return;
                    }
                };
                while let Ok(Some(chunk)) = response.chunk().await {
                    bytes.fetch_add(chunk.len() as u64, Ordering::Relaxed);
                    if stop.load(Ordering::Relaxed) {
                        return;
                    }
                }
            }
        }
    })
    .await?;

    let up_client = tunnel_client(30)?;
    let up_mbps = measure_phase(app, "upload", UP_STREAMS, ping, move |bytes, stop| {
        let client = up_client.clone();
        async move {
            while !stop.load(Ordering::Relaxed) {
                let sent = client.post(UP_URL).body(vec![0u8; UP_CHUNK]).send().await.and_then(|r| r.error_for_status());
                if let Err(error) = sent {
                    note_refusal(&error);
                    return;
                }
                bytes.fetch_add(UP_CHUNK as u64, Ordering::Relaxed);
            }
        }
    })
    .await?;

    emit(app, "done", 0.0, 1.0, Some(ping.0), Some(ping.1));
    Ok(SpeedResult { down_mbps, up_mbps, ping_ms: ping.0, jitter_ms: ping.1 })
}

fn note_refusal(error: &reqwest::Error) {
    if let Some(status) = error.status() {
        REFUSED_STATUS.store(status.as_u16(), Ordering::Relaxed);
    }
}

/// What to tell the user when a phase moved no data: the server answered with an error, or it
/// never answered at all.
fn no_data_message(refused_status: u16) -> String {
    match refused_status {
        0 => "the speed test server could not be reached through the tunnel".to_string(),
        status => format!("the speed test server refused the request (HTTP {status})"),
    }
}

/// Median latency and the average change between consecutive samples (jitter), in milliseconds.
fn latency_stats(samples: &[f64]) -> (f64, f64) {
    if samples.is_empty() {
        return (0.0, 0.0);
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let mid = sorted.len() / 2;
    let median = if sorted.len().is_multiple_of(2) { (sorted[mid - 1] + sorted[mid]) / 2.0 } else { sorted[mid] };
    let jitter = if samples.len() < 2 {
        0.0
    } else {
        samples.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / (samples.len() - 1) as f64
    };
    (median, jitter)
}

fn mbps_between(bytes: u64, elapsed: Duration) -> f64 {
    let seconds = elapsed.as_secs_f64().max(0.001);
    (bytes as f64 * 8.0 / 1_000_000.0) / seconds
}

#[cfg(test)]
#[path = "tests/diagnostics.rs"]
mod tests;
