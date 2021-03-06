//! libical high level interface
//!
//! This library offers a high-level interface for the widely used libical. It relies upon the
//! libical-sys crate, which is a thin rust layer over the libical C API.
//! It provides a safe interface to libical that is rather lower-level, as well as convenience
//! functionality build on this low-level interface for easy handling of icalendar data.
//!

#![warn(unused_extern_crates)]
#![allow(clippy::redundant_closure)] // disable "redundant closure" lint

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(test)]
#[macro_use]
extern crate indoc;

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
use ical; // extern crate

// libical does some weird, non-threadsafe things in timezone methods, notably
// icaltime_convert_to_zone (which is also called in icaltime_as_timet_with_zone)
// see these two (independent!) bugs:
// https://github.com/libical/libical/issues/86
// https://github.com/libical/libical/commit/0ebf2d9a7183be94991c2681c6e3f009c64cf7cc
use std::sync::Mutex;
lazy_static! {
    static ref TZ_MUTEX: Mutex<i32> = Mutex::new(0);
}

pub mod component;
pub mod duration;
pub mod property;
pub mod time;
pub mod timezone;
pub mod vcalendar;
pub mod vevent;
mod utils;

#[cfg(test)]
pub mod testing;

pub use crate::component::IcalComponent;
pub use crate::duration::IcalDuration;
pub use crate::property::IcalProperty;
pub use crate::time::IcalTime;
pub use crate::timezone::IcalTimeZone;
pub use crate::vcalendar::IcalEventIter;
pub use crate::vcalendar::IcalVCalendar;
pub use crate::vevent::IcalVEvent;

