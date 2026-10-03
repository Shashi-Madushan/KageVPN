use std::time::{Duration, Instant};

use reqwest::{Client, Proxy};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::config::MIXED_PORT;
use crate::error::{AppError, Result};

const TCP_TIMEOUT: Duration = Duration::from_secs(5);
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
use crate::app_config::urls::{CONNECTION_PROBE as PROBE_URL, EXIT_IP as IP_URL};

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

pub async fn proxy_delay() -> Result<u64> {
    let started = Instant::now();
    proxied_client()?.get(PROBE_URL).send().await?.error_for_status()?;
    Ok(elapsed_ms(started))
}

pub async fn exit_ip() -> Result<String> {
    let body = proxied_client()?.get(IP_URL).send().await?.error_for_status()?.text().await?;
    Ok(body.trim().to_string())
}

fn proxied_client() -> Result<Client> {
    let proxy = Proxy::all(format!("socks5h://127.0.0.1:{MIXED_PORT}"))?;
    Ok(Client::builder().proxy(proxy).timeout(HTTP_TIMEOUT).build()?)
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}
