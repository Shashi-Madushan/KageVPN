use super::*;

#[test]
fn the_program_is_read_from_the_display_icon() {
    assert_eq!(exe_from_display_icon(r#""C:\Program Files\Slack\slack.exe",0"#), Some(PathBuf::from(r"C:\Program Files\Slack\slack.exe")));
    assert_eq!(exe_from_display_icon(r"C:\Program Files\App\App.EXE,1"), Some(PathBuf::from(r"C:\Program Files\App\App.EXE")));
    assert_eq!(exe_from_display_icon(r"C:\Program Files\App\app.exe"), Some(PathBuf::from(r"C:\Program Files\App\app.exe")));
    assert_eq!(exe_from_display_icon(r"C:\Program Files\App\app.ico"), None);
    assert_eq!(exe_from_display_icon(""), None);
}

#[test]
fn uninstallers_and_updaters_are_skipped() {
    assert!(is_helper(Path::new(r"C:\App\unins000.exe")));
    assert!(is_helper(Path::new(r"C:\App\Update.exe")));
    assert!(!is_helper(Path::new(r"C:\App\slack.exe")));
}

#[test]
fn an_app_is_its_program_folder() {
    assert_eq!(app_folder(Path::new(r"C:\Program Files\Slack\slack.exe")).as_deref(), Some(r"C:\Program Files\Slack"));
    assert_eq!(folder_of_process(r"C:\Program Files\Slack\slack.exe"), Some(r"C:\Program Files\Slack"));
    assert_eq!(folder_of_process("/Applications/Slack.app/Contents/MacOS/Slack"), None);
    assert_eq!(folder_of_process(r"C:\x.exe"), None);
}
