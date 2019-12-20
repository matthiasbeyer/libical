use chrono::NaiveDate;
use std::ffi::CStr;
use std::fmt;

use super::component::IcalComponent;
use crate::ical;

/// A property in the ical data
///
/// This type represents a single property (name + value).
pub struct IcalProperty<'a> {
    pub ptr: *mut ical::icalproperty,
    _parent: &'a dyn IcalComponent,
}

impl<'a> Drop for IcalProperty<'a> {
    fn drop(&mut self) {
        unsafe {
            ical::icalproperty_free(self.ptr);
        }
    }
}

impl<'a> fmt::Debug for IcalProperty<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_ical_string())
    }
}

impl<'a> IcalProperty<'a> {
    pub fn from_ptr(ptr: *mut ical::icalproperty, parent: &'a dyn IcalComponent) -> Self {
        IcalProperty {
            ptr,
            _parent: parent,
        }
    }

    pub fn get_name(&self) -> String {
        unsafe {
            let cstr = CStr::from_ptr(ical::icalproperty_get_property_name(self.ptr));
            cstr.to_string_lossy().into_owned()
        }
    }

    pub fn get_value(&self) -> String {
        unsafe {
            let cstr = CStr::from_ptr(ical::icalproperty_get_value_as_string(self.ptr));
            cstr.to_string_lossy().into_owned()
        }
    }

    pub fn as_ical_string(&self) -> String {
        unsafe {
            let cstr = CStr::from_ptr(ical::icalproperty_as_ical_string(self.ptr));
            cstr.to_string_lossy().trim().to_owned()
        }
    }

    pub fn get_value_as_date(&self) -> Option<NaiveDate> {
        unsafe {
            let date = ical::icaltime_from_string(ical::icalproperty_get_value_as_string(self.ptr));
            NaiveDate::from_ymd_opt(date.year, date.month as u32, date.day as u32)
        }
    }
}
