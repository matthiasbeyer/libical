use std::io::prelude::*;
use std::path::Path;
use std::{fs, io};

pub fn read_file_to_string(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
