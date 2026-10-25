use super::*;

fn err() -> Result<u64, String> {
    Err("failed".into())
}

#[test]
fn a_working_tunnel_is_healthy_whatever_the_node_check_said() {
    assert_eq!(classify(&err(), &Ok(40), None), Verdict::Healthy { latency_ms: 40 });
    assert_eq!(classify(&Ok(30), &Ok(40), None), Verdict::Healthy { latency_ms: 40 });
}

#[test]
fn a_reachable_server_without_traffic_is_no_traffic() {
    let Verdict::Down(reason) = classify(&Ok(30), &err(), Some("ERROR whatever")) else { panic!("expected down") };
    assert_eq!(reason.cause, Cause::NoTraffic);
}

#[test]
fn a_silent_server_is_unreachable() {
    let line = "ERROR connection: open connection to 1.2.3.4:443 using outbound/selector[proxy]: context deadline exceeded";
    let Verdict::Down(reason) = classify(&err(), &err(), Some(line)) else { panic!("expected down") };
    assert_eq!(reason.cause, Cause::ServerUnreachable);
}

#[test]
fn handshake_wording_means_the_server_rejected_us() {
    assert_eq!(cause_from_log("ERROR dial: tls: bad certificate"), Cause::Handshake);
    assert_eq!(cause_from_log("ERROR vless: authentication failed"), Cause::Handshake);
    assert_eq!(cause_from_log("ERROR dial tcp: connection refused"), Cause::ServerUnreachable);
}

#[test]
fn no_log_line_defaults_to_unreachable() {
    let Verdict::Down(reason) = classify(&err(), &err(), None) else { panic!("expected down") };
    assert_eq!(reason.cause, Cause::ServerUnreachable);
}

#[test]
fn udp_protocols_skip_the_tcp_check() {
    assert!(!needs_tcp_check("hysteria2"));
    assert!(!needs_tcp_check("tuic"));
    assert!(!needs_tcp_check("hysteria"));
    assert!(!needs_tcp_check("wireguard"));
    assert!(needs_tcp_check("vless"));
    assert!(needs_tcp_check("shadowsocks"));
    assert!(needs_tcp_check("ssh"));
}
