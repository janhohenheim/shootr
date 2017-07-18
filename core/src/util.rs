extern crate chrono;

use self::chrono::{DateTime, Utc};
use std::env;

pub fn read_env_var(var: &str) -> String {
    env::var_os(var)
        .expect(&format!(
            "{} must be specified. \
             Did you forget to add it to your .env file?",
            var
        ))
        .into_string()
        .expect(&format!("{} does not contain a valid UTF8 string", var))
}


pub fn elapsed_ms(from: DateTime<Utc>, to: DateTime<Utc>) -> u64 {
    let ms = to.signed_duration_since(from).num_milliseconds();
    assert!(ms >= 0);
    ms as u64
}


pub fn clamp<T>(val: T, min: T, max: T) -> T
where
    T: Ord,
{
    match (val < min, val > max) {
        (true, _) => min,
        (_, true) => max,
        _ => val,
    }
}


#[test]
fn read_string_envvar() {
    env::set_var("TEST", "foo");
    assert_eq!("foo", &read_env_var("TEST"));
}

#[test]
#[should_panic]
fn read_empty_envvar() {
    env::remove_var("EMPTY");
    read_env_var("EMPTY");
}


#[cfg(test)]
use util::chrono::TimeZone;

#[test]
fn one_elapsed_ms() {
    let a = Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 0, 0);
    let b = Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 0, 1);
    assert_eq!(1, elapsed_ms(a, b));
}

#[test]
fn one_elapsed_second() {
    let a = Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 0, 0);
    let b = Utc.ymd(1970, 1, 1).and_hms(0, 0, 1);
    assert_eq!(1000, elapsed_ms(a, b));
}

#[test]
fn no_elapsed_time() {
    let a = Utc.ymd(1970, 1, 1).and_hms(0, 0, 1);
    assert_eq!(0, elapsed_ms(a, a));
}


#[test]
#[should_panic]
fn negative_elapsed_time() {
    let a = Utc.ymd(1970, 1, 1).and_hms(0, 0, 1);
    let b = Utc.ymd(1970, 1, 1).and_hms(0, 0, 0);
    elapsed_ms(a, b);
}

#[test]
fn clamp_in_range() {
    let res = clamp(1, 0, 2);
    assert_eq!(1, res);
}

#[test]
fn clamp_min() {
    let res = clamp(-2, -2, 2);
    assert_eq!(-2, res);
}

#[test]
fn clamp_max() {
    let res = clamp(0, -2, 0);
    assert_eq!(0, res);
}

#[test]
fn clamp_less_than_min() {
    let res = clamp(-1, 0, 2);
    assert_eq!(0, res);
}

#[test]
fn clamp_more_than_max() {
    let res = clamp(999, 9, 10);
    assert_eq!(10, res);
}
