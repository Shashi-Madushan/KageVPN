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
