use std::sync::Mutex;
use std::time::{Duration, Instant};

use reqwest::{Client, Proxy};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::config::{mixed_port, DEFAULT_CLASH_PORT, DEFAULT_MIXED_PORT};
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
    port_open(mixed_port()).await
}

async fn port_open(port: u16) -> bool {
    TcpStream::connect(("127.0.0.1", port)).await.is_ok()
}

/// How many ports after the preferred one are tried before asking the system for any free port.
const PORT_SEARCH_SPAN: u16 = 50;

/// Whether this program could listen on `port` of the loopback address now. A listener on another
/// address (such as every address) can still let a bind succeed on macOS, so an answering
/// connection counts as taken too.
async fn port_free(port: u16) -> bool {
    !port_open(port).await && std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// `preferred` when it is free, else the next free port after it (skipping `avoid`), else any port
/// the system hands out.
pub async fn free_port(preferred: u16, avoid: &[u16]) -> Option<u16> {
    for port in preferred..=preferred.saturating_add(PORT_SEARCH_SPAN) {
        if !avoid.contains(&port) && port_free(port).await {
            return Some(port);
        }
    }
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).ok()?;
    listener.local_addr().ok().map(|address| address.port()).filter(|port| !avoid.contains(port))
}

/// The two ports sing-box will listen on: the local proxy and the controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ports {
    pub mixed: u16,
    pub clash: u16,
}

impl Ports {
    pub fn is_default(self) -> bool {
        self.mixed == DEFAULT_MIXED_PORT && self.clash == DEFAULT_CLASH_PORT
    }
}

/// Picks the ports for a new connection. `mixed_taken_by_us` is true when the busy proxy port is
/// most likely a sing-box this app stopped a moment ago: it is waited for, because moving to
/// another port would leave that process running.
pub async fn choose_ports(mixed_taken_by_us: bool) -> Result<Ports> {
    if mixed_taken_by_us && port_open(DEFAULT_MIXED_PORT).await && !wait_until_closed(DEFAULT_MIXED_PORT, Duration::from_secs(10)).await {
        return Err(AppError::Launch(format!("port {DEFAULT_MIXED_PORT} is already in use, is another sing-box running?")));
    }
    let none_free = || AppError::Launch("no free local port was found for the proxy".into());
    let mixed = free_port(DEFAULT_MIXED_PORT, &[]).await.ok_or_else(none_free)?;
    let clash = free_port(DEFAULT_CLASH_PORT, &[mixed]).await.ok_or_else(none_free)?;
    Ok(Ports { mixed, clash })
}

/// Waits until nothing listens on `port` any more, for at most `timeout`. Returns whether it is free.
pub async fn wait_until_closed(port: u16, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if !port_open(port).await {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
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
/// It is built again when the proxy address or login changes.
fn proxied_client() -> Result<Client> {
    static CLIENT: Mutex<Option<(String, Client)>> = Mutex::new(None);
    let proxy_url = crate::config::proxy_url();
    let mut cached = CLIENT.lock().unwrap();
    if let Some((built_for, client)) = cached.as_ref() {
        if *built_for == proxy_url {
            return Ok(client.clone());
        }
    }
    let proxy = Proxy::all(&proxy_url)?;
    let client = Client::builder().proxy(proxy).timeout(HTTP_TIMEOUT).pool_max_idle_per_host(0).build()?;
    *cached = Some((proxy_url, client.clone()));
    Ok(client)
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}

#[cfg(test)]
#[path = "tests/net.rs"]
mod tests;
