use super::*;
use crate::profile::def;

const ID: &str = "11111111-2222-4333-8444-555555555555";

fn sample_links() -> Vec<String> {
    vec![
        format!("vless://{ID}@a.example.com:443?security=tls&type=tcp#vless"),
        "trojan://secret@b.example.com:443?security=tls&type=tcp#trojan".into(),
        "ss://YWVzLTI1Ni1nY206cGE1NXdvcmQ@c.example.com:8388#ss".into(),
        "hysteria2://auth@d.example.com:443/?sni=d.example.com#hy2".into(),
        format!("tuic://{ID}:pw@e.example.com:443?alpn=h3#tuic"),
        "hysteria://f.example.com:443?auth=pw&upmbps=10&downmbps=50#hy1".into(),
        "ssh://root:pw@g.example.com:22#ssh".into(),
        format!("wireguard://{}@h.example.com:51820?publickey={}&address=10.0.0.2#wg", crate::profile::TestKey(0xfb).url(), crate::profile::TestKey(0xfe).url()),
    ]
}

#[test]
fn one_paste_can_mix_every_protocol() {
    let (profiles, errors) = parse_many(&sample_links().join("\n"));
    assert!(errors.is_empty(), "{errors:?}");
    let kinds: Vec<_> = profiles.iter().map(|p| p.protocol.as_str()).collect();
    assert_eq!(kinds, ["vless", "trojan", "shadowsocks", "hysteria2", "tuic", "hysteria", "ssh", "wireguard"]);
}

#[test]
fn schemes_are_case_insensitive_and_web_addresses_are_ignored() {
    let (profiles, errors) = parse_many("HY2://pw@h.example.com/#up https://example.com");
    assert_eq!((profiles.len(), errors.len()), (1, 0));
    assert_eq!(profiles[0].name, "up", "a web address after a link is not part of its name");
    assert!(contains_link("hello TROJAN://x@y.example.com"));
    assert!(!contains_link("hello world"));
}

#[test]
fn each_registered_scheme_belongs_to_exactly_one_protocol() {
    let mut seen = std::collections::HashSet::new();
    for def in REGISTRY {
        for scheme in def.schemes() {
            assert!(seen.insert(*scheme), "scheme {scheme} is registered twice");
        }
    }
}

#[test]
fn registry_ids_are_unique_and_resolve_back_to_themselves() {
    let mut seen = std::collections::HashSet::new();
    for def in REGISTRY {
        assert!(seen.insert(def.id()), "protocol {} is registered twice", def.id());
        assert_eq!(ProtocolId::lookup(def.id()).map(ProtocolId::as_str), Some(def.id()));
    }
    assert!(ProtocolId::lookup("nope").is_none());
}

#[test]
fn old_saved_profiles_load_as_vless() {
    let old = format!(r#"{{"id":"1","name":"n","server":"s","port":443,"uuid":"{ID}","security":"tls"}}"#);
    let profile: Profile = serde_json::from_str(&old).unwrap();
    assert_eq!(profile.protocol, "vless");
}

#[test]
fn different_protocols_at_one_address_are_not_duplicates() {
    let a = parse_link("trojan://secret@h.example.com:443").unwrap();
    let b = parse_link("hysteria2://secret@h.example.com:443").unwrap();
    assert!(!a.same_endpoint(&b));
}

#[test]
fn an_unknown_protocol_in_saved_data_is_an_error_not_a_silent_vless() {
    let saved = r#"{"id":"1","protocol":"anytls","name":"n","server":"s","port":1}"#;
    assert!(serde_json::from_str::<Profile>(saved).is_err());
}

#[test]
fn identity_ignores_name_and_group_but_not_the_server_or_credential() {
    let a = parse_link("trojan://secret@h.example.com:443#one").unwrap();
    let b = parse_link("trojan://secret@h.example.com:443#two").unwrap();
    let other = parse_link("trojan://different@h.example.com:443#one").unwrap();
    let id = |p: &Profile| def(p.protocol).identity(p);
    assert_eq!(id(&a), id(&b));
    assert_ne!(id(&a), id(&other));
}


/// A tiny deterministic generator, so a failure can be replayed without a fuzzing crate.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[(self.next() % items.len() as u64) as usize]
    }
}

/// Valid links of every protocol, plus some that reach less common branches.
fn fuzz_seeds() -> Vec<String> {
    let mut seeds: Vec<String> = crate::profile::REGISTRY.iter().map(|d| d.sample().to_string()).collect();
    seeds.extend([
        format!("vless://{ID}@h.example.com:443?encryption=none&security=tls&sni=s.example.com&fp=chrome&alpn=h2%2Chttp%2F1.1&pcs=0123456789ABCDEF&type=ws&path=%2Fws&host=cdn.example.com#demo"),
        format!("vless://{ID}@[2001:db8::1]:8443?security=reality&pbk=abc&sid=12&flow=xtls-rprx-vision&type=grpc&serviceName=svc#v6"),
        "ss://2022-blake3-aes-256-gcm:c2VjcmV0@h.example.com:8388#modern".to_string(),
        "ss://YWVzLTI1Ni1nY206cGFzcw@h.example.com:8388/?plugin=obfs-local#plug".to_string(),
        "ss://YWVzLTI1Ni1nY206cGFzc0BoLmV4YW1wbGUuY29tOjgzODg#legacy".to_string(),
        "hysteria2://pw@h.example.com:443?obfs=salamander&obfs-password=x&insecure=1&sni=s.example.com#hy".to_string(),
        "hysteria2://pw@h.example.com:443,5000-6000?sni=s.example.com#ports".to_string(),
        format!("tuic://{ID}:p%40w@[2001:db8::1]:443?congestion_controller=cubic&udp_relay_mode=quic&reduce_rtt=1&allow_insecure=1#t6"),
        "trojan://p%40ss@h.example.com:443?security=tls&type=grpc&serviceName=svc&allowInsecure=1#t".to_string(),
    ]);
    seeds
}

