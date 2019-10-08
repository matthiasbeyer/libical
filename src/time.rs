use super::IcalDuration;
use super::IcalTimeZone;
use super::TZ_MUTEX;
use crate::ical;
use crate::utils::dateutil;
use chrono::{Date, DateTime, Local, TimeZone, Utc};
use std::ffi::{CStr, CString};
use std::fmt::{Display, Error, Formatter};
use std::ops::{Add, Deref};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct IcalTime {
    time: ical::icaltimetype,
}

impl IcalTime {
    pub fn utc() -> Self {
        dateutil::now().into()
    }

    pub fn local() -> Self {
        dateutil::now().with_timezone(&Local).into()
    }

    pub fn floating_ymd(year: i32, month: i32, day: i32) -> Self {
        let time = ical::icaltimetype {
            year,
            month,
            day,
            hour: 0,
            minute: 0,
            second: 0,
            is_date: 1,
            is_daylight: 0,
            zone: ::std::ptr::null(),
        };
        IcalTime { time }
    }

    pub fn and_hms(&self, hour: i32, minute: i32, second: i32) -> Self {
        let mut time = self.time;
        time.hour = hour;
        time.minute = minute;
        time.second = second;
        time.is_date = 0;

        IcalTime { time }
    }

    pub fn from_timestamp(timestamp: i64) -> Self {
        let _lock = TZ_MUTEX.lock();
        let utc = IcalTimeZone::utc();
        let is_date = 0;
        let time = unsafe { ical::icaltime_from_timet_with_zone(timestamp, is_date, *utc) };
        IcalTime { time }
    }

    pub fn timestamp(&self) -> i64 {
        let _lock = TZ_MUTEX.lock();
        unsafe { ical::icaltime_as_timet_with_zone(self.time, self.time.zone) }
    }

    pub fn is_date(&self) -> bool {
        self.time.is_date != 0
    }

    pub fn as_date(&self) -> IcalTime {
        let mut time = self.time;
        time.is_date = 1;
        IcalTime { time }
    }

    pub fn get_timezone(&self) -> Option<IcalTimeZone> {
        if self.time.zone.is_null() {
            return None;
        }
        let tz_ptr = unsafe { ical::icaltime_get_timezone(self.time) };
        Some(IcalTimeZone::from_ptr_copy(tz_ptr))
    }

    pub fn with_timezone(&self, timezone: &IcalTimeZone) -> IcalTime {
        let _lock = TZ_MUTEX.lock();
        let mut time = unsafe { ical::icaltime_convert_to_zone(self.time, **timezone) };
        //icaltime_convert_to_zone does nothing if is_date == 1
        time.zone = **timezone;
        IcalTime { time }
    }

    pub fn pred(&self) -> IcalTime {
        let mut time = self.time;
        time.day -= 1;
        let time = unsafe { ical::icaltime_normalize(time) };
        IcalTime { time }
    }

    pub fn succ(&self) -> IcalTime {
        let mut time = self.time;
        time.day += 1;
        let time = unsafe { ical::icaltime_normalize(time) };
        IcalTime { time }
    }
}

impl Deref for IcalTime {
    type Target = ical::icaltimetype;

    fn deref(&self) -> &ical::icaltimetype {
        &self.time
    }
}

impl Display for IcalTime {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let cstr = unsafe { CStr::from_ptr(ical::icaltime_as_ical_string(self.time)) };
        let string = cstr.to_string_lossy();
        write!(f, "{}", string)
    }
}

impl FromStr for IcalTime {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unsafe {
            let c_str = CString::new(s).unwrap();
            let time = ical::icaltime_from_string(c_str.as_ptr());
            if ical::icaltime_is_null_time(time) == 0 {
                Ok(IcalTime { time })
            } else {
                Err(format!("Could not parse time {}", s))
            }
        }
    }
}

impl PartialEq<IcalTime> for IcalTime {
    fn eq(&self, rhs: &IcalTime) -> bool {
        let _lock = TZ_MUTEX.lock();
        let cmp = unsafe { ical::icaltime_compare(self.time, rhs.time) };
        cmp == 0
    }
}

impl Eq for IcalTime {}

impl From<ical::icaltimetype> for IcalTime {
    fn from(time: ical::icaltimetype) -> IcalTime {
        IcalTime { time }
    }
}

impl Add<IcalDuration> for IcalTime {
    type Output = IcalTime;

    fn add(self, other: IcalDuration) -> IcalTime {
        let time = unsafe { ical::icaltime_add(self.time, *other) };
        IcalTime { time }
    }
}

impl<T: Into<IcalTime> + Clone> From<&T> for IcalTime {
    fn from(time: &T) -> IcalTime {
        time.clone().into()
    }
}

impl From<DateTime<Local>> for IcalTime {
    fn from(time: DateTime<Local>) -> IcalTime {
        let timestamp = time.timestamp();
        let local = IcalTimeZone::local();
        IcalTime::from_timestamp(timestamp).with_timezone(&local)
    }
}

impl From<DateTime<Utc>> for IcalTime {
    fn from(time: DateTime<Utc>) -> IcalTime {
        let timestamp = time.timestamp();
        IcalTime::from_timestamp(timestamp)
    }
}

