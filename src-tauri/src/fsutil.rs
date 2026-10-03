use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

/// Writes `bytes` to `path` so a crash can never leave a half-written file behind.
///
/// The data goes to a sibling temp file (owner read/write only), is flushed to disk, and is then
/// renamed over the target, which is atomic on the same filesystem.
pub fn write_private_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = Path::new(&tmp);

    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let written = options.open(tmp).and_then(|mut file| {
        file.write_all(bytes)?;
        file.sync_all()
    });
    match written.and_then(|()| fs::rename(tmp, path)) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(tmp);
            Err(error)
        }
    }
}

#[cfg(test)]
#[path = "tests/fsutil.rs"]
mod tests;
