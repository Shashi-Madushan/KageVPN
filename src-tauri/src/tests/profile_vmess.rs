use super::*;

fn link(json: serde_json::Value) -> String {
    format!("vmess://{}", STANDARD.encode(json.to_string()))
}

fn cfg(profile: &Profile) -> VmessSettings {
    profile.settings_as().unwrap()
}

#[test]
fn parses_the_sample_with_tls_websocket_and_name() {
    let p = parse(Vmess.sample()).unwrap();
    assert_eq!((p.name.as_str(), p.server.as_str(), p.port), ("Sample VMess", "vmess.example.com", 443));
    let c = cfg(&p);
    assert!(c.tls_enabled);
    assert_eq!((c.transport.kind, c.transport.path.as_deref(), c.transport.host.as_deref()), (TransportKind::Ws, Some("/ws"), Some("cdn.example.com")));
    assert_eq!(c.tls.sni.as_deref(), Some("vmess.example.com"));
}

#[test]
fn accepts_numbers_written_as_numbers_and_a_plain_tcp_server() {
    let p = parse(&link(serde_json::json!({
        "add": "h.example.com", "port": 8443, "id": "11111111-2222-4333-8444-555555555555", "aid": 0, "net": "tcp", "tls": "", "ps": "Plain"
    })))
    .unwrap();
    assert_eq!(p.port, 8443);
    assert!(!cfg(&p).tls_enabled);
    assert!(Vmess.outbound(&p).get("tls").is_none());
}

#[test]
fn a_grpc_service_name_travels_in_the_path_field() {
    let json = serde_json::json!({ "add": "h.example.com", "port": "443", "id": "11111111-2222-4333-8444-555555555555", "net": "grpc", "path": "svc", "tls": "tls" });
    let p = parse(&link(json)).unwrap();
    assert_eq!(cfg(&p).transport.service_name.as_deref(), Some("svc"));
    let again = parse(&to_link(&p)).unwrap();
    assert_eq!(cfg(&again).transport.service_name.as_deref(), Some("svc"));
}

#[test]
fn rejects_what_it_cannot_run() {
    let base = serde_json::json!({ "add": "h.example.com", "port": "443", "id": "11111111-2222-4333-8444-555555555555" });
    for change in [
        serde_json::json!({ "aid": "64" }),
        serde_json::json!({ "net": "h2" }),
        serde_json::json!({ "net": "kcp" }),
        serde_json::json!({ "id": "nope" }),
        serde_json::json!({ "port": "0" }),
        serde_json::json!({ "add": "" }),
    ] {
        let mut json = base.clone();
        json.as_object_mut().unwrap().extend(change.as_object().unwrap().clone());
        assert!(parse(&link(json.clone())).is_err(), "{json}");
    }
    assert!(parse("vmess://not base64 !!").is_err());
}
