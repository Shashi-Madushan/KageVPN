use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

/// Options for a file only its owner may read: new files get mode 0600. Logs and the usage history
/// list the sites a person visited, so they are never left readable by other accounts on the Mac.
pub fn private_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

/// Opens `path` for appending, creating it owner-only. A file that already exists is tightened too.
pub fn open_private_append(path: &Path) -> io::Result<fs::File> {
    let file = private_options().create(true).append(true).open(path)?;
    restrict(path);
    Ok(file)
}

/// Creates `path` empty (or empties it), owner-only.
pub fn create_private(path: &Path) -> io::Result<fs::File> {
    let file = private_options().create(true).write(true).truncate(true).open(path)?;
    restrict(path);
    Ok(file)
}

/// Makes an existing file owner-only; best effort, since a file made by root cannot be changed.
pub fn restrict(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// Makes a folder reachable by its owner only (mode 0700), best effort.
pub fn restrict_dir(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o700));
    }
    #[cfg(not(unix))]
    let _ = path;
}

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
