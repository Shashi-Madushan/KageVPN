use super::*;

fn bundle(dir: &Path, name: &str, plist: Option<&str>) -> PathBuf {
    let bundle = dir.join(name);
    std::fs::create_dir_all(bundle.join("Contents")).unwrap();
    if let Some(body) = plist {
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>{body}</dict></plist>"#
        );
        std::fs::write(bundle.join("Contents/Info.plist"), xml).unwrap();
    }
    bundle
}

#[test]
fn scan_reads_names_from_the_bundle_and_looks_one_folder_deep() {
    let root = std::env::temp_dir().join(format!("kage-apps-{}", uuid::Uuid::new_v4()));
    bundle(&root, "Slack.app", Some("<key>CFBundleName</key><string>Slack</string><key>CFBundleIdentifier</key><string>com.tinyspeck.slackmacgap</string>"));
    bundle(&root, "zoom.us.app", Some("<key>CFBundleDisplayName</key><string>Zoom</string><key>CFBundleName</key><string>zoom.us</string>"));
    // No Info.plist: the folder name is used.
    bundle(&root, "Broken.app", None);
    // Apps inside a vendor folder are found; deeper ones and plain folders are not listed.
    bundle(&root.join("Setapp"), "CleanShot X.app", Some("<key>CFBundleName</key><string>CleanShot X</string>"));
    bundle(&root.join("Setapp/Nested"), "TooDeep.app", None);
    std::fs::write(root.join("notes.txt"), "not an app").unwrap();

    let apps = scan(&[root.clone(), root.join("missing")]);
    let names: Vec<&str> = apps.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["Broken", "CleanShot X", "Slack", "Zoom"]);

    let slack = apps.iter().find(|a| a.name == "Slack").unwrap();
    assert_eq!(slack.path, root.join("Slack.app").to_string_lossy());
    assert_eq!(slack.bundle_id.as_deref(), Some("com.tinyspeck.slackmacgap"));

    // A root listed twice does not list its apps twice.
    assert_eq!(scan(&[root.clone(), root.clone()]).len(), 4);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_process_belongs_to_its_outermost_app() {
    assert_eq!(bundle_of("/Applications/Slack.app/Contents/MacOS/Slack"), Some("/Applications/Slack.app"));
    assert_eq!(
        bundle_of("/Applications/Google Chrome.app/Contents/Frameworks/Google Chrome Framework.framework/Versions/1/Helpers/Google Chrome Helper.app/Contents/MacOS/Google Chrome Helper"),
        Some("/Applications/Google Chrome.app")
    );
    assert_eq!(bundle_of("/usr/bin/curl"), None);
    assert_eq!(bundle_of(""), None);
    assert_eq!(bundle_of("Slack.app/Contents/MacOS/Slack"), None);
    // Non-ASCII names are cut on a character boundary.
    assert_eq!(bundle_of("/Applications/Café.app/Contents/MacOS/Café"), Some("/Applications/Café.app"));
}

#[test]
fn the_icon_file_name_gets_its_ending() {
    let dict = |value: &str| {
        let mut d = plist::Dictionary::new();
        d.insert("CFBundleIconFile".into(), plist::Value::String(value.into()));
        d
    };
    assert_eq!(icon_file_name(&dict("AppIcon")).as_deref(), Some("AppIcon.icns"));
    assert_eq!(icon_file_name(&dict("AppIcon.icns")).as_deref(), Some("AppIcon.icns"));
    assert_eq!(icon_file_name(&dict("Logo.ICNS")).as_deref(), Some("Logo.ICNS"));
    assert_eq!(icon_file_name(&dict("  ")), None);
    assert_eq!(icon_file_name(&plist::Dictionary::new()), None);
}

#[test]
fn no_icon_for_a_path_that_is_not_an_app() {
    let cache = std::env::temp_dir().join("kage-icon-test");
    assert_eq!(icon_data_url(Path::new("/usr/bin"), &cache), None);
    assert_eq!(icon_data_url(Path::new("/Applications/Does Not Exist.app"), &cache), None);
    assert_eq!(cache_name(Path::new("/Applications/Slack.app")), "_Applications_Slack_app.png");
}

#[test]
fn a_real_app_icon_comes_back_as_a_png() {
    // Safari ships with every Mac; skip where it is missing.
    let safari = Path::new("/Applications/Safari.app");
    if !safari.is_dir() {
        return;
    }
    let cache = std::env::temp_dir().join(format!("kage-icon-real-{}", std::process::id()));
    let url = icon_data_url(safari, &cache).expect("an icon");
    assert!(url.starts_with("data:image/png;base64,"));
    assert!(icon_data_url(safari, &cache).is_some(), "second call uses the cache");
    std::fs::remove_dir_all(cache).unwrap();
}
