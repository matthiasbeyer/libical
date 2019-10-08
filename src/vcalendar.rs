use std::ffi::{CStr, CString};
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::IcalComponent;
use super::IcalTime;
use super::IcalVEvent;

pub struct IcalVCalendar {
  comp: Rc<IcalComponentOwner>,
  path: Option<PathBuf>,
  instance_timestamp: Option<IcalTime>,
}

pub struct IcalEventIter<'a> {
  iter: ical::icalcompiter,
  parent: &'a IcalVCalendar,
}

impl IcalComponent for IcalVCalendar {
  fn get_ptr(&self) -> *mut ical::icalcomponent {
    self.comp.ptr
  }

  fn as_component(&self) -> &dyn IcalComponent {
    self
  }
}

impl Clone for IcalVCalendar {
  fn clone(&self) -> Self {
    let new_comp_ptr = unsafe { ical::icalcomponent_new_clone(self.comp.ptr) };
    let mut new_calendar = IcalVCalendar::from_ptr(new_comp_ptr);
    new_calendar.path = self.path.clone();
    new_calendar.instance_timestamp = self.instance_timestamp.clone();
    new_calendar
  }
}

impl IcalVCalendar {
  fn from_ptr(ptr: *mut ical::icalcomponent) -> Self {
    IcalVCalendar {
      comp: Rc::new(IcalComponentOwner { ptr }),
      path: None,
      instance_timestamp: None,
    }
  }

  pub fn shallow_copy(&self) -> Self {
    IcalVCalendar {
      comp: self.comp.clone(),
      path: self.path.clone(),
      instance_timestamp: self.instance_timestamp.clone(),
    }
  }

  //pub fn with_internal_timestamp(mut self, datetime: &IcalTime) -> IcalVCalendar {
  //self.instance_timestamp = Some(datetime.clone());
  //self
  //}

  pub fn with_path(mut self, path: &Path) -> IcalVCalendar {
    self.path = Some(path.to_path_buf());
    self
  }

  //TODO should probably be private
  pub fn from_str(str: &str, path: Option<&Path>) -> io::Result<Self> {
    unsafe {
      let c_str = CString::new(str).unwrap();
      let parsed_cal = ical::icalparser_parse_string(c_str.as_ptr());
      if parsed_cal.is_null() {
        return Err(io::Error::new(io::ErrorKind::Other, "calendar has no path"));
      }

      let kind = ical::icalcomponent_isa(parsed_cal);
      if kind != ical::icalcomponent_kind_ICAL_VCALENDAR_COMPONENT {
        let kind = CStr::from_ptr(ical::icalcomponent_kind_to_string(kind)).to_string_lossy();
        return Err(io::Error::new(
          io::ErrorKind::Other,
          format!("expected VCALENDAR component, got {}", kind),
        ));
      }

      let mut cal = IcalVCalendar::from_ptr(parsed_cal);
      cal.path = path.map(|path| path.to_path_buf());

      Ok(cal)
    }
  }

  pub fn with_normalize(self) -> Self {
    unsafe {
      ical::icalcomponent_normalize(self.get_ptr());
    }
    self
  }

  pub fn to_string(&self) -> String {
    unsafe {
      let ical_cstr = CStr::from_ptr(ical::icalcomponent_as_ical_string(self.get_ptr()));
      ical_cstr.to_string_lossy().into_owned()
    }
  }

  pub fn get_uid(&self) -> String {
    unsafe {
      let uid_cstr = CStr::from_ptr(ical::icalcomponent_get_uid(
        self.get_principal_event().get_ptr(),
      ));
      uid_cstr.to_string_lossy().into_owned()
    }
  }

