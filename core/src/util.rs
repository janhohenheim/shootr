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
    to.signed_duration_since(from).num_milliseconds() as u64
}


pub fn clamp<'a, T>(val: &'a T, min: &'a T, max: &'a T) -> &'a T
where
    T: Ord,
{
    match (*val < *min, *val > *max) {
        (true, _) => min,
        (_, true) => max,
        _ => val,
    }
}


#[test]
fn clamp_in_range() {
    let res = clamp(&1, &0, &2);
    assert_eq!(&1, res);
}
