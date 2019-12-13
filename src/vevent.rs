use std::ffi::CStr;

use super::IcalComponent;
use super::IcalDuration;
use super::IcalTime;
use super::IcalVCalendar;
use crate::ical;

pub struct IcalVEvent {
    ptr: *mut ical::icalcomponent,
    parent: Option<IcalVCalendar>,
}

impl Drop for IcalVEvent {
    fn drop(&mut self) {
        unsafe {
            // println!("free");
            ical::icalcomponent_free(self.ptr);
        }
    }
}

impl IcalComponent for IcalVEvent {
    fn get_ptr(&self) -> *mut ical::icalcomponent {
        self.ptr
    }
    fn as_component(&self) -> &dyn IcalComponent {
        self
    }
}

impl IcalVEvent {
    pub fn from_ptr_with_parent(
        ptr: *mut ical::icalcomponent,
        parent: &IcalVCalendar,
    ) -> IcalVEvent {
        IcalVEvent {
            ptr,
            parent: Some(parent.shallow_copy()),
            //instance_timestamp: None,
        }
    }

    pub fn get_dtend(&self) -> Option<IcalTime> {
        unsafe {
            let dtend = ical::icalcomponent_get_dtend(self.ptr);
            trace!("{:?}", dtend);
            if ical::icaltime_is_null_time(dtend) == 1 {
                None
            } else {
                Some(IcalTime::from(dtend))
            }
        }
    }

    fn get_duration_internal(&self) -> Option<IcalDuration> {
        unsafe {
            let duration = ical::icalcomponent_get_duration(self.ptr);
            if ical::icaldurationtype_is_bad_duration(duration) == 0
                && ical::icaldurationtype_is_null_duration(duration) == 0
            {
                Some(IcalDuration::from(duration))
            } else {
                None
            }
        }
    }

    pub fn get_duration(&self) -> Option<IcalDuration> {
        self.get_duration_internal().or_else(|| {
            if self.get_dtstart()?.is_date() {
                Some(IcalDuration::from_seconds(24 * 60 * 60))
            } else {
                Some(IcalDuration::from_seconds(0))
            }
        })
    }

    pub fn get_dtstart(&self) -> Option<IcalTime> {
        unsafe {
            let dtstart = ical::icalcomponent_get_dtstart(self.ptr);
            if ical::icaltime_is_null_time(dtstart) == 1 {
                None
            } else {
                Some(IcalTime::from(dtstart))
            }
        }
    }

    pub fn has_property_rrule(&self) -> bool {
        !self
            .get_properties(ical::icalproperty_kind_ICAL_RRULE_PROPERTY)
            .is_empty()
    }

    pub fn get_recur_datetimes(&self) -> Vec<IcalTime> {
        let mut result: Vec<IcalTime> = vec![];
        let result_ptr: *mut ::std::os::raw::c_void =
            &mut result as *mut _ as *mut ::std::os::raw::c_void;

        let dtstart = self.get_dtstart().unwrap();
        unsafe {
            //unroll up to 1 year in the future
            let mut dtend = ical::icaltime_today();
            dtend.year += 1;

            ical::icalcomponent_foreach_recurrence(
                self.ptr,
                *dtstart,
                dtend,
                Some(recur_callback),
                result_ptr,
            );
        }

        if dtstart.is_date() {
            result = result.into_iter().map(|time| time.as_date()).collect();
        }

        result
    }

    pub fn shallow_copy(&self) -> IcalVEvent {
        IcalVEvent {
            ptr: self.ptr,
            parent: self.parent.as_ref().map(|parent| parent.shallow_copy()),
        }
    }

    //TODO remove this function
    pub(crate) fn with_internal_timestamp(&self, _datetime: &IcalTime) -> IcalVEvent {
        IcalVEvent {
            ptr: self.ptr,
            parent: self.parent.as_ref().map(|parent| parent.shallow_copy()),
        }
    }

    pub fn get_parent(&self) -> Option<&IcalVCalendar> {
        self.parent.as_ref()
    }

    pub fn get_summary(&self) -> Option<String> {
        unsafe {
            let ptr = ical::icalcomponent_get_summary(self.ptr);
            if !ptr.is_null() {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            } else {
                None
            }
        }
    }

    pub fn get_description(&self) -> Option<String> {
        unsafe {
            let ptr = ical::icalcomponent_get_description(self.ptr);
            if !ptr.is_null() {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            } else {
                None
            }
        }
    }

    pub fn get_location(&self) -> Option<String> {
        unsafe {
            let ptr = ical::icalcomponent_get_location(self.ptr);
            if !ptr.is_null() {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            } else {
                None
            }
        }
    }

    pub fn get_uid(&self) -> String {
        unsafe {
            let cstr = CStr::from_ptr(ical::icalcomponent_get_uid(self.ptr));
            cstr.to_string_lossy().into_owned()
        }
    }

    pub fn is_allday(&self) -> bool {
        unsafe {
            let dtstart = ical::icalcomponent_get_dtstart(self.ptr);
            dtstart.is_date == 1
        }
    }
}

