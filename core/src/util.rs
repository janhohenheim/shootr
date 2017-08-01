extern crate chrono;


use self::chrono::{DateTime, Utc};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use model::game::Vector;
use std::f64::consts;

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


pub fn elapsed_ms(from: DateTime<Utc>, to: DateTime<Utc>) -> Result<u64, ()> {
    let ms = to.signed_duration_since(from).num_milliseconds();
    if ms >= 0 { Ok(ms as u64) } else { Err(()) }
}

pub fn timestamp() -> u64 {
    let now = SystemTime::now();
    let elapsed = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1_000_000
}

pub fn angle(a: &Vector, b: &Vector) -> f64 {
    assert_ne!(*a, *b);
    let d_x = b.x as f64 - a.x as f64;
    // swapped because a positive y means down for us
    let d_y = a.y as f64 - b.y as f64;
    let mut rad = d_y.atan2(d_x);
    while rad < 0.0 {
        rad += consts::PI * 2.0
    }
    rad.to_degrees()
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

pub type SeqId = u32;
#[derive(Default)]
pub struct SeqIdGen {
    curr_id: SeqId,
}
impl SeqIdGen {
    pub fn gen(&mut self) -> SeqId {
        self.curr_id += 1;
        self.curr_id
    }
}


#[macro_export]
macro_rules! newtype {
    (  $name:ident($type:ty)  ) => {
        pub struct $name(pub $type);
        add_impl!($name, $type);
    };
    (  $name:ident($type:ty) : $($derives:meta), + ) => {
        #[derive(
            $($derives,)+
        )]
        pub struct $name(pub $type);
        add_impl!($name, $type);
    };
}

macro_rules! add_impl {
     (  $name:ident, $type:ty ) => {
         impl Deref for $name {
            type Target = $type;

            fn deref(&self) -> &$type {
                &self.0
            }
        }
        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut $type {
                &mut self.0
            }
        }
        impl From<$type> for $name {
            fn from(t: $type) -> Self {
                $name(t)
            }
        }
     };
}

#[cfg(test)]
mod test {
    use super::*;
    use util::chrono::TimeZone;

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


    #[test]
    fn one_elapsed_ms() {
        let a = Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 0, 0);
        let b = Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 0, 1);
        assert_eq!(1, elapsed_ms(a, b).unwrap());
    }

    #[test]
    fn one_elapsed_second() {
        let a = Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 0, 0);
        let b = Utc.ymd(1970, 1, 1).and_hms(0, 0, 1);
        assert_eq!(1000, elapsed_ms(a, b).unwrap());
    }

    #[test]
    fn no_elapsed_time() {
        let a = Utc.ymd(1970, 1, 1).and_hms(0, 0, 1);
        assert_eq!(0, elapsed_ms(a, a).unwrap());
    }


    #[test]
    fn negative_elapsed_time() {
        let a = Utc.ymd(1970, 1, 1).and_hms(0, 0, 1);
        let b = Utc.ymd(1970, 1, 1).and_hms(0, 0, 0);
        assert!(elapsed_ms(a, b).is_err());
    }

    #[test]
    fn timestamp_wait() {
        use std::thread;
        use std::time::Duration;

        let a = timestamp();
        let delta = 420;
        thread::sleep(Duration::from_millis(delta + 10));
        let b = timestamp();
        assert!(b - a >= delta)
    }


    #[test]
    #[should_panic]
    fn angle_same() {
        let a = Vector { x: 0, y: 0 };
        let b = a.clone();
        angle(&a, &b);
    }

    #[test]
    fn angle_right() {
        let a = Vector { x: 0, y: 0 };
        let b = Vector { x: 1, y: 0 };
        assert_eq!(0.0, angle(&a, &b));
    }


    #[test]
    fn angle_down() {
        let a = Vector { x: 0, y: 0 };
        let b = Vector { x: 0, y: 1 };
        assert_eq!(270.0, angle(&a, &b));
    }


    #[test]
    fn angle_left() {
        let a = Vector { x: 0, y: 0 };
        let b = Vector { x: -1, y: 0 };
        assert_eq!(180.0, angle(&a, &b));
    }


    #[test]
    fn angle_up() {
        let a = Vector { x: 0, y: 0 };
        let b = Vector { x: 0, y: -1 };
        assert_eq!(90.0, angle(&a, &b));
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


    #[test]
    fn seq_id_gen_sequential() {
        let mut id_gen = SeqIdGen::default();
        let ids_count = 1000;
        for i in 0..ids_count {
            assert_eq!(i + 1, id_gen.gen());
        }
    }
}
