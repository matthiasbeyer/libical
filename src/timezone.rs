use crate::ical;
use std::ffi::{CStr, CString};
use std::ops::Deref;

use super::IcalTime;
use crate::utils::dateutil;

pub struct IcalTimeZone {
    timezone: *mut ical::icaltimezone,
}

impl Deref for IcalTimeZone {
    type Target = *mut ical::icaltimezone;

    fn deref(&self) -> &*mut ical::icaltimezone {
        &self.timezone
    }
}

impl IcalTimeZone {
    pub fn from_ptr_copy(ptr: *const ical::icaltimezone) -> Self {
        let timezone = unsafe {
            // unsafe, but we know icaltimezone_copy doesn't actually mutate
            ical::icaltimezone_copy(ptr as *mut ical::icaltimezone)
        };
        IcalTimeZone { timezone }
    }

    pub fn from_name(tz_name: &str) -> Result<Self, String> {
        let tz_cstr = CString::new(tz_name).unwrap();
        let builtin = unsafe { ical::icaltimezone_get_builtin_timezone(tz_cstr.as_ptr()) };
        if !builtin.is_null() {
            // need to copy here to guarantee we don't touch the builtin zones
            let timezone = unsafe { ical::icaltimezone_copy(builtin) };
            Ok(IcalTimeZone { timezone })
        } else {
            Err(format!("Unknown timezone: {}", tz_name))
        }
    }

    pub fn local() -> Self {
        let tz_name = dateutil::find_local_timezone();
        IcalTimeZone::from_name(&tz_name).unwrap()
    }

    pub fn utc() -> Self {
        let timezone = unsafe { ical::icaltimezone_get_utc_timezone() };
        IcalTimeZone { timezone }
    }

    pub fn get_name(&self) -> String {
        unsafe {
            let name = ical::icaltimezone_get_display_name(self.timezone);
            CStr::from_ptr(name).to_string_lossy().trim().to_string()
        }
    }

    pub fn get_offset_at_time(&self, time: &IcalTime) -> i32 {
        let mut icaltime = **time;
        let mut is_dst = 0;
        unsafe { ical::icaltimezone_get_utc_offset(self.timezone, &mut icaltime, &mut is_dst) }
    }

    pub fn ymd(&self, year: i32, month: i32, day: i32) -> IcalTime {
        IcalTime::floating_ymd(year, month, day).with_timezone(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testdata;

    #[test]
    fn test_utc() {
        let tz = IcalTimeZone::utc();
        assert_eq!("UTC", tz.get_name());
    }

    #[test]
    fn test_get_offset_utc() {
        let tz = IcalTimeZone::utc();
        let time = IcalTime::utc();

        let offset = tz.get_offset_at_time(&time);

        assert_eq!(0, offset);
    }

    #[test]
    fn test_ymd() {
        let tz = IcalTimeZone::from_name("US/Eastern").unwrap();
        let date = tz.ymd(2014, 3, 1);
        assert_eq!(IcalTime::floating_ymd(2014, 3, 1), date);
        assert_eq!(1393650000, date.timestamp());
        assert_eq!("US/Eastern", date.get_timezone().unwrap().get_name());
    }

    #[test]
    fn test_ymd_and_hms() {
        let tz = IcalTimeZone::from_name("US/Eastern").unwrap();
        let date = tz.ymd(2014, 3, 1).and_hms(17, 20, 0);
        assert_eq!(IcalTime::floating_ymd(2014, 3, 1).and_hms(17, 20, 0), date);
        assert_eq!(1393712400, date.timestamp());
        assert_eq!("US/Eastern", date.get_timezone().unwrap().get_name());
    }

    #[test]
    fn test_from_name() {
        let tz = IcalTimeZone::from_name("US/Eastern").unwrap();
        assert_eq!("US/Eastern", tz.get_name());
    }

    #[test]
    fn test_from_name_fail() {
        let tz = IcalTimeZone::from_name("lulz");
        assert!(tz.is_err());
    }

    #[test]
    fn test_get_offset_eastern() {
        let time = IcalTime::utc();
        let tz = IcalTimeZone::from_name("US/Eastern").unwrap();

        let offset = tz.get_offset_at_time(&time);

        assert_eq!(-5 * 60 * 60, offset);
    }
}
