use super::*;

fn make_archive(dir: &Path, entry_name: &str) -> std::path::PathBuf {
    let path = dir.join("test.tar.gz");
    let encoder = flate2::write::GzEncoder::new(File::create(&path).unwrap(), flate2::Compression::fast());
    let mut builder = tar::Builder::new(encoder);
    let data = b"#!/bin/sh\necho hi\n";
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append_data(&mut header, entry_name, &data[..]).unwrap();
    builder.into_inner().unwrap().finish().unwrap();
    path
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("singbox-gui-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn extracts_the_binary_from_a_nested_folder() {
    let dir = scratch("extract");
    let archive = make_archive(&dir, "sing-box-1.0.0-darwin-arm64/sing-box");
    let out = dir.join("sing-box");
    extract_binary(&archive, &out).unwrap();
    assert!(out.is_file());
}

#[test]
fn fails_when_binary_is_missing() {
    let dir = scratch("missing");
    let archive = make_archive(&dir, "sing-box-1.0.0-darwin-arm64/LICENSE");
    assert!(extract_binary(&archive, &dir.join("sing-box")).is_err());
}

#[test]
fn hashes_files_like_sha256sum() {
    let dir = scratch("hash");
    let path = dir.join("f");
    fs::write(&path, b"abc").unwrap();
    assert_eq!(
        sha256_file(&path).unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
