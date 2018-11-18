use std::path::{Path};

use icalwrap::{IcalComponent,IcalVCalendar,IcalProperty};
use utils;

pub fn shortprint_dir(dir: &Path) {
  for filepath in utils::file_iter(dir) {
    shortprint_file(&filepath);
  }
}

pub fn shortprint_file(filepath: &Path) {
  match utils::read_file_to_string(filepath) {
    Ok(content) => {
      let comp = IcalVCalendar::from_str(&content, None);
      let inner = comp.unwrap();
      shortprint_comp(&inner);
    },
    Err(error) => print!("{}", error)
  }
}

pub fn prettyprint_file(filepath: &Path) {
  match utils::read_file_to_string(filepath) {
    Ok(content) => {
      let comp = IcalVCalendar::from_str(&content, Some(filepath.to_path_buf())).unwrap();
      prettyprint_comp(&comp);
    },
    Err(error) => print!("{}", error)
  }
}

pub fn shortprint_comp(comp: &IcalVCalendar) {
  let event = comp.events_iter().next().expect("No event in VCalendar!");
  let date = event.get_dtstart().format("%Y-%m-%d");
  let description = event.get_summary().unwrap_or(String::from("?"));
  println!("{} {}", date, description);
}

pub fn prettyprint_comp(comp: &IcalVCalendar) {
  let properties = comp.get_properties_all();
  println!("num: {}", properties.len());
  for property in properties {
    prettyprint_prop(&property);
  }
}

fn prettyprint_prop(property: &IcalProperty) {
  let name = property.get_name();
  let value = property.get_value();
  match name.as_str() {
    "DTSTART" => {
      let date = property.get_value_as_date();
      println!("start: {}", date);
    },
    "DESCRIPTION" => println!("description: {}", value),
    _  => println!("{} - {}", name, value),
  }
}

