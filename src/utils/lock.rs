use fs2::FileExt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct FileLock {
    path: PathBuf,
    lockfile: fs::File,
}

impl Drop for FileLock {
    fn drop(&mut self) {
        debug!("Dropping lock on file {}", self.path.to_string_lossy());

        self.lockfile.unlock().unwrap();
    }
}

pub fn lock_file_exclusive(path: &Path) -> io::Result<FileLock> {
    debug!("Locking on file ({})", path.to_string_lossy());

    let lockfile = fs::File::create(path)?;
    lockfile.try_lock_exclusive()?;

    Ok(FileLock {
        path: PathBuf::from(path),
        lockfile,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_lock() {
        let lockfile = NamedTempFile::new().unwrap();
        let lock = lock_file_exclusive(lockfile.path());
        assert!(lock.is_ok());
    }

    #[test]
    fn test_lock_fail() {
        let lockfile = NamedTempFile::new().unwrap();
        let lock = lock_file_exclusive(lockfile.path());
        let lock_err = lock_file_exclusive(lockfile.path());
        assert!(lock.is_ok());
        assert!(lock_err.is_err());
    }
}
