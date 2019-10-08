use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::{fs, io};
use std::fs::OpenOptions;

use crate::IcalVCalendar;

pub fn file_iter(dir: &Path) -> impl Iterator<Item = PathBuf> {
  use walkdir::WalkDir;

  WalkDir::new(dir)
    .follow_links(true)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_file())
    .map(|entry| entry.into_path())
}

pub fn dir_iter(dir: &Path) -> impl Iterator<Item = PathBuf> {
  use walkdir::WalkDir;

  let dir = dir.to_path_buf();
  WalkDir::new(&dir)
    .follow_links(true)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_dir())
    .filter(move |f| f.path() != dir)
    .map(|entry| entry.into_path())
}

pub fn write_file(filepath: &Path, contents: &str) -> io::Result<()> {
  let mut file = fs::File::create(filepath)?;
  file.write_all(contents.as_bytes())
}

pub fn append_file(filepath: &Path, contents: &str) -> io::Result<()> {
  let mut file = OpenOptions::new()
              .append(true)
              .create(true)
              .open(filepath)?;
  file.write_all(contents.as_bytes())
}

pub fn read_lines_from_file(filepath: &Path) -> io::Result<impl DoubleEndedIterator<Item = String>> {
  let f = fs::File::open(filepath)?;
  let f = BufReader::new(f);
  let lines: Result<Vec<String>, io::Error> = f.lines().collect();
  lines.map(|result| result.into_iter())
}

pub fn read_file_to_string(path: &Path) -> io::Result<String> {
  let mut file = fs::File::open(&path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  Ok(contents)
}

