use super::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A local web server that answers every request with `status` and counts them.
fn counted_site(status: &'static str) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/generate_204", listener.local_addr().unwrap());
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    std::thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            counter.fetch_add(1, Ordering::SeqCst);
            let _ = stream.read(&mut [0u8; 2048]);
            let _ = stream.write_all(format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes());
        }
    });
    (url, hits)
}

fn site(status: &'static str) -> String {
    counted_site(status).0
}

/// An address nothing listens on.
fn closed_port() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    format!("http://{}/", listener.local_addr().unwrap())
}

fn run<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
}

#[test]
fn one_blocked_probe_site_does_not_make_a_working_tunnel_look_dead() {
    let (dead, alive) = (closed_port(), site("204 No Content"));
    let client = Client::builder().no_proxy().build().unwrap();
    assert!(run(first_answer(&client, &[&dead, &alive])).is_ok());
}

#[test]
fn the_check_fails_only_when_every_site_fails() {
    let (first, second, rejecting) = (closed_port(), closed_port(), site("503 Service Unavailable"));
    let client = Client::builder().no_proxy().build().unwrap();
    assert!(run(first_answer(&client, &[&first, &second, &rejecting])).is_err());
    assert!(run(first_answer(&client, &[])).is_err());
}

#[test]
fn a_site_answering_first_is_not_followed_by_more_requests() {
    // A healthy tunnel must cost one request per check, not three.
    let ((first, first_hits), (second, second_hits)) = (counted_site("204 No Content"), counted_site("204 No Content"));
    let client = Client::builder().no_proxy().build().unwrap();
    assert!(run(first_answer(&client, &[&first, &second])).is_ok());
    assert_eq!((first_hits.load(Ordering::SeqCst), second_hits.load(Ordering::SeqCst)), (1, 0));
}

#[test]
fn a_port_that_frees_up_soon_is_waited_for() {
    // A sing-box that was just stopped can hold its port for a few seconds.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let started = std::time::Instant::now();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        drop(listener);
    });
    assert!(run(wait_until_closed(port, Duration::from_secs(5))));
    assert!(started.elapsed() >= Duration::from_millis(400), "it did wait for the listener to go away");
    assert!(started.elapsed() < Duration::from_secs(4));
}

#[test]
fn a_port_that_stays_busy_is_reported_busy_after_the_wait() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    assert!(!run(wait_until_closed(port, Duration::from_millis(600))));
    assert!(!run(wait_until_closed(port, Duration::ZERO)), "still busy");
    drop(listener);
    assert!(run(wait_until_closed(port, Duration::from_millis(0))), "free once the listener is gone");
}

#[test]
fn a_free_preferred_port_is_used_and_a_busy_one_is_skipped() {
    // A port found by asking the system, then released: free unless something grabs it meanwhile.
    let preferred = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    assert_eq!(run(free_port(preferred, &[])), Some(preferred));

    let holder = TcpListener::bind(("127.0.0.1", preferred)).unwrap();
    let next = run(free_port(preferred, &[])).expect("another port is found");
    assert_ne!(next, preferred, "the busy port is not offered");
    drop(holder);
    assert_eq!(run(free_port(preferred, &[])), Some(preferred), "free again once released");
}

#[test]
fn a_port_to_avoid_is_never_returned() {
    let preferred = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let chosen = run(free_port(preferred, &[preferred])).unwrap();
    assert_ne!(chosen, preferred);
}

#[test]
fn the_proxied_client_follows_the_port() {
    // Switching the port must not leave probes aimed at the old one.
    crate::config::set_ports(DEFAULT_MIXED_PORT, DEFAULT_CLASH_PORT);
    assert!(proxied_client().is_ok());
    crate::config::set_ports(2191, 19291);
    assert!(proxied_client().is_ok());
    assert_eq!((crate::config::mixed_port(), crate::config::clash_port()), (2191, 19291));
    crate::config::set_ports(DEFAULT_MIXED_PORT, DEFAULT_CLASH_PORT);
}
