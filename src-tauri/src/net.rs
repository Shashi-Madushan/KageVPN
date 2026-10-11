use std::sync::OnceLock;
use std::time::{Duration, Instant};

use reqwest::{Client, Proxy};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::config::MIXED_PORT;
use crate::error::{AppError, Result};

const TCP_TIMEOUT: Duration = Duration::from_secs(5);
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
/// Each probe address gets this long, so three failing in a row still finish within a health cycle.
const PROBE_TIMEOUT: Duration = Duration::from_secs(6);
use crate::app_config::urls::{CONNECTION_PROBE as PROBE_URL, CONNECTION_PROBE_FALLBACKS as PROBE_FALLBACKS, EXIT_IP as IP_URL};

pub async fn tcp_latency(host: &str, port: u16) -> Result<u64> {
    let started = Instant::now();
    timeout(TCP_TIMEOUT, TcpStream::connect((host, port)))
        .await
        .map_err(|_| AppError::Launch("timed out".into()))??;
    Ok(elapsed_ms(started))
}

pub async fn proxy_port_open() -> bool {
    TcpStream::connect(("127.0.0.1", MIXED_PORT)).await.is_ok()
}

/// How long a tiny page takes through the tunnel. A healthy tunnel costs one request; the other
/// addresses are only tried when the first fails.
pub async fn proxy_delay() -> Result<u64> {
    let urls: Vec<&str> = std::iter::once(PROBE_URL).chain(PROBE_FALLBACKS).collect();
    first_answer(&proxied_client()?, &urls).await
}

/// The time of the first address in `urls` that answers, or the last error when none does.
async fn first_answer(client: &Client, urls: &[&str]) -> Result<u64> {
    let mut last = AppError::Launch("no address to check".into());
    for url in urls {
        let started = Instant::now();
        match client.get(*url).timeout(PROBE_TIMEOUT).send().await.and_then(|response| response.error_for_status()) {
            Ok(_) => return Ok(elapsed_ms(started)),
            Err(error) => last = error.into(),
        }
    }
    Err(last)
}

pub async fn exit_ip() -> Result<String> {
    let body = proxied_client()?.get(IP_URL).send().await?.error_for_status()?.text().await?;
    Ok(body.trim().to_string())
}

/// One client for every probe: building a client loads TLS roots, which is wasteful to repeat
/// every few seconds. Idle connections are not kept, so a restarted tunnel never meets a stale one.
fn proxied_client() -> Result<Client> {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    if let Some(client) = CLIENT.get() {
        return Ok(client.clone());
    }
    let proxy = Proxy::all(format!("socks5h://127.0.0.1:{MIXED_PORT}"))?;
    let client = Client::builder().proxy(proxy).timeout(HTTP_TIMEOUT).pool_max_idle_per_host(0).build()?;
    Ok(CLIENT.get_or_init(|| client).clone())
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}

#[cfg(test)]
#[path = "tests/net.rs"]
mod tests;
