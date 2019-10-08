use assert_fs::prelude::*;
use assert_fs::TempDir;
use assert_fs::fixture::{ChildPath, FixtureError};
use std::path::{PathBuf,Path};
use std::fs;

pub fn path_to(artifact: &str) -> PathBuf {
  [env!("CARGO_MANIFEST_DIR"), "testdata", artifact].iter().collect()
}

pub fn touch_testfile(testdir: &TempDir, relative_path: &Path) -> Result<ChildPath, FixtureError> {
  let testfile = testdir.child(relative_path);
  testfile.touch()?;
  Ok(testfile)
}