/// One random edit: swap in a troublesome character, insert one, delete one, or cut the text short.
fn mutate(text: &str, rng: &mut Rng) -> String {
    const NASTY: [&str; 14] = ["é", "€", "🇩🇪", "я", "日", "\u{0}", "\u{202e}", "%", "%E2%82", "%zz", "@", ":", "#", "[" ];
    let mut chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return String::new();
    }
    let at = (rng.next() % chars.len() as u64) as usize;
    let nasty = rng.pick(&NASTY);
    match rng.next() % 4 {
        0 => {
            chars.splice(at..=at, nasty.chars());
        }
        1 => {
            chars.splice(at..at, nasty.chars());
        }
        2 => {
            chars.remove(at);
        }
        _ => chars.truncate(at),
    }
    chars.into_iter().collect()
}

#[test]
fn no_parser_panics_on_hostile_links() {
    // A panic ends the whole app (`panic = "abort"`), and links arrive from pasted text,
    // subscription servers and deep links, so nothing in here may ever panic. Mutating valid
    // links reaches far deeper into each parser than random text does.
    let seeds = fuzz_seeds();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..60_000 {
        let mut text = seeds[(rng.next() % seeds.len() as u64) as usize].clone();
        for _ in 0..=(rng.next() % 4) {
            text = mutate(&text, &mut rng);
        }
        let _ = parse_many(&text);
        let _ = parse_link(&text);
        let _ = contains_link(&text);
    }
}

#[test]
fn a_name_with_unencoded_spaces_is_kept_whole() {
    // Many panels write `#My Server 1` without encoding it; the name used to stop at the first space.
    let text = format!("vless://{ID}@a.example.com:443?security=tls#My Server 1\nvless://{ID}@b.example.com:443?security=tls#Second\r\n");
    let (profiles, errors) = parse_many(&text);
    assert!(errors.is_empty(), "{errors:?}");
    let names: Vec<_> = profiles.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["My Server 1", "Second"]);
}

#[test]
fn several_links_on_one_line_still_split_apart() {
    let text = format!("vless://{ID}@a.example.com:443?security=tls#Frankfurt DE trojan://pw@b.example.com:443#Paris FR");
    let (profiles, errors) = parse_many(&text);
    assert!(errors.is_empty(), "{errors:?}");
    let names: Vec<_> = profiles.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["Frankfurt DE", "Paris FR"]);
}

#[test]
fn words_after_a_link_without_a_name_are_not_glued_on() {
    let (profiles, _) = parse_many(&format!("vless://{ID}@a.example.com:443?security=tls (my main server)"));
    assert_eq!(profiles[0].name, "a.example.com");
}

#[test]
fn a_byte_order_mark_does_not_hide_the_first_link() {
    let text = format!("\u{feff}vless://{ID}@a.example.com:443?security=tls#First\ntrojan://pw@b.example.com:443#Second");
    assert!(contains_link(&text));
    assert_eq!(parse_many(&text).0.len(), 2);
}

#[test]
fn links_this_app_cannot_run_are_reported_not_silently_dropped() {
    let text = format!("socks5://u:p@a.example.com:1080#S\nvless://{ID}@b.example.com:443?security=tls#V\nSSR://x@c.example.com:1#H\nanytls://k@d.example.com:443");
    let (profiles, errors) = parse_many(&text);
    assert_eq!(profiles.len(), 1);
    assert_eq!(errors, ["socks5:// links are not supported yet", "ssr:// links are not supported yet", "anytls:// links are not supported yet"]);
}

#[test]
fn servers_imported_with_a_default_come_with_a_note() {
    let parsed = parse_all("hysteria://h.example.com:443?auth=pw#Speedless\nhysteria://h.example.com:444?auth=pw&upmbps=5&downmbps=5#Full\ntrojan://x@t.example.com:443#T");
    assert_eq!((parsed.profiles.len(), parsed.errors.len()), (3, 0));
    assert_eq!(parsed.notes.len(), 1);
    assert!(parsed.notes[0].starts_with("Speedless: no speeds in the link"), "{:?}", parsed.notes);
}

#[test]
fn a_wireguard_file_is_one_server_named_after_the_file() {
    let conf = format!(
        "[Interface]\nPrivateKey = {}\nAddress = 10.0.0.2/32\nDNS = 1.1.1.1\n\n[Peer]\nPublicKey = {}\nAllowedIPs = 0.0.0.0/0\nEndpoint = wg.example.com:51820\n",
        crate::profile::TestKey(0xfb),
        crate::profile::TestKey(0xfe)
    );
    let conf = conf.as_str();
    let parsed = parse_all_named(conf, Some("warp-tokyo.conf"));
    assert_eq!((parsed.profiles.len(), parsed.errors.len()), (1, 0));
    assert_eq!((parsed.profiles[0].protocol.as_str(), parsed.profiles[0].name.as_str()), ("wireguard", "warp-tokyo"));
    assert_eq!(parsed.notes, ["warp-tokyo: DNS from the file is not used; Kage's DNS and routing settings decide"]);
    let refused = parse_all(&conf.replace("Endpoint = wg.example.com:51820\n", ""));
    assert_eq!(refused.errors, ["the file has no Endpoint for its peer"]);
}