  pub fn with_uid(mut self, uid: &str) -> Result<Self, String> {
    {
      let events = self.events_iter();
      if events.unique_uid_count() > 1 {
        return Err(format!(
          "More than one event in file: {}",
          self.get_path_as_string().unwrap_or_else(|| "".to_string())
        ));
      }
      let events = self.events_iter();
      let uid_cstr = CString::new(uid).unwrap();
      for event in events {
        unsafe {
          ical::icalcomponent_set_uid(event.get_ptr(), uid_cstr.as_ptr());
        }
      }
    }
    self.path = self
      .path
      .map(|path| path.with_file_name(uid.to_owned() + ".ics"));
    Ok(self)
  }

  pub fn with_dtstamp_now(self) -> Self {
    let dtstamp = IcalTime::utc();
    unsafe {
      ical::icalcomponent_set_dtstamp(self.get_ptr(), *dtstamp);
    }
    self
  }

  pub fn with_dtstart(self, dtstart: &IcalTime) -> Self {
    let event = self.get_principal_event();
    unsafe {
      let timezone = event.get_dtstart().and_then(|x| x.get_timezone());
      match timezone {
        Some(timezone) => {
          ical::icalcomponent_set_dtstart(event.get_ptr(), *dtstart.with_timezone(&timezone));
        }
        None => {
          ical::icalcomponent_set_dtstart(event.get_ptr(), **dtstart);
        }
      }
    }
    self
  }

  pub fn with_dtend(self, dtend: &IcalTime) -> Self {
    let event = self.get_principal_event();
    unsafe {
      let timezone = event.get_dtend().and_then(|x| x.get_timezone());
      match timezone {
        Some(timezone) => {
          ical::icalcomponent_set_dtend(event.get_ptr(), *dtend.with_timezone(&timezone));
        }
        None => {
          ical::icalcomponent_set_dtend(event.get_ptr(), **dtend);
        }
      }
    }
    self
  }

  pub fn with_location(self, location: &str) -> Self {
    let event = self.get_principal_event();
    unsafe {
      let c_str = CString::new(location).unwrap();
      ical::icalcomponent_set_location(event.get_ptr(), c_str.as_ptr());
    }
    self
  }

  pub fn with_summary(self, summary: &str) -> Self {
    let event = self.get_principal_event();
    unsafe {
      let c_str = CString::new(summary).unwrap();
      ical::icalcomponent_set_summary(event.get_ptr(), c_str.as_ptr());
    }
    self
  }

  pub fn with_last_modified_now(self) -> Self {
    let event = self.get_principal_event();
    unsafe {
      let now_icaltime = IcalTime::utc();

      let last_modified_kind = ical::icalproperty_kind_ICAL_LASTMODIFIED_PROPERTY;
      if let Some(prop) = event.get_property(last_modified_kind) {
        ical::icalproperty_set_lastmodified(prop.ptr, *now_icaltime);
      } else {
        let prop_lastmod = ical::icalproperty_new_lastmodified(*now_icaltime);
        ical::icalcomponent_add_property(event.get_ptr(), prop_lastmod);
      }
    }
    self
  }

  pub fn with_remove_property(self, property_name: &str) -> (Self, usize) {
    let property_kind = unsafe {
      let c_str = CString::new(property_name).unwrap();
      ical::icalproperty_string_to_kind(c_str.as_ptr())
    };

    let count = unsafe { IcalComponent::remove_property_all(&self, property_kind) };
    (self, count)
  }

  pub fn with_keep_uid(self, uid_to_keep: &str) -> Self {
    unsafe {
      ical::icalcomponent_get_first_component(
        self.comp.ptr,
        ical::icalcomponent_kind_ICAL_ANY_COMPONENT,
      );

      loop {
        let comp = ical::icalcomponent_get_current_component(self.comp.ptr);
        if comp.is_null() {
          return self;
        }
        let uid_ptr = ical::icalcomponent_get_uid(comp);
        if !uid_ptr.is_null() {
          let uid = CStr::from_ptr(uid_ptr).to_string_lossy();
          if uid != uid_to_keep {
            ical::icalcomponent_remove_component(self.comp.ptr, comp);
            continue;
          }
        }
        ical::icalcomponent_get_next_component(
          self.comp.ptr,
          ical::icalcomponent_kind_ICAL_ANY_COMPONENT,
        );
      }
    }
  }

