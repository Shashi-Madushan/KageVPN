use super::*;

#[test]
fn quotes_paths_with_spaces_and_apostrophes() {
    assert_eq!(shell_quote("/a b/it's"), r"'/a b/it'\''s'");
}

#[test]
fn escapes_applescript_specials() {
    assert_eq!(applescript_escape(r#"a"b\c"#), r#"a\"b\\c"#);
}

mod helper {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("kage-helper-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A stand-in for sing-box that just stays alive.
    fn fake_core(dir: &Path) -> PathBuf {
        let path = dir.join("fake-core");
        fs::write(&path, "#!/bin/sh\nexec sleep 300\n").unwrap();
        Command::new("chmod").args(["+x", path.to_str().unwrap()]).status().unwrap();
        path
    }

    fn alive(pid: u32) -> bool {
        Command::new("kill").args(["-0", &pid.to_string()]).stderr(Stdio::null()).status().unwrap().success()
    }

    fn wait_for(what: &str, mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(8);
        while !condition() {
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            sleep(Duration::from_millis(100));
        }
    }

    fn start_helper(dir: &Path, core: &Path, parent: u32) -> Child {
        let script = dir.join("helper.sh");
        fs::write(&script, super::super::HELPER).unwrap();
        Command::new("/bin/sh")
            .args([script.to_str().unwrap(), core.to_str().unwrap(), "unused.json", dir.to_str().unwrap(), &parent.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
    }

    fn request_run(dir: &Path) {
        let _ = fs::remove_file(dir.join("child.pid"));
        fs::write(dir.join("helper.start"), "none").unwrap();
    }

    fn child_pid(dir: &Path) -> Option<u32> {
        fs::read_to_string(dir.join("child.pid")).ok()?.trim().parse().ok()
    }

    fn paths_in(dir: &Path) -> crate::paths::AppPaths {
        crate::paths::AppPaths::new(dir.to_path_buf()).unwrap()
    }

    fn block_on<T>(future: impl std::future::Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
    }

    #[test]
    fn a_helper_that_answers_gives_the_pid_of_the_run_it_started() {
        let dir = scratch("request");
        let core = fake_core(&dir);
        let mut helper = start_helper(&dir, &core, std::process::id());

        let pid = block_on(super::super::request_start(&paths_in(&dir), "none", Duration::from_secs(8))).unwrap().expect("an answer");
        assert!(alive(pid));
        assert_eq!(Some(pid), child_pid(&dir));

        fs::write(dir.join("helper.quit"), "").unwrap();
        wait_for("the helper to exit", || helper.try_wait().unwrap().is_some());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn a_helper_that_never_answers_gives_nothing_instead_of_hanging() {
        // What a service that was unloaded looks like: the request is written and nobody reads it.
        let dir = scratch("silent");
        let started = Instant::now();
        let answer = block_on(super::super::request_start(&paths_in(&dir), "none", Duration::from_millis(400))).unwrap();
        assert_eq!(answer, None);
        assert!(started.elapsed() < Duration::from_secs(3));
        assert!(dir.join("helper.start").exists(), "the request is left for a helper that starts later");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn one_helper_serves_connect_stop_and_reconnect_without_being_started_again() {
        let dir = scratch("reuse");
        let core = fake_core(&dir);
        let mut helper = start_helper(&dir, &core, std::process::id());

        request_run(&dir);
        wait_for("the first run", || child_pid(&dir).is_some());
        let first = child_pid(&dir).unwrap();
        assert!(alive(first));

        fs::write(dir.join("stop"), "").unwrap();
        wait_for("the first run to stop", || !alive(first));
        assert!(helper.try_wait().unwrap().is_none(), "the helper keeps running between connections");

        request_run(&dir);
        wait_for("the second run", || child_pid(&dir).is_some());
        let second = child_pid(&dir).unwrap();
        assert_ne!(first, second);
        assert!(alive(second));

        fs::write(dir.join("helper.quit"), "").unwrap();
        wait_for("the helper to exit", || helper.try_wait().unwrap().is_some());
        wait_for("the core to stop with it", || !alive(second));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn the_helper_and_its_core_end_when_the_app_does() {
        let dir = scratch("parent");
        let core = fake_core(&dir);
        let mut app = Command::new("sleep").arg("300").spawn().unwrap();
        let mut helper = start_helper(&dir, &core, app.id());

        request_run(&dir);
        wait_for("a run", || child_pid(&dir).is_some());
        let child = child_pid(&dir).unwrap();

        app.kill().unwrap();
        let _ = app.wait();
        wait_for("the helper to follow the app", || helper.try_wait().unwrap().is_some());
        wait_for("the core to follow the app", || !alive(child));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn the_root_service_outlives_the_app_but_its_core_does_not() {
        let dir = scratch("service");
        let core = fake_core(&dir);
        let mut app = Command::new("sleep").arg("300").spawn().unwrap();
        let script = dir.join("helper.sh");
        fs::write(&script, super::super::HELPER).unwrap();
        // "-" as the parent argument is how launchd starts the service.
        let mut service = Command::new("/bin/sh")
            .args([script.to_str().unwrap(), core.to_str().unwrap(), "unused.json", dir.to_str().unwrap(), "-"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let _ = fs::remove_file(dir.join("child.pid"));
        fs::write(dir.join("helper.start"), format!("none\n{}\n", app.id())).unwrap();
        wait_for("a run", || child_pid(&dir).is_some());
        let child = child_pid(&dir).unwrap();

        app.kill().unwrap();
        let _ = app.wait();
        wait_for("the core to follow the app", || !alive(child));
        assert!(service.try_wait().unwrap().is_none(), "the service itself keeps running");
        let _ = service.kill();
        let _ = service.wait();
        let _ = fs::remove_dir_all(dir);
    }
}