extern "C" fn recur_callback(
    _comp: *mut ical::icalcomponent,
    span: *mut ical::icaltime_span,
    data: *mut ::std::os::raw::c_void,
) {
    let data: &mut Vec<IcalTime> = unsafe { &mut *(data as *mut Vec<IcalTime>) };

    let spanstart = unsafe {
        let start = (*span).start;
        IcalTime::from_timestamp(start)
    };

    data.push(spanstart);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata;
    use chrono::NaiveDate;

    #[test]
    fn test_get_all_properties() {
        let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();

        let event = cal.get_principal_event();
        let props = event.get_properties_all();
        assert_eq!(7, props.len());
    }

    #[test]
    fn test_get_property_get_value() {
        let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY_ALLDAY, None).unwrap();
        let event = cal.get_principal_event();
        let prop = event.get_properties_by_name("DTSTART");

        assert_eq!(1, prop.len());
        assert_eq!("DTSTART", prop[0].get_name());
        assert_eq!("20070628", prop[0].get_value());
        assert_eq!(
            NaiveDate::from_ymd_opt(2007, 6, 28),
            prop[0].get_value_as_date()
        );
    }

    #[test]
    fn test_get_property_debug() {
        let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY_ALLDAY, None).unwrap();
        let event = cal.get_principal_event();
        let prop = event
            .get_property(ical::icalproperty_kind_ICAL_DTSTART_PROPERTY)
            .unwrap();

        assert_eq!("DTSTART;VALUE=DATE:20070628", format!("{:?}", prop));
    }

    #[test]
    fn test_get_summary() {
        let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(
            Some("Festival International de Jazz de Montreal".to_string()),
            event.get_summary()
        );
    }

    #[test]
    fn test_get_summary_none() {
        let cal = IcalVCalendar::from_str(testdata::TEST_NO_SUMMARY, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(None, event.get_summary());
    }

    #[test]
    fn test_get_dtstart() {
        let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(
            IcalTime::floating_ymd(2007, 06, 28).and_hms(13, 29, 00),
            event.get_dtstart().unwrap()
        );
    }

    #[test]
    fn test_get_dtstart_negative() {
        let cal = IcalVCalendar::from_str(testdata::TEST_NO_DTSTART, None).unwrap();
        let event = cal.get_principal_event();

        assert!(event.get_dtstart().is_none());
    }

    #[test]
    fn test_get_dtend() {
        let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(
            IcalTime::floating_ymd(2007, 7, 9).and_hms(7, 29, 00),
            event.get_dtend().unwrap()
        );
    }

    #[test]
    fn test_get_dtend_negative() {
        let cal = IcalVCalendar::from_str(testdata::TEST_NO_DTSTART, None).unwrap();
        let event = cal.get_principal_event();

        assert!(event.get_dtend().is_none());
    }

    #[test]
    fn test_get_duration_internal_normal() {
        let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(
            Some(IcalDuration::from_seconds(10 * 24 * 60 * 60 + 18 * 60 * 60)),
            event.get_duration_internal()
        );
    }

    #[test]
    fn test_get_duration_normal() {
        let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(
            Some(IcalDuration::from_seconds(10 * 24 * 60 * 60 + 18 * 60 * 60)),
            event.get_duration()
        );
    }

    #[test]
    fn test_get_duration_inernal_startdate_only() {
        let cal = IcalVCalendar::from_str(testdata::TEST_DTSTART_ONLY_DATE, None).unwrap();
        let event = cal.get_principal_event();

        assert!(event.get_duration_internal().is_none());
    }

    #[test]
    fn test_get_duration_startdate_only() {
        let cal = IcalVCalendar::from_str(testdata::TEST_DTSTART_ONLY_DATE, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(
            Some(IcalDuration::from_seconds(24 * 60 * 60)),
            event.get_duration()
        );
    }

    #[test]
    fn test_get_duration_internal_startdatetime_only() {
        let cal = IcalVCalendar::from_str(testdata::TEST_DTSTART_ONLY_DATETIME, None).unwrap();
        let event = cal.get_principal_event();

        assert!(event.get_duration_internal().is_none());
    }

    #[test]
    fn test_get_duration_startdatetime_only() {
        let cal = IcalVCalendar::from_str(testdata::TEST_DTSTART_ONLY_DATETIME, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(Some(IcalDuration::from_seconds(0)), event.get_duration());
    }

    #[test]
    fn test_get_description() {
        let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_ONE_MEETING, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(
      Some(
        "Discuss how we can test c&s interoperability\nusing iCalendar and other IETF standards."
          .to_string()
      ),
      event.get_description()
    );
    }

    #[test]
    fn test_get_description_none() {
        let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_MULTIDAY, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(None, event.get_description());
    }

    #[test]
    fn test_get_location() {
        let cal = IcalVCalendar::from_str(testdata::TEST_EVENT_ONE_MEETING, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(Some("LDB Lobby".to_string()), event.get_location());
    }

    #[test]
    fn test_get_location_none() {
        let cal = IcalVCalendar::from_str(testdata::TEST_NO_SUMMARY, None).unwrap();
        let event = cal.get_principal_event();

        assert_eq!(None, event.get_location());
    }
}