  pub fn get_path_as_string(&self) -> Option<String> {
    self.path.as_ref().map(|path| format!("{}", path.display()))
  }

  pub fn get_path(&self) -> Option<&PathBuf> {
    self.path.as_ref()
  }

  pub fn get_calendar_name(&self) -> Option<String> {
    let calendar_name = self.path.as_ref()?.parent()?.file_name()?;
    Some(calendar_name.to_string_lossy().into_owned())
  }

  pub fn events_iter(&self) -> IcalEventIter {
    IcalEventIter::from_vcalendar(self)
  }

  pub fn get_first_event(&self) -> IcalVEvent {
    let event = unsafe {
      ical::icalcomponent_get_first_component(
        self.get_ptr(),
        ical::icalcomponent_kind_ICAL_VEVENT_COMPONENT,
      )
    };
    if self.events_iter().unique_uid_count() > 1 {
      warn!(
        "More than one event in file: {}",
        self.get_path_as_string().unwrap_or_else(|| "".to_string())
      )
    }
    IcalVEvent::from_ptr_with_parent(event, self)
  }

  pub(crate) fn get_principal_event(&self) -> IcalVEvent {
    let mut event = self.get_first_event();
    if let Some(ref timestamp) = self.instance_timestamp {
      event = event.with_internal_timestamp(timestamp)
    }
    event
  }

  pub fn check_for_errors(&self) -> Option<Vec<String>> {
    unsafe { IcalVCalendar::check_icalcomponent(self.get_ptr()) }
  }

  /// to be used after parsing, parser adds X-LIC-ERROR properties for any error
  /// ical::icalrestriction_check() checks if the specification is violated and adds X-LIC-ERRORs accordingly
  /// ical::icalcomponent_count_errors() counts all X-LIC-ERROR properties
  unsafe fn check_icalcomponent(comp: *mut ical::icalcomponent) -> Option<Vec<String>> {
    ical::icalrestriction_check(comp);
    let error_count = ical::icalcomponent_count_errors(comp);
    let mut output: Vec<String> = Vec::new();
    if error_count > 0 {
      output.append(&mut IcalVCalendar::get_errors(comp));

      let mut inner_comp =
        ical::icalcomponent_get_first_component(comp, ical::icalcomponent_kind_ICAL_ANY_COMPONENT);
      while !inner_comp.is_null() {
        output.append(&mut IcalVCalendar::get_errors(inner_comp));
        inner_comp =
          ical::icalcomponent_get_next_component(comp, ical::icalcomponent_kind_ICAL_ANY_COMPONENT)
      }
    }
    if let Some(errstring) = IcalVCalendar::check_uid(comp) {
      output.push(errstring);
    }
    if output.is_empty() {
      None
    } else {
      Some(output)
    }
  }

  unsafe fn check_uid(comp: *mut ical::icalcomponent) -> Option<String> {
    let uid = ical::icalcomponent_get_uid(comp);
    if uid.is_null() {
      Some("missing required property: UID".to_string())
    } else {
      None
    }
  }

  unsafe fn get_errors(comp: *mut ical::icalcomponent) -> Vec<String> {
    let mut prop =
      ical::icalcomponent_get_first_property(comp, ical::icalproperty_kind_ICAL_XLICERROR_PROPERTY);
    let mut output: Vec<String> = Vec::new();
    while !prop.is_null() {
      let error_cstr = CStr::from_ptr(ical::icalproperty_get_xlicerror(prop))
        .to_str()
        .unwrap();
      output.push(error_cstr.to_owned());
      prop = ical::icalcomponent_get_next_property(
        comp,
        ical::icalproperty_kind_ICAL_XLICERROR_PROPERTY,
      );
    }
    output
  }
}

