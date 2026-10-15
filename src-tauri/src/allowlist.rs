//! The server addresses the kill switch keeps reachable while it blocks everything else.
//!
//! A server name can resolve to different addresses over time (CDN-fronted hosts rotate them), and
//! the block cannot resolve names itself. So the app looks the names up on every connect and every
//! few minutes while connected, remembers the last few answers per server, and writes the union to
//! `<data dir>/allow`, which the root helper reads when it puts the block up.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::Duration;

use crate::paths::AppPaths;

/// Answers remembered per server. An address that stopped appearing a few lookups ago is dropped.
const REMEMBERED_LOOKUPS: usize = 3;
/// A lookup can hang while the block is up (it drops DNS), so none may hold up a reconnect long.
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(3);
pub const MAX_ADDRESSES: usize = 64;
/// How often the list is refreshed while connected.
pub const REFRESH_EVERY: Duration = Duration::from_secs(300);

#[derive(Default)]
pub struct History {
    by_server: HashMap<String, VecDeque<Vec<String>>>,
}

impl History {
    /// Keeps `addresses` as the newest answer for `server`. An empty answer (the lookup failed or
    /// was blocked) changes nothing, so the addresses known so far stay allowed.
    pub fn record(&mut self, server: &str, mut addresses: Vec<String>) {
        if addresses.is_empty() {
            return;
        }
        addresses.sort();
        addresses.dedup();
        let answers = self.by_server.entry(server.to_string()).or_default();
        if answers.front() != Some(&addresses) {
            answers.push_front(addresses);
            answers.truncate(REMEMBERED_LOOKUPS);
        }
    }

    /// Stops remembering servers that are no longer in use.
    pub fn keep_only(&mut self, servers: &[String]) {
        self.by_server.retain(|server, _| servers.contains(server));
    }

    /// Every remembered address of `servers`, sorted and without repeats, at most `MAX_ADDRESSES`.
    pub fn addresses(&self, servers: &[String]) -> Vec<String> {
        let mut all: Vec<String> = servers.iter().filter_map(|s| self.by_server.get(s)).flatten().flatten().cloned().collect();
        all.sort();
        all.dedup();
        all.truncate(MAX_ADDRESSES);
        all
    }
}

#[derive(Default)]
struct Current {
    history: History,
    targets: Vec<(String, u16)>,
}

static CURRENT: Mutex<Option<Current>> = Mutex::new(None);

/// The addresses remembered for one server; empty when nothing is known.
pub fn known(server: &str) -> Vec<String> {
    let guard = CURRENT.lock().unwrap();
    guard.as_ref().map(|c| c.history.addresses(&[server.to_string()])).unwrap_or_default()
}

/// Looks the servers up again, remembers the answers, writes `<data dir>/allow` and returns the
/// addresses as a comma list. With `targets` the set of servers changes to those (a new connect);
/// without, the servers of the current connection are refreshed. With `lookup` false (the kill
/// switch is blocking, and DNS with it) nothing is looked up and what is known is returned.
pub async fn refresh(paths: &AppPaths, targets: Option<Vec<(String, u16)>>, lookup: bool) -> String {
    let targets = {
        let mut guard = CURRENT.lock().unwrap();
        let current = guard.get_or_insert_with(Current::default);
        if let Some(targets) = targets {
            current.history.keep_only(&targets.iter().map(|(server, _)| server.clone()).collect::<Vec<_>>());
            current.targets = targets;
        }
        current.targets.clone()
    };

    let mut answers = Vec::new();
    for (server, port) in targets.iter().filter(|_| lookup) {
        let found = match tokio::time::timeout(LOOKUP_TIMEOUT, tokio::net::lookup_host((server.as_str(), *port))).await {
            Ok(Ok(found)) => found.map(|a| a.ip().to_string()).collect(),
            _ => Vec::new(),
        };
        answers.push((server.clone(), found));
    }

    let servers: Vec<String> = targets.iter().map(|(server, _)| server.clone()).collect();
    let list = {
        let mut guard = CURRENT.lock().unwrap();
        let current = guard.get_or_insert_with(Current::default);
        for (server, found) in answers {
            current.history.record(&server, found);
        }
        current.history.addresses(&servers).join(",")
    };
    let _ = crate::fsutil::write_private_atomic(&paths.allow_file(), list.as_bytes());
    list
}

#[cfg(test)]
#[path = "tests/allowlist.rs"]
mod tests;
