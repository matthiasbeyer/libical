use std::ops::{Deref,Add};
use std::ffi::{CStr,CString};
use crate::ical;
use std::fmt::{Error,Display,Formatter};
use std::str::FromStr;
use std::cmp::Ordering;


#[derive(Clone,Debug)]
pub struct IcalDuration {
  duration: ical::icaldurationtype,
}

impl IcalDuration {
  pub fn from_seconds(seconds: i32) -> IcalDuration {
    let duration = unsafe { ical::icaldurationtype_from_int(seconds) };
    IcalDuration{ duration }
  }

  pub fn to_seconds(&self) -> i32 {
    unsafe { ical::icaldurationtype_as_int(self.duration) }
  }
}

impl Deref for IcalDuration {
  type Target = ical::icaldurationtype;

  fn deref(&self) -> &ical::icaldurationtype {
    &self.duration
  }
}

impl Display for IcalDuration {
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    let cstr = unsafe {
      CStr::from_ptr(ical::icaldurationtype_as_ical_string(self.duration))
    };
    let string = cstr.to_string_lossy();
    write!(f, "{}", string)
  }
}

impl FromStr for IcalDuration {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let c_str = CString::new(s).unwrap();
    let duration = unsafe {
      let duration = ical::icaldurationtype_from_string(c_str.as_ptr());
      if ical::icaldurationtype_is_null_duration(duration) == 0 {
        Some(duration)
      } else {
        None
      }
    };
    if let Some(duration) = duration {
      Ok(IcalDuration { duration })
    } else {
      return Err(format!("Could not parse duration {}", s));
    }
  }
}

impl PartialEq<IcalDuration> for IcalDuration {
  fn eq(&self, rhs: &IcalDuration) -> bool {
    self.to_seconds() == rhs.to_seconds()
  }
}

impl Eq for IcalDuration {}

impl PartialOrd for IcalDuration {
  fn partial_cmp(&self, rhs: &IcalDuration) -> Option<Ordering> {
    Some(self.cmp(rhs))
  }
}

impl Ord for IcalDuration {
  fn cmp(&self, rhs: &IcalDuration) -> Ordering {
    let left = self.to_seconds();
    let right = rhs.to_seconds();
    if left == right {
      Ordering::Equal
    } else if left < right {
      Ordering::Less
    } else {
      Ordering::Greater
    }
  }
}

impl From<ical::icaldurationtype> for IcalDuration {
  fn from(duration: ical::icaldurationtype) -> IcalDuration {
    IcalDuration { duration }
  }
}

impl From<IcalDuration> for chrono::Duration {
  fn from(duration: IcalDuration) -> chrono::Duration {
    chrono::Duration::seconds(i64::from(duration.to_seconds()))
  }
}

impl From<chrono::Duration> for IcalDuration {
  fn from(duration: chrono::Duration) -> IcalDuration {
    IcalDuration::from_seconds(duration.num_seconds() as i32)
  }
}
impl Add for IcalDuration {
    type Output = IcalDuration;

    fn add(self, other: IcalDuration) -> IcalDuration {
      let seconds = self.to_seconds() + other.to_seconds();
      IcalDuration::from_seconds(seconds)
    }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse() {
    let duration = "PT86400S".parse::<IcalDuration>().unwrap();
    assert_eq!(IcalDuration::from_seconds(24*60*60), duration);
    assert_eq!(86400, duration.to_seconds());
  }

  #[test]
  fn test_parse_fail() {
    let duration = "swag".parse::<IcalDuration>();
    assert!(duration.is_err());
  }

  #[test]
  fn test_display() {
    let duration = IcalDuration::from_seconds(5*24*60*60 + 22*60*60 + 33*60 + 33);
    assert_eq!("P5DT22H33M33S", duration.to_string());
  }

  #[test]
  fn test_to_chrono() {
    let from_duration = IcalDuration::from_seconds(5*24*60*60 + 22*60*60 + 33*60 + 33);
    let duration: chrono::Duration = from_duration.into();
    assert_eq!(chrono::Duration::seconds(5*24*60*60 + 22*60*60 + 33*60 + 33), duration);
  }

  #[test]
  fn test_from_chrono() {
    let from_duration = chrono::Duration::seconds(5*24*60*60 + 22*60*60 + 33*60 + 33);
    let duration: IcalDuration = from_duration.into();
    assert_eq!(IcalDuration::from_seconds(5*24*60*60 + 22*60*60 + 33*60 + 33), duration);
  }

  #[test]
  fn test_add() {
    let fst = IcalDuration::from_seconds(123);
    let snd = IcalDuration::from_seconds(4567);

    let sum = fst + snd;

    assert_eq!(IcalDuration::from_seconds(123+4567), sum);
  }

  #[test]
  fn test_cmp() {
    let more = IcalDuration::from_seconds(49128);
    let less = IcalDuration::from_seconds(5);

    assert!(less == less);
    assert!(more == more);
    assert!(less < more);
    assert!(!(more < less));
    assert!(!(more == less));
  }
}