impl<'a> IcalEventIter<'a> {
  fn from_vcalendar(cal: &'a IcalVCalendar) -> Self {
    let vevent_kind = ical::icalcomponent_kind_ICAL_VEVENT_COMPONENT;
    let iter = unsafe { ical::icalcomponent_begin_component(cal.get_ptr(), vevent_kind) };
    IcalEventIter { iter, parent: &cal }
  }

  fn unique_uid_count(self) -> usize {
    let mut uids = self.map(|event| event.get_uid()).collect::<Vec<String>>();
    uids.sort_unstable();
    uids.dedup();
    uids.len()
  }
}

impl<'a> Iterator for IcalEventIter<'a> {
  type Item = IcalVEvent;

  fn next(&mut self) -> Option<Self::Item> {
    unsafe {
      let ptr = ical::icalcompiter_deref(&mut self.iter);
      if ptr.is_null() {
        None
      } else {
        ical::icalcompiter_next(&mut self.iter);
        let vevent = IcalVEvent::from_ptr_with_parent(ptr, self.parent);
        Some(vevent)
      }
    }
  }
}

struct IcalComponentOwner {
  ptr: *mut ical::icalcomponent,
}

impl Drop for IcalComponentOwner {
  fn drop(&mut self) {
    unsafe {
      // println!("free");
      ical::icalcomponent_free(self.ptr);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::testdata;
  use chrono::{Local, TimeZone};

  #[test]
  fn test_from_str_empty() {
    assert!(IcalVCalendar::from_str("", None).is_err());
  }

  #[test]
  fn test_from_str_event() {
    assert!(IcalVCalendar::from_str(testdata::TEST_BARE_EVENT, None).is_err());
  }

  #[test]
  fn event_iterator_element_count() {
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();
    assert_eq!(cal.events_iter().count(), 1)
  }

  #[test]
  fn event_iterator_element_count_with_other() {
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_WITH_TIMEZONE_COMPONENT, None).unwrap();
    assert_eq!(cal.events_iter().count(), 1)
  }

  #[test]
  fn load_serialize() {
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();
    let back = cal.to_string().replace("\r\n", "\n");
    assert_eq!(back.trim(), testdata::TEST_EVENT_MULTIDAY)
  }

  #[test]
  fn load_serialize_with_error() {
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_WITH_X_LIC_ERROR, None).unwrap();
    let back = cal.to_string().replace("\r\n", "\n");
    assert_eq!(back.trim(), testdata::TEST_EVENT_WITH_X_LIC_ERROR)
  }

  #[test]
  fn with_dtstamp_test() {
    let mut cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();
    cal = cal.with_dtstamp_now();
    let event = cal.get_principal_event();

    let dtstamp_prop = ical::icalproperty_kind_ICAL_DTSTAMP_PROPERTY;
    let now_dtstamp = event.get_property(dtstamp_prop).unwrap().get_value();
    assert_eq!(now_dtstamp, "20130101T010203Z")
  }

