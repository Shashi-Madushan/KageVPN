use std::time::{Duration, Instant};

use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};

use crate::config::MIXED_PORT;
use crate::error::Result;

const IP_URL: &str = "https://ipwho.is/";
const DNS_URL: &str = "http://edns.ip-api.com/json";
const DOWN_URL: &str = "https://speed.cloudflare.com/__down?bytes=25000000";
const UP_URL: &str = "https://speed.cloudflare.com/__up";
const UP_BYTES: usize = 6 * 1024 * 1024;

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
}

fn direct_client() -> Result<Client> {
    Ok(Client::builder().no_proxy().timeout(Duration::from_secs(12)).build()?)
}

fn tunnel_client(timeout: u64) -> Result<Client> {
    let proxy = Proxy::all(format!("socks5h://127.0.0.1:{MIXED_PORT}"))?;
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

pub async fn speed_test() -> Result<SpeedResult> {
    let client = tunnel_client(40)?;

    let started = Instant::now();
    let down = client.get(DOWN_URL).send().await?.error_for_status()?.bytes().await?;
    let down_mbps = mbps(down.len(), started.elapsed());

    let started = Instant::now();
    client.post(UP_URL).body(vec![0u8; UP_BYTES]).send().await?.error_for_status()?;
    let up_mbps = mbps(UP_BYTES, started.elapsed());
    Ok(SpeedResult { down_mbps, up_mbps })
}

fn mbps(bytes: usize, elapsed: Duration) -> f64 {
    let seconds = elapsed.as_secs_f64().max(0.001);
    (bytes as f64 * 8.0 / 1_000_000.0) / seconds
}