impl From<Date<Local>> for IcalTime {
    fn from(date: Date<Local>) -> IcalTime {
        let timestamp = date.with_timezone(&Utc).and_hms(0, 0, 0).timestamp();
        let timezone = IcalTimeZone::local();
        IcalTime::from_timestamp(timestamp)
            .with_timezone(&timezone)
            .as_date()
    }
}

impl From<Date<Utc>> for IcalTime {
    fn from(date: Date<Utc>) -> IcalTime {
        let timestamp = date.and_hms(0, 0, 0).timestamp();
        IcalTime::from_timestamp(timestamp).as_date()
    }
}

impl From<IcalTime> for Date<Local> {
    fn from(time: IcalTime) -> Date<Local> {
        Local.timestamp(time.timestamp(), 0).date()
    }
}

impl From<IcalTime> for DateTime<Local> {
    fn from(time: IcalTime) -> DateTime<Local> {
        Local.timestamp(time.timestamp(), 0)
    }
}

impl From<IcalTime> for Date<Utc> {
    fn from(time: IcalTime) -> Date<Utc> {
        Utc.timestamp(time.timestamp(), 0).date()
    }
}

impl From<IcalTime> for DateTime<Utc> {
    fn from(time: IcalTime) -> DateTime<Utc> {
        Utc.timestamp(time.timestamp(), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testdata;

    #[test]
    fn test_now() {
        let now = IcalTime::utc();

        assert_eq!("20130101T010203Z", now.to_string());
        assert_eq!(1357002123, now.timestamp());
    }

    #[test]
    fn test_from_local() {
        let local_time = Local.ymd(2014, 01, 01).and_hms(01, 02, 03);
        let time = IcalTime::from(local_time);

        assert_eq!("Europe/Berlin", time.get_timezone().unwrap().get_name());
        assert_eq!(1388534523, time.timestamp());
    }

    #[test]
    fn test_local() {
        let time = IcalTime::local();
        assert_eq!("20130101T020203", time.to_string());
    }

    #[test]
    fn test_parse() {
        let time = "20130101T010203Z".parse::<IcalTime>().unwrap();
        assert_eq!("20130101T010203Z", time.to_string());
    }

    #[test]
    fn test_parse_negative() {
        let time = "201XXX01T010203Z".parse::<IcalTime>();
        assert!(time.is_err());
    }

    #[test]
    fn test_with_timezone() {
        let utc = IcalTime::utc();
        let tz = IcalTimeZone::from_name("US/Eastern").unwrap();

        let time = utc.with_timezone(&tz);

        assert_eq!("US/Eastern", time.get_timezone().unwrap().get_name());
        assert_eq!("20121231T200203", time.to_string());
        assert_eq!(1357002123, time.timestamp());
    }

    #[test]
    fn test_get_timezone_negative() {
        let time = IcalTime::floating_ymd(2018, 02, 03);
        assert!(time.get_timezone().is_none());
    }

    #[test]
    fn test_from_local_date() {
        let local_date = Local.ymd(2014, 01, 01);
        let time = IcalTime::from(local_date);

        assert_eq!("Europe/Berlin", time.get_timezone().unwrap().get_name());
        assert_eq!("20140101", time.to_string());
    }

    #[test]
    fn test_from_utc_date() {
        let utc_date = Utc.ymd(2014, 01, 01);
        let time = IcalTime::from(utc_date);

        assert_eq!("UTC", time.get_timezone().unwrap().get_name());
        assert_eq!("20140101", time.to_string());
    }

    #[test]
    fn test_into_utc_date() {
        let time = IcalTimeZone::utc().ymd(2014, 02, 02);
        let date: Date<Utc> = time.into();
        assert_eq!(Utc.ymd(2014, 02, 02), date);
    }

    #[test]
    fn test_into_utc_datetime() {
        let time = IcalTimeZone::utc().ymd(2014, 02, 02).and_hms(13, 37, 00);
        let datetime: DateTime<Utc> = time.into();
        assert_eq!(Utc.ymd(2014, 02, 02).and_hms(13, 37, 00), datetime);
    }

    #[test]
    fn test_into_local_date() {
        let time = IcalTimeZone::local().ymd(2014, 02, 02);
        let date: Date<Local> = time.into();
        assert_eq!(Local.ymd(2014, 02, 02), date);
    }

    #[test]
    fn test_into_local_datetime() {
        let time = IcalTimeZone::local().ymd(2014, 02, 02).and_hms(13, 37, 00);
        let datetime: DateTime<Local> = time.into();
        assert_eq!(Local.ymd(2014, 02, 02).and_hms(13, 37, 00), datetime);
    }

    #[test]
    fn test_into_local_datetime_utc() {
        let time = IcalTimeZone::utc().ymd(2014, 02, 02).and_hms(13, 37, 00);
        let datetime: DateTime<Local> = time.into();
        assert_eq!(Local.ymd(2014, 02, 02).and_hms(14, 37, 00), datetime);
    }

    #[test]
    fn test_add() {
        let now = IcalTime::utc();
        let duration = IcalDuration::from_seconds(123);

        let sum = now + duration;

        assert_eq!(1357002123 + 123, sum.timestamp());
    }

    #[test]
    fn test_pred() {
        let time = IcalTime::utc();
        assert_eq!("20121231T010203Z", time.pred().to_string());
    }

    #[test]
    fn test_succ() {
        let time = IcalTime::utc();
        assert_eq!("20130102T010203Z", time.succ().to_string());
    }
}
