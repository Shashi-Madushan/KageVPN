//! Notices when the Mac wakes from sleep or switches networks, and restarts the tunnel if it
//! stopped working. No extra processes or system frameworks: one cheap check every few seconds.

use std::ffi::CStr;
use std::time::{Duration, Instant, SystemTime};

use tauri::{AppHandle, Manager};
use tokio::time::sleep;

use crate::session::State;
use crate::state::AppState;
use crate::{applog, net, reliability};

const TICK: Duration = Duration::from_secs(3);
/// A tick that took this much longer on the wall clock than on the monotonic clock means the Mac slept.
const SLEEP_GAP: Duration = Duration::from_secs(10);
const WAKE_SETTLE: Duration = Duration::from_secs(4);
const NETWORK_SETTLE: Duration = Duration::from_secs(2);
const START_SETTLE: Duration = Duration::from_secs(5);
const PROBE_ATTEMPTS: u32 = 2;
const PROBE_PAUSE: Duration = Duration::from_secs(2);
/// While a reconnect that gave up is waiting, how often to check whether the server is reachable
/// again. The internet can come back without any interface change (a router or ISP outage), so
/// waiting for a network change alone could wait forever.
const WAITING_POLL: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Change {
    Woke,
    Network,
}

/// Interfaces that do not describe where the Mac reaches the internet from.
pub(crate) fn is_virtual(name: &str) -> bool {
    ["lo", "utun", "awdl", "llw", "bridge", "gif", "stf", "anpi", "ap", "vmenet", "vnic", "ipsec"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// A stable, sorted description of the physical interfaces and their addresses.
pub fn fingerprint(entries: impl IntoIterator<Item = (String, String)>) -> Vec<String> {
    let mut parts: Vec<String> = entries
        .into_iter()
        .filter(|(name, _)| !is_virtual(name))
        .map(|(name, address)| format!("{name}={address}"))
        .collect();
    parts.sort();
    parts.dedup();
    parts
}

pub fn slept(wall: Duration, monotonic: Duration) -> bool {
    wall.saturating_sub(monotonic) >= SLEEP_GAP
}

/// What happened between two checks, if anything. A wake outranks an address change.
pub fn detect(wall: Duration, monotonic: Duration, before: &[String], now: &[String]) -> Option<Change> {
    if slept(wall, monotonic) {
        Some(Change::Woke)
    } else if before != now {
        Some(Change::Network)
    } else {
        None
    }
}

fn current_fingerprint() -> Vec<String> {
    let mut entries = Vec::new();
    // SAFETY: getifaddrs fills a list we only read and always release with freeifaddrs.
    unsafe {
        let mut list: *mut libc::ifaddrs = std::ptr::null_mut();
        if libc::getifaddrs(&mut list) != 0 {
            return Vec::new();
        }
        let mut cursor = list;
        while !cursor.is_null() {
            let item = &*cursor;
            let up = item.ifa_flags & libc::IFF_UP as u32 != 0 && item.ifa_flags & libc::IFF_RUNNING as u32 != 0;
            if up && !item.ifa_addr.is_null() {
                let name = CStr::from_ptr(item.ifa_name).to_string_lossy().into_owned();
                match i32::from((*item.ifa_addr).sa_family) {
                    libc::AF_INET => {
                        let addr = &*(item.ifa_addr as *const libc::sockaddr_in);
                        entries.push((name, std::net::Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr)).to_string()));
                    }
                    libc::AF_INET6 => {
                        let addr = &*(item.ifa_addr as *const libc::sockaddr_in6);
                        let ip = std::net::Ipv6Addr::from(addr.sin6_addr.s6_addr);
                        // Link-local addresses flap on their own and say nothing about the route out.
                        if (ip.segments()[0] & 0xffc0) != 0xfe80 {
                            entries.push((name, ip.to_string()));
                        }
                    }
                    _ => {}
                }
            }
            cursor = item.ifa_next;
        }
        libc::freeifaddrs(list);
    }
    fingerprint(entries)
}

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut wall = SystemTime::now();
        let mut mono = Instant::now();
        let mut known = current_fingerprint();
        let mut last_poll = Instant::now();
        // The network the Mac is on at launch counts too.
        sleep(START_SETTLE).await;
        let manual = app.state::<AppState>().manual_actions.load(std::sync::atomic::Ordering::SeqCst);
        if app.state::<AppState>().store.lock().unwrap().networks().enabled {
            crate::networks::apply(&app, manual).await;
        }
        loop {
            sleep(TICK).await;
            let (wall_now, mono_now) = (SystemTime::now(), Instant::now());
            let now = current_fingerprint();
            let change = detect(
                wall_now.duration_since(wall).unwrap_or_default(),
                mono_now.duration_since(mono),
                &known,
                &now,
            );
            wall = wall_now;
            mono = mono_now;
            known = now;
            if let Some(change) = change {
                handle(&app, change).await;
                // Whatever settled during the wait is the new normal.
                known = current_fingerprint();
                wall = SystemTime::now();
                mono = Instant::now();
            } else if last_poll.elapsed() >= WAITING_POLL {
                last_poll = Instant::now();
                resume_if_reachable(&app).await;
            }
        }
    });
}

/// Starts a reconnect that gave up again once the server answers, even if the network looks the same.
async fn resume_if_reachable(app: &AppHandle) {
    let state = app.state::<AppState>();
    if !state.reconnect_waiting.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let Some(profile) = state.store.lock().unwrap().active_profile().cloned() else { return };
    if crate::health::preflight(&profile, state.blocked.load(std::sync::atomic::Ordering::SeqCst)).await.is_ok() && reliability::take_resume(app) {
        applog::info("network watch: the server is reachable again, reconnecting");
        reliability::spawn_reconnect(app.clone());
    }
}

async fn handle(app: &AppHandle, change: Change) {
    applog::info(format!("network watch: {change:?}"));
    let manual = app.state::<AppState>().manual_actions.load(std::sync::atomic::Ordering::SeqCst);
    crate::health::recheck();
    let settle = if change == Change::Woke { WAKE_SETTLE } else { NETWORK_SETTLE };
    // A reconnect that gave up while the network was down starts over now that it changed.
    if reliability::take_resume(app) {
        applog::info("network watch: reconnecting after the network returned");
        sleep(settle).await;
        reliability::spawn_reconnect(app.clone());
        return;
    }
    // Auto-connect by network: acts once the network has settled, unless the user chose by hand.
    if app.state::<AppState>().store.lock().unwrap().networks().enabled {
        sleep(settle).await;
        crate::networks::apply(app, manual).await;
    }
    if !should_react(app) {
        return;
    }
    sleep(settle).await;
    let pid = *app.state::<AppState>().pid.lock().unwrap();
    let Some(pid) = pid else { return };
    for attempt in 0..PROBE_ATTEMPTS {
        if net::proxy_delay().await.is_ok() {
            return;
        }
        if attempt + 1 < PROBE_ATTEMPTS {
            sleep(PROBE_PAUSE).await;
        }
    }
    let why = if change == Change::Woke { "Your Mac woke up" } else { "Your network changed" };
    reliability::restart_tunnel(app, pid, why).await;
}

fn should_react(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();
    let connected = state.status.lock().unwrap().state == State::Connected;
    let auto = state.store.lock().unwrap().reliability().auto_reconnect;
    connected && auto && !state.reconnecting.load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(test)]
#[path = "tests/network_watch.rs"]
mod tests;
