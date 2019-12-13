use chrono::prelude::*;
use std::fmt::Display;
use std::time;

pub fn joinlines(first: &str, second: &str) -> String {
    use itertools::Itertools;

    let first = first.split(|x| x == '\n');
    let second = second.split(|x| x == '\n');
    let maxlen = first.clone().map(|x| x.len()).max().unwrap();

    first
        .zip(second)
        .map(|(fst, snd)| format!("{:width$} {}", fst, snd, width = maxlen))
        .join("\n")
}

pub fn format_duration(duration: &time::Duration) -> impl Display {
    duration.as_millis()
}

pub fn get_bucket_for_date(date: Date<Local>) -> String {
    date.format("%G-W%V").to_string()
}

#[cfg(not(test))]
pub fn make_new_uid() -> String {
    use uuid::Uuid;

    let suffix = "@khaleesi";
    format!("{}{}", Uuid::new_v4().to_hyphenated_ref(), suffix)
}

#[cfg(test)]
pub fn make_new_uid() -> String {
    "11111111-2222-3333-4444-444444444444@khaleesi".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_test() {
        let millis: u64 = 12345678;
        let duration = time::Duration::from_millis(millis);
        let string_duration = format!("{}", format_duration(&duration));
        let string_from_secs = format!("{}", millis);
        assert_eq!(string_from_secs, string_duration);
    }

    #[test]
    fn joinlines_test() {
        let first = ["123", "ß", "1234"].join("\n");
        let second = ["abc", "1", "Otto"].join("\n");
        let expected = indoc!(
            "
      123  abc
      ß    1
      1234 Otto"
        );
        assert_eq!(expected, joinlines(first.as_str(), second.as_str()));
    }
}
