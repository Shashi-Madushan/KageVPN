use super::*;

#[test]
fn median_and_jitter_of_latency_samples() {
    let (median, jitter) = latency_stats(&[40.0, 50.0, 45.0, 100.0]);
    assert_eq!(median, 47.5);
    assert!((jitter - (10.0 + 5.0 + 55.0) / 3.0).abs() < 1e-9);
    assert_eq!(latency_stats(&[]), (0.0, 0.0));
    assert_eq!(latency_stats(&[30.0]), (30.0, 0.0));
}

#[test]
fn converts_bytes_over_time_to_megabits() {
    let rate = mbps_between(12_500_000, Duration::from_secs(1));
    assert!((rate - 100.0).abs() < 1e-9);
    assert!(mbps_between(0, Duration::from_secs(1)) == 0.0);
}

#[test]
fn a_refused_speed_test_says_so_instead_of_claiming_the_server_is_unreachable() {
    assert!(no_data_message(0).contains("could not be reached"));
    assert!(no_data_message(403).contains("refused") && no_data_message(403).contains("403"));
}

#[test]
fn each_download_request_asks_for_a_size_the_speed_server_accepts() {
    // Cloudflare answers 403 to very large requests (100 MB was refused), and the worker loops
    // for as long as the phase runs, so a smaller size loses nothing.
    let bytes: u64 = DOWN_URL.rsplit('=').next().unwrap().parse().unwrap();
    assert!(bytes <= 50_000_000, "{DOWN_URL}");
}
