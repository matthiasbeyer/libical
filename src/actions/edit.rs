use std::env;
use std::fs;
use std::process::Command;

use input;

pub fn do_edit(_args: &[String]) -> Result<(), String> {
  let khline = input::default_input_single()?;

  let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

  if let Err(error) = Command::new(&editor)
    .arg(khline.path.as_os_str())
    .stdin(fs::File::open("/dev/tty").unwrap())
    .status() {
      return Err(format!("{} command failed to start, error: {}", editor, error));
    };

  Ok(())
}
