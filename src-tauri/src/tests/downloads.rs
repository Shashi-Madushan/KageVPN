use super::*;

#[test]
fn progress_events_use_the_names_the_ui_listens_for() {
    let json = serde_json::to_value(Progress {
        id: "core:sing-box".into(),
        kind: Kind::Core,
        title: "sing-box".into(),
        version: "1.12.9".into(),
        stage: Stage::Verify,
        received: 5,
        total: 10,
    })
    .unwrap();
    assert_eq!(json["id"], "core:sing-box");
    assert_eq!(json["kind"], "core");
    assert_eq!(json["stage"], "verify");
    assert_eq!(json["received"], 5);
}

#[test]
fn finished_events_carry_either_a_version_or_an_error() {
    let ok = serde_json::to_value(Finished {
        id: "app".into(),
        kind: Kind::App,
        title: "App".into(),
        version: Some("2.0.0".into()),
        error: None,
    })
    .unwrap();
    assert_eq!(ok["kind"], "app");
    assert!(ok["error"].is_null());
}

#[test]
fn cancel_is_a_flag_the_next_download_clears() {
    let downloads = Downloads::default();
    assert!(!downloads.is_cancelled() && !downloads.is_busy());
    downloads.cancel();
    assert!(downloads.is_cancelled());
}
