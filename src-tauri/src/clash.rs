use std::time::Duration;

use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::time::sleep;

use crate::config::{node_tag, CLASH_PORT};
use crate::error::{AppError, Result};
use crate::state::AppState;

const ATTEMPTS: u32 = 8;
const RETRY_DELAY: Duration = Duration::from_millis(300);
const DELAY_URL: &str = "http%3A%2F%2Fcp.cloudflare.com%2Fgenerate_204";
const DELAY_TIMEOUT_MS: u32 = 5000;

#[derive(Deserialize)]
struct DelayResponse {
    delay: u64,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawMetadata {
    network: String,
    host: String,
    #[serde(rename = "destinationIP")]
    destination_ip: String,
    #[serde(rename = "destinationPort")]
    destination_port: String,
    #[serde(rename = "processPath")]
    process_path: String,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawConnection {
    id: String,
    metadata: RawMetadata,
    upload: u64,
    download: u64,
    start: String,
    chains: Vec<String>,
    rule: String,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ConnectionsResponse {
    connections: Vec<RawConnection>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    pub host: String,
    pub destination_ip: String,
    pub destination_port: String,
    pub process: String,
    /// The app bundle the process belongs to, when it is inside one, for routing rules by app.
    pub app_path: Option<String>,
    pub network: String,
    pub up: u64,
    pub down: u64,
    pub start: String,
    pub chain: String,
    pub rule: String,
}

fn client() -> Result<Client> {
    Ok(Client::builder().no_proxy().timeout(Duration::from_secs(10)).build()?)
}

fn request(app: &AppHandle, method: reqwest::Method, path: &str) -> Result<RequestBuilder> {
    let secret = app
        .state::<AppState>()
        .clash_secret
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| AppError::Invalid("not connected".into()))?;
    Ok(client()?
        .request(method, format!("http://127.0.0.1:{CLASH_PORT}{path}"))
        .bearer_auth(secret))
}

/// Switches the active node of the `proxy` selector without restarting sing-box.
pub async fn select_profile(app: &AppHandle, profile_id: &str) -> Result<()> {
    let mut last = AppError::Invalid("could not reach the sing-box controller".into());
    for _ in 0..ATTEMPTS {
        let response = request(app, reqwest::Method::PUT, "/proxies/proxy")?
            .json(&serde_json::json!({ "name": node_tag(profile_id) }))
            .send()
            .await;
        match response {
            Ok(r) if r.status().is_success() => return Ok(()),
            Ok(r) => last = AppError::Launch(format!("controller replied {}", r.status())),
            Err(e) => last = e.into(),
        }
        sleep(RETRY_DELAY).await;
    }
    Err(last)
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ProxiesResponse {
    proxies: std::collections::HashMap<String, serde_json::Value>,
}

/// The outbound tags the running engine knows. Servers added after connecting, or beyond the
/// number a config holds, are not among them and have to be measured some other way.
pub async fn known_tags(app: &AppHandle) -> Result<std::collections::HashSet<String>> {
    let response = request(app, reqwest::Method::GET, "/proxies")?.send().await?.error_for_status()?;
    Ok(response.json::<ProxiesResponse>().await?.proxies.into_keys().collect())
}

pub async fn node_delay(app: &AppHandle, profile_id: &str) -> Result<u64> {
    delay_by_tag(app, &node_tag(profile_id)).await
}

/// Delay of an outbound or group measured by sing-box itself, independent of the system route.
pub async fn delay_by_tag(app: &AppHandle, tag: &str) -> Result<u64> {
    let path = format!("/proxies/{tag}/delay?url={DELAY_URL}&timeout={DELAY_TIMEOUT_MS}");
    let response = request(app, reqwest::Method::GET, &path)?.send().await?.error_for_status()?;
    Ok(response.json::<DelayResponse>().await?.delay)
}

pub async fn connections(app: &AppHandle) -> Result<Vec<Connection>> {
    let response = request(app, reqwest::Method::GET, "/connections")?.send().await?.error_for_status()?;
    let parsed: ConnectionsResponse = response.json().await?;
    Ok(parsed.connections.into_iter().map(simplify).collect())
}

pub async fn close_connection(app: &AppHandle, id: &str) -> Result<()> {
    request(app, reqwest::Method::DELETE, &format!("/connections/{id}"))?.send().await?;
    Ok(())
}

pub async fn close_all(app: &AppHandle) -> Result<()> {
    request(app, reqwest::Method::DELETE, "/connections")?.send().await?;
    Ok(())
}

fn simplify(raw: RawConnection) -> Connection {
    let process = raw.metadata.process_path.rsplit('/').next().unwrap_or_default().to_string();
    Connection {
        id: raw.id,
        host: raw.metadata.host,
        destination_ip: raw.metadata.destination_ip,
        destination_port: raw.metadata.destination_port,
        process,
        app_path: crate::apps::bundle_of(&raw.metadata.process_path).map(str::to_string),
        network: raw.metadata.network,
        up: raw.upload,
        down: raw.download,
        start: raw.start,
        chain: raw.chains.first().cloned().unwrap_or_default(),
        rule: raw.rule,
    }
}
