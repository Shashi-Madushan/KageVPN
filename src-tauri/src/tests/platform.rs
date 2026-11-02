use super::*;

#[test]
#[cfg(target_os = "macos")]
fn macos_offers_everything() {
    assert_eq!(platform(), Platform { os: "macos", tun: true, kill_switch: true, root_service: true, system_proxy: true, app_nap: true });
}

#[test]
#[cfg(windows)]
fn windows_starts_with_proxy_only() {
    let p = platform();
    assert_eq!(p.os, "windows");
    assert!(p.system_proxy && !p.tun && !p.kill_switch && !p.app_nap);
}

#[test]
fn the_front_end_reads_camel_case_names() {
    let json = serde_json::to_value(platform()).unwrap();
    assert!(json.get("killSwitch").is_some() && json.get("rootService").is_some() && json.get("appNap").is_some(), "{json}");
}
