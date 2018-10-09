extern crate chrono;
extern crate libc;

use chrono::{Datelike, Duration, NaiveTime};
use icalwrap::*;
use std::collections::HashMap;
use std::fs;
use utils;

fn get_buckets(comp: &mut Icalcomponent) -> Vec<String> {
  let mut buckets: Vec<String> = comp
    .map(|x| {
      let mut start_date = x.get_dtstart();
      let mut end_date = x.get_dtend();
      //info!("start: {}", start_date);
      //info!("end: {}", end_date);
      // end-dtimes are non-inclusive 
      // so in case of date-only events, the dtend given is one day after the last day of the event
      if end_date.time() == NaiveTime::from_hms(0, 0, 0) {
        end_date = end_date.checked_sub_signed(Duration::days(1)).unwrap();
      }
      let mut buckets = Vec::new();
      while start_date.iso_week() <= end_date.iso_week() {
        let bucket = format!(
          "{}-{:02}",
          start_date.iso_week().year(),
          start_date.iso_week().week()
        );
        buckets.push(bucket);
        start_date = start_date.checked_add_signed(Duration::days(7)).unwrap();
      }
      //if buckets.len() > 1 {
      //  info!("{}: {} buckets", x.get_uid(), buckets.len());
      //}
      buckets
    }).flatten()
    .collect();
  buckets.sort();
  buckets.dedup();
  buckets
}

fn add_buckets_for_component(buckets: &mut HashMap<String, Vec<String>>, comp: &mut Icalcomponent) {
  let comp_buckets = get_buckets(comp);
  for bucketid in comp_buckets {
    buckets
      .entry(bucketid)
      .and_modify(|items| items.push(comp.get_uid()))
      .or_insert(::utils::vec_from_string(comp.get_uid()));
  }
}

pub fn index_dir(dir: &str) {
  let mut buckets: HashMap<String, Vec<String>> = HashMap::new();

  if let Ok(entries) = fs::read_dir(dir) {
    for entry in entries {
      if let Ok(entry) = entry {
        if ! entry.path().is_file() {
          continue;
        }
        if entry
          .path()
          .extension()
          .map_or(false, |extension| extension == "ics")
        {
          match utils::read_file_to_string(&entry.path()) {
            Ok(content) => {
              let mut comp = Icalcomponent::from_str(&content);
              add_buckets_for_component(&mut buckets, &mut comp);
            }
            Err(error) => error!("{}", error),
          }
        }
      }
    }
  }
  info!("{} buckets", buckets.len());
  for (key, val) in buckets.iter() {
    if let Err(error) = utils::write_file(key, val.join("\n")) {
      error!("{}", error);
    }
  }
}

#[test]
fn buckets_multi_day_allday() {
  let event_str = "BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VEVENT
UID:20070423T123432Z-541111@example.com
DTSTAMP:20070423T123432Z
DTSTART;VALUE=DATE:20070628
DTEND;VALUE=DATE:20070709
SUMMARY:Festival International de Jazz de Montreal
TRANSP:TRANSPARENT
END:VEVENT
END:VCALENDAR";

  let mut comp = Icalcomponent::from_str(event_str);
  let comp_buckets = get_buckets(&mut comp);
  assert_eq!(comp_buckets, ["2007-26", "2007-27"]);
}
