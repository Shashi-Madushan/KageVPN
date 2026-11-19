use super::*;
use serde_json::json;

const SID: &str = "S-1-5-21-1004336348-1177238915-682003330-1001";

fn tun_config() -> Value {
    json!({ "inbounds": [{ "type": "mixed" }, { "type": "tun", "auto_route": true }], "log": { "output": "C:\\Users\\a\\x.log" } })
}

#[test]
fn a_start_request_round_trips_as_one_json_line() {
    let request = Request::Start { config: tun_config() };
    let line = serde_json::to_string(&request).unwrap();
    assert!(!line.contains('\n'));
    assert_eq!(parse_request(&line).unwrap(), request);
    assert_eq!(parse_request(r#"{"op":"stop"}"#).unwrap(), Request::Stop);
    assert!(parse_request("not json").is_err());
    assert!(parse_request(r#"{"op":"format_c"}"#).is_err());
}

#[test]
fn a_reply_leaves_out_what_it_does_not_say() {
    assert_eq!(serde_json::to_string(&Reply::ok()).unwrap(), r#"{"ok":true}"#);
    let error = serde_json::to_string(&Reply::fail("no")).unwrap();
    assert!(error.contains(r#""error":"no""#) && error.contains(r#""ok":false"#));
}

#[test]
fn a_normal_tun_configuration_is_accepted() {
    assert!(validate_config(&tun_config()).is_ok());
}

#[test]
fn a_configuration_without_a_tun_inbound_is_refused() {
    assert!(validate_config(&json!({ "inbounds": [{ "type": "mixed" }] })).is_err());
    assert!(validate_config(&json!([])).is_err());
}

#[test]
fn keys_that_make_system_touch_files_are_refused_anywhere_in_the_configuration() {
    for key in FORBIDDEN_KEYS {
        let mut config = tun_config();
        config["experimental"] = json!({ "nested": [{ key: "x" }] });
        let error = validate_config(&config).unwrap_err();
        assert!(error.contains(key), "{key}: {error}");
    }
}

#[test]
fn only_local_srs_rule_sets_with_absolute_paths_are_allowed() {
    let with = |set: Value| {
        let mut config = tun_config();
        config["route"] = json!({ "rule_set": [set] });
        validate_config(&config)
    };
    assert!(with(json!({ "type": "local", "path": "C:\\Users\\a\\rs\\geoip-lk.srs" })).is_ok());
    assert!(with(json!({ "type": "local", "path": "rs\\geoip-lk.srs" })).is_err());
    assert!(with(json!({ "type": "local", "path": "C:\\Users\\a\\..\\b\\x.srs" })).is_err());
    assert!(with(json!({ "type": "local", "path": "C:\\Windows\\System32\\config\\SAM" })).is_err());
    assert!(with(json!({ "type": "remote", "url": "https://x/y.srs" })).is_err());
}

#[test]
fn the_service_replaces_the_log_path_with_its_own() {
    let config = with_service_log(tun_config(), r"C:\ProgramData\Kage\run\sing-box.log");
    assert_eq!(config["log"]["output"], r"C:\ProgramData\Kage\run\sing-box.log");
}

#[test]
fn sids_are_checked_before_they_reach_a_descriptor() {
    assert!(is_sid(SID));
    assert!(is_sid("S-1-5-18"));
    assert!(!is_sid("S-1-5"));
    assert!(!is_sid("S-1-5-18)(A;;GA;;;WD"));
    assert!(!is_sid(""));
    assert!(pipe_sddl("x").is_none() && service_sddl("x").is_none());
}

#[test]
fn the_pipe_lets_in_only_system_administrators_and_the_installing_user() {
    let sddl = pipe_sddl(SID).unwrap();
    assert_eq!(sddl, format!("D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GRGW;;;{SID})"));
    assert!(!sddl.contains("WD") && !sddl.contains(";;;BU") && !sddl.contains(";;;IU"));
}

#[test]
fn the_user_sid_is_read_from_whoami_output() {
    assert_eq!(parse_whoami_sid(&format!("\"DESKTOP\\shashi\",\"{SID}\"\r\n")).as_deref(), Some(SID));
    assert_eq!(parse_whoami_sid("garbage"), None);
}

#[test]
fn the_install_script_copies_checks_and_registers() {
    let script = install_script(r"C:\Users\a\AppData\Local\Kage\kage-vpn.exe", r"C:\Users\a\it's\sing-box.exe", "ABCDEF", SID).unwrap();
    assert!(script.contains("'C:\\Users\\a\\it''s\\sing-box.exe'"), "quotes are doubled");
    assert!(script.contains("'abcdef'") && script.contains("the engine changed while it was being copied"));
    assert!(script.contains(&format!("--service --user-sid {SID}")));
    assert!(script.contains("icacls.exe $d /inheritance:r") && script.contains("sdset"));
    assert!(install_script("a", "b", "c", "S-1-5").is_err());
}

#[test]
fn the_uninstall_script_removes_the_service_and_the_folder() {
    let script = uninstall_script();
    assert!(script.contains("sc.exe delete") && script.contains("Remove-Item"));
}

#[test]
fn a_script_is_encoded_as_utf16_base64() {
    use base64::Engine;
    let encoded = encode_script("Write-Host 'é'");
    let bytes = base64::engine::general_purpose::STANDARD.decode(encoded).unwrap();
    let units: Vec<u16> = bytes.chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    assert_eq!(String::from_utf16(&units).unwrap(), "Write-Host 'é'");
}
