//! The optional root service: a launchd daemon that runs `helper.sh` as root, so TUN mode never
//! asks for the administrator password again. Installing and removing it takes one prompt each.
//! The script and the sing-box copy it runs live in root-owned folders, so a process running as the
//! user cannot replace what runs as root. The app talks to it with the same `helper.start` /
//! `stop` / `child.pid` files the per-launch helper uses.

use std::path::{Path, PathBuf};

use std::sync::atomic::AtomicBool;

use super::{run_admin_script, run_admin_script_until, shell_quote, HELPER};
use crate::error::Result;
use crate::paths::AppPaths;

pub const LABEL: &str = "com.apzur.kage.daemon";
const SCRIPT: &str = "/Library/PrivilegedHelperTools/com.apzur.kage.daemon.sh";
const BIN_DIR: &str = "/Library/Application Support/Kage VPN";
const BIN: &str = "/Library/Application Support/Kage VPN/sing-box";
const PLIST: &str = "/Library/LaunchDaemons/com.apzur.kage.daemon.plist";

const INSTALL_PROMPT: &str = "Kage VPN wants to install its VPN service. After this, connecting in TUN mode will not ask for your password again.";
const REMOVE_PROMPT: &str = "Kage VPN wants to remove its VPN service.";

pub fn installed() -> bool {
    Path::new(PLIST).exists() && Path::new(SCRIPT).exists() && Path::new(BIN).exists()
}

/// Whether the service's own copy of sing-box is the one the app wants to run now.
pub fn binary_current(binary: &Path) -> bool {
    let (Ok(ours), Ok(theirs)) = (std::fs::metadata(binary), std::fs::metadata(BIN)) else { return false };
    ours.len() == theirs.len() && ours.modified().ok() == theirs.modified().ok()
}

/// Whether the installed service runs the same script this version of the app ships. An app update
/// that changes the script (a fix, a new kill-switch rule) would otherwise never reach a service
/// installed earlier, and the two would drift apart without anyone noticing.
pub fn script_current() -> bool {
    script_matches(Path::new(SCRIPT))
}

fn script_matches(installed: &Path) -> bool {
    std::fs::read(installed).is_ok_and(|bytes| bytes == HELPER.as_bytes())
}

/// Installs (or refreshes) the service for `binary`, starting it with launchd.
/// `cancel` ends the password prompt early when the user stops connecting meanwhile.
pub async fn install(paths: &AppPaths, binary: &Path, cancel: Option<&AtomicBool>) -> Result<()> {
    let plist = plist(paths);
    let commands = [
        "set -e".to_string(),
        format!("mkdir -p '/Library/PrivilegedHelperTools' {}", shell_quote(BIN_DIR)),
        format!("launchctl bootout system/{LABEL} 2>/dev/null || true"),
        format!("printf %s {} > {}", shell_quote(HELPER), shell_quote(SCRIPT)),
        format!("printf %s {} > {}", shell_quote(&plist), shell_quote(PLIST)),
        format!("cp -p {} {}", shell_quote(&binary.display().to_string()), shell_quote(BIN)),
        format!("chown root:wheel {} {} {} {}", shell_quote(SCRIPT), shell_quote(PLIST), shell_quote(BIN), shell_quote(BIN_DIR)),
        format!("chmod 755 {} {} {}", shell_quote(SCRIPT), shell_quote(BIN), shell_quote(BIN_DIR)),
        format!("chmod 644 {}", shell_quote(PLIST)),
        format!("launchctl bootstrap system {}", shell_quote(PLIST)),
    ];
    run_admin_script_until(&commands.join("; "), INSTALL_PROMPT, cancel).await.map(|_| ())
}

pub async fn uninstall() -> Result<()> {
    let commands = [
        format!("launchctl bootout system/{LABEL} 2>/dev/null || true"),
        format!("rm -f {} {} {}", shell_quote(PLIST), shell_quote(SCRIPT), shell_quote(BIN)),
        format!("rmdir {} 2>/dev/null || true", shell_quote(BIN_DIR)),
    ];
    run_admin_script(&commands.join("; "), REMOVE_PROMPT).await.map(|_| ())
}

fn plist(paths: &AppPaths) -> String {
    let arguments: Vec<String> = ["/bin/sh", SCRIPT, BIN]
        .iter()
        .map(|s| s.to_string())
        .chain([paths.config_file(), paths.data_dir()].iter().map(|p: &PathBuf| p.display().to_string()))
        .chain(["-".to_string()])
        .collect();
    let items: String = arguments.iter().map(|a| format!("    <string>{}</string>\n", xml_escape(a))).collect();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n<dict>\n  <key>Label</key>\n  <string>{LABEL}</string>\n  <key>ProgramArguments</key>\n  <array>\n{items}  </array>\n  <key>RunAtLoad</key>\n  <true/>\n  <key>KeepAlive</key>\n  <true/>\n</dict>\n</plist>\n"
    )
}

fn xml_escape(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_installed_script_counts_as_current_only_when_it_matches_this_apps_copy() {
        let dir = std::env::temp_dir().join(format!("kage-script-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let installed = dir.join("daemon.sh");
        assert!(!script_matches(&installed), "nothing installed");
        std::fs::write(&installed, HELPER).unwrap();
        assert!(script_matches(&installed));
        std::fs::write(&installed, HELPER.replace("anchor=", "anchor2=")).unwrap();
        assert!(!script_matches(&installed), "an older script is noticed");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_install_command_writes_the_script_byte_for_byte() {
        // `script_current` relies on the root copy being identical, so the quoting used to write it
        // must survive apostrophes, quotes and newlines.
        let dir = std::env::temp_dir().join(format!("kage-script-quote-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("written.sh");
        let command = format!("printf %s {} > {}", shell_quote(HELPER), shell_quote(&target.display().to_string()));
        assert!(std::process::Command::new("/bin/sh").args(["-c", &command]).status().unwrap().success());
        assert_eq!(std::fs::read(&target).unwrap(), HELPER.as_bytes());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn plist_runs_the_root_script_for_this_data_dir_and_keeps_it_alive() {
        let dir = std::env::temp_dir().join("kage <&> test");
        let paths = AppPaths::new(dir.clone()).unwrap();
        let text = plist(&paths);
        assert!(text.contains(LABEL));
        assert!(text.contains("<key>KeepAlive</key>"));
        assert!(text.contains("kage &lt;&amp;&gt; test"));
        assert!(text.contains("<string>-</string>"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
