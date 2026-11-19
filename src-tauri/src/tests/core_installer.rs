use super::*;
use std::io::Write;

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
    let dir = std::env::temp_dir().join(format!("kage-vpn-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn extracts_the_binary_from_a_nested_folder() {
    let dir = scratch("extract");
    let archive = make_archive(&dir, "sing-box-1.0.0-darwin-arm64/sing-box");
    let out = dir.join("sing-box");
    extract_binary("sing-box", &archive, &out).unwrap();
    assert!(out.is_file());
}

#[test]
fn fails_when_binary_is_missing() {
    let dir = scratch("missing");
    let archive = make_archive(&dir, "sing-box-1.0.0-darwin-arm64/LICENSE");
    assert!(extract_binary("sing-box", &archive, &dir.join("sing-box")).is_err());
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

#[test]
fn an_archive_that_unpacks_into_something_huge_is_stopped() {
    // A small download that expands far past what an engine binary can be (the limit is lower in
    // tests so the archive can be built quickly).
    let dir = scratch("bomb");
    let path = dir.join("bomb.tar.gz");
    let encoder = flate2::write::GzEncoder::new(File::create(&path).unwrap(), flate2::Compression::fast());
    let mut builder = tar::Builder::new(encoder);
    let mut header = tar::Header::new_gnu();
    header.set_size(MAX_BINARY_BYTES + 1);
    header.set_mode(0o755);
    header.set_cksum();
    // The data is a stream of zeros that compresses to almost nothing.
    builder.append_data(&mut header, "sing-box", std::io::repeat(0).take(MAX_BINARY_BYTES + 1)).unwrap();
    builder.into_inner().unwrap().finish().unwrap();
    assert!(fs::metadata(&path).unwrap().len() < 8 * 1024 * 1024, "the archive itself is small");

    let out = dir.join("sing-box");
    let error = extract_binary("sing-box", &path, &out).unwrap_err();
    assert!(error.to_string().contains("larger than expected"), "{error}");
    assert!(!out.exists() || fs::metadata(&out).unwrap().len() == 0, "nothing large was left behind");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_release_whose_version_could_leave_the_engine_folder_is_refused() {
    let release = |version: &str, url: &str| Release {
        version: version.into(),
        prerelease: false,
        published_at: None,
        notes: String::new(),
        size: 1,
        url: url.into(),
        sha256: Some("a".repeat(64)),
    };
    assert!(check_release(&release("1.14.2", "https://github.com/x")).is_ok());
    assert!(check_release(&release("1.14.2-rc.1", "https://github.com/x")).is_ok());
    for bad in ["1.0.0-../../../../tmp/x", "1.0.0./etc", "../1.0.0", "1.0.0-a/b", "1.0.0-a b", "latest"] {
        assert!(check_release(&release(bad, "https://github.com/x")).is_err(), "{bad} was accepted");
    }
    assert!(check_release(&release("1.14.2", "http://github.com/x")).is_err(), "plain http is refused");
}

fn make_zip(dir: &Path, entries: &[(&str, &[u8])]) -> std::path::PathBuf {
    let path = dir.join("test.zip");
    let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, data) in entries {
        zip.start_file(*name, options).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap();
    path
}

#[test]
fn extracts_the_windows_binary_from_a_zip() {
    let dir = scratch("zip");
    let archive = make_zip(&dir, &[("sing-box-1.0.0-windows-amd64/LICENSE", b"text"), ("sing-box-1.0.0-windows-amd64/sing-box.exe", b"MZ binary")]);
    let out = dir.join("sing-box.exe");
    extract_binary("sing-box.exe", &archive, &out).unwrap();
    assert_eq!(fs::read(&out).unwrap(), b"MZ binary");
}

#[test]
fn a_zip_without_the_binary_is_refused() {
    let dir = scratch("zip-missing");
    let archive = make_zip(&dir, &[("sing-box-1.0.0-windows-amd64/LICENSE", b"text")]);
    assert!(extract_binary("sing-box.exe", &archive, &dir.join("sing-box.exe")).is_err());
}

#[test]
fn a_zip_that_unpacks_into_something_huge_is_stopped() {
    let dir = scratch("zip-bomb");
    let zeros = vec![0u8; (MAX_BINARY_BYTES + 1) as usize];
    let archive = make_zip(&dir, &[("sing-box.exe", &zeros)]);
    let error = extract_binary("sing-box.exe", &archive, &dir.join("sing-box.exe")).unwrap_err();
    assert!(error.to_string().contains("larger than expected"), "{error}");
}

#[test]
fn the_download_keeps_the_extension_that_says_how_to_unpack_it() {
    assert_eq!(archive_name("https://x/sing-box-1.0.0-windows-amd64.zip"), "archive.zip");
    assert_eq!(archive_name("https://x/sing-box-1.0.0-darwin-arm64.tar.gz"), "archive.tar.gz");
}