  #[test]
  fn get_calendar_name_test() {
    let path = PathBuf::from("calname/event");
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY_ALLDAY, Some(&path)).unwrap();
    assert_eq!("calname".to_string(), cal.get_calendar_name().unwrap())
  }

  #[test]
  fn test_get_all_properties_cal() {
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();

    let props = cal.get_properties_all();
    assert_eq!(2, props.len());
  }

  #[test]
  fn parse_checker_test_empty_summary() {
    let c_str = CString::new(testdata::TEST_EVENT_EMPTY_SUMMARY).unwrap();
    unsafe {
      let parsed_cal = ical::icalparser_parse_string(c_str.as_ptr());
      assert!(IcalVCalendar::check_icalcomponent(parsed_cal).is_some())
    }
  }

  #[test]
  fn parse_checker_test_no_uid() {
    let c_str = CString::new(testdata::TEST_EVENT_NO_UID).unwrap();
    unsafe {
      let parsed_cal = ical::icalparser_parse_string(c_str.as_ptr());
      assert!(IcalVCalendar::check_icalcomponent(parsed_cal).is_some())
    }
  }

  #[test]
  fn parse_checker_test_no_prodid() {
    let c_str = CString::new(testdata::TEST_EVENT_NO_PRODID).unwrap();
    unsafe {
      let parsed_cal = ical::icalparser_parse_string(c_str.as_ptr());
      assert!(IcalVCalendar::check_icalcomponent(parsed_cal).is_some())
    }
  }

  #[test]
  fn test_with_last_modified_now() {
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY_LASTMODIFIED, None).unwrap();

    let new_cal = cal.with_last_modified_now();
    let event = new_cal.get_principal_event();

    let last_modified_kind = ical::icalproperty_kind_ICAL_LASTMODIFIED_PROPERTY;
    assert_eq!(
      "20130101T010203Z",
      event.get_property(last_modified_kind).unwrap().get_value()
    );
  }

  #[test]
  fn test_with_last_modified_now_added() {
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();

    let new_cal = cal.with_last_modified_now();
    let event = new_cal.get_principal_event();

    let last_modified_kind = ical::icalproperty_kind_ICAL_LASTMODIFIED_PROPERTY;
    assert_eq!(
      "20130101T010203Z",
      event.get_property(last_modified_kind).unwrap().get_value()
    );
  }

  #[test]
  fn test_with_location() {
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();

    let location = "test";
    let new_cal = cal.with_location(location);

    let event = new_cal.get_principal_event();
    assert_eq!(location, event.get_location().unwrap())
  }

  //#[test]
  //fn test_with_internal_timestamp() {
  //let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();
  //
  //let timestamp = IcalTime::floating_ymd(2018, 1, 1).and_hms(11, 30, 20);
  //let new_cal = cal.with_internal_timestamp(&timestamp);
  //
  //let event = new_cal.get_principal_event();
  //assert_eq!(timestamp, event.get_dtstart().unwrap());
  //}

  #[test]
  fn with_uid_test() {
    let path = PathBuf::from("test/path");
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, Some(&path)).unwrap();

    let uid = "my_new_uid";
    let new_cal = cal.with_uid(uid).unwrap();

    for event in new_cal.events_iter() {
      assert_eq!(uid, event.get_uid());
    }
    assert_eq!(
      Some(path.with_file_name(uid.to_owned() + ".ics")),
      new_cal.path
    );
  }

  #[test]
  fn with_uid_multiple_test() {
    let path = PathBuf::from("test/path");
    let cal = IcalVCalendar::from_str(testdata::TEST_MULTIPLE_EVENTS, Some(&path)).unwrap();

    let uid = "my_new_uid";
    let new_cal = cal.with_uid(uid);

    assert!(new_cal.is_err());
  }

  #[test]
  fn with_keep_uid_test() {
    let path = PathBuf::from("test/path");
    let cal = IcalVCalendar::from_str(testdata::TEST_MULTIPLE_EVENTS, Some(&path)).unwrap();

    for uid in &["uid1", "uid2"] {
      let new_cal = cal.clone().with_keep_uid(uid);

      assert_eq!(1, new_cal.events_iter().count());
      assert_eq!(*uid, new_cal.get_uid());
      assert_eq!(*uid, new_cal.get_principal_event().get_uid());
    }
  }

  #[test]
  fn clone_test() {
    let path = PathBuf::from("test/path");
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, Some(&path)).unwrap();
    let cal2 = cal.clone().with_uid("my_new_uid").unwrap();

    assert_ne!(cal.get_uid(), cal2.get_uid());
  }

  #[test]
  fn parse_checker_test_negative() {
    let c_str = CString::new(testdata::TEST_EVENT_NO_PRODID).unwrap();
    unsafe {
      let parsed_cal = ical::icalparser_parse_string(c_str.as_ptr());
      assert!(IcalVCalendar::check_icalcomponent(parsed_cal).is_some())
    }
  }

  #[test]
  fn parse_checker_test() {
    let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();
    assert!(cal.check_for_errors().is_none());
  }
}
