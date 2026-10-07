use super::*;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kage-applog-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_log_rotates_instead_of_growing_without_bound() {
    let dir = scratch("rotate");
    let path = dir.join(FILE);
    let file = OpenOptions::new().create(true).append(true).open(&path).unwrap();
    let mut sink = Sink { path: path.clone(), file, written: MAX_BYTES };

    rotate(&mut sink);

    assert_eq!(sink.written, 0);
    assert!(dir.join("app.log.1").is_file(), "the full file is kept as .1");
    assert_eq!(fs::metadata(&path).unwrap().len(), 0);
    let _ = fs::remove_dir_all(dir);
}
