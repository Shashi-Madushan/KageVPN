use super::*;

const LINK: &str = "trojan://p%40ss%3Aword@t.example.com:8443?security=tls&sni=front.example.com&fp=chrome&alpn=h2%2Chttp%2F1.1&type=ws&path=%2Fws&host=cdn.example.com#Trojan%20WS";

#[test]
fn parses_password_transport_and_name() {
    let p = parse(LINK).unwrap();
    assert_eq!((p.protocol.as_str(), p.port, p.name.as_str()), ("trojan", 8443, "Trojan WS"));
    assert_eq!(p.password, "p@ss:word");
    assert_eq!(p.sni.as_deref(), Some("front.example.com"));
    assert_eq!(p.transport.path.as_deref(), Some("/ws"));
    assert_eq!(p.alpn, ["h2", "http/1.1"]);
}

#[test]
fn round_trips_through_its_own_link() {
    let first = parse(LINK).unwrap();
    let again = parse(&to_link(&first)).unwrap();
    assert!(first.same_endpoint(&again));
    assert_eq!(again.password, "p@ss:word");
}

#[test]
fn defaults_the_port_and_accepts_the_peer_alias_and_insecure_flag() {
    let p = parse("trojan://secret@t.example.com?peer=front.example.com&allowInsecure=1").unwrap();
    assert_eq!((p.port, p.sni.as_deref(), p.allow_insecure), (443, Some("front.example.com"), true));
}

#[test]
fn rejects_what_it_cannot_run() {
    for link in [
        "trojan://@t.example.com:443",
        "trojan://secret@t.example.com:443?security=reality",
        "trojan://secret@t.example.com:443?type=xhttp",
    ] {
        assert!(parse(link).is_err(), "{link}");
    }
}

#[test]
fn outbound_has_password_tls_and_transport() {
    let out = outbound(&parse(LINK).unwrap());
    assert_eq!(out["type"], "trojan");
    assert_eq!(out["password"], "p@ss:word");
    assert_eq!(out["tls"]["server_name"], "front.example.com");
    assert_eq!(out["transport"]["type"], "ws");
}
