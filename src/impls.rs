use std::fmt;
use std::time::{Duration, SystemTime, Instant, UNIX_EPOCH};
use std::marker::PhantomData;

use serde::ser::{Serialize, Serializer, Error as SerError};
use serde::de::{Deserialize, Deserializer, Error as DeError, Visitor};

use traits::{Sealed, Milliseconds};


impl Sealed for Duration {
    fn encode<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.as_secs().checked_mul(1000).and_then(|x| {
            x.checked_add((self.subsec_nanos()/1_000_000) as u64)
        })
        .ok_or_else(|| S::Error::custom("duration value out of range"))
        .and_then(|v| v.serialize(serializer))
    }
    fn decode<'de, D>(deserializer: D) -> Result<Self, D::Error>
        where Self: Sized,
              D: Deserializer<'de>,
    {
        let val = Deserialize::deserialize(deserializer)?;
        Ok(Duration::from_millis(val))
    }
}
impl Sealed for SystemTime {
    fn encode<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.duration_since(UNIX_EPOCH)
            .map_err(|_| S::Error::custom("invalid system time"))
            .and_then(|x| x.encode(serializer))
    }
    fn decode<'de, D>(deserializer: D) -> Result<Self, D::Error>
        where Self: Sized,
              D: Deserializer<'de>,
    {
        let val = Duration::decode(deserializer)?;
        Ok((UNIX_EPOCH + val))
    }
}
impl Sealed for Instant {
    fn encode<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let inow = Instant::now();
        let snow = SystemTime::now();
        if *self < inow {
            (snow - inow.duration_since(*self)).encode(serializer)
        } else {
            (snow + self.duration_since(inow)).encode(serializer)
        }
    }
    fn decode<'de, D>(deserializer: D) -> Result<Self, D::Error>
        where Self: Sized,
              D: Deserializer<'de>,
    {
        let inow = Instant::now();
        let snow = SystemTime::now();
        let stime = UNIX_EPOCH + Duration::decode(deserializer)?;
        if stime > snow {
            stime.duration_since(snow)
                .map_err(|_| D::Error::custom("instant out of range"))
                .map(|x| inow + x)
        } else {
            snow.duration_since(stime)
                .map_err(|_| D::Error::custom("instant out of range"))
                .map(|x| inow - x)
        }
    }
}

impl<T: Sealed> Sealed for Option<T> {
    fn encode<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        struct Data<'a, V: 'a>(&'a V) where V: Sealed;

        impl<'a, V: Sealed + 'a> Serialize for Data<'a, V> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
            {
                self.0.encode(serializer)
            }
        }

        match *self {
            Some(ref value) => serializer.serialize_some(&Data(value)),
            None => serializer.serialize_none(),
        }
    }
    fn decode<'de, D>(deserializer: D) -> Result<Self, D::Error>
        where Self: Sized,
              D: Deserializer<'de>,
    {
        struct OptionVisitor<T> {
            marker: PhantomData<T>,
        }

        impl<'de, T: Sealed> Visitor<'de> for OptionVisitor<T> {
            type Value = Option<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result
            {
                formatter.write_str("option")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Option<T>, E>
                where E: DeError,
            {
                Ok(None)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Option<T>, E>
                where E: DeError,
            {
                Ok(None)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D)
                -> Result<Option<T>, D::Error>
                where D: Deserializer<'de>,
            {
                T::decode(deserializer).map(Some)
            }
        }

        deserializer.deserialize_option(OptionVisitor { marker: PhantomData })
    }
}

impl Milliseconds for Duration {}
impl Milliseconds for SystemTime {}
impl Milliseconds for Instant {}
impl<T: Milliseconds> Milliseconds for Option<T> {}


#[cfg(test)]
mod test {
    use std::time::{Duration, SystemTime, Instant, UNIX_EPOCH};

    use serde_json;

    use traits::Milliseconds;

    fn encode<V: Milliseconds>(v: V) -> String {
        #[derive(Serialize)]
        struct Data<V: Milliseconds>(#[serde(with="::funcs")]V);
        serde_json::to_string(&Data(v)).unwrap()
    }

    fn decode<V: Milliseconds>(src: &str) -> V {
        #[derive(Deserialize)]
        struct Data<V: Milliseconds>(#[serde(with="::funcs")]V);
        let data: Data<V> = serde_json::from_str(src).unwrap();
        return data.0
    }

    #[test]
    fn test_duration() {
        assert_eq!(encode(Duration::new(1, 0)), "1000");
        assert_eq!(encode(Duration::new(1234, 0)), "1234000");
        assert_eq!(encode(Duration::new(1, 2300000)), "1002");
        assert_eq!(decode::<Duration>("1002"), Duration::new(1, 2000000));
        assert_eq!(decode::<Duration>("1000"), Duration::new(1, 0));
        assert_eq!(decode::<Duration>("1234000"), Duration::new(1234, 0));
    }

    #[test]
    fn test_systemtime_past() {
        assert_eq!(encode(UNIX_EPOCH + Duration::from_millis(1511885454870)),
            "1511885454870");
        assert_eq!(decode::<SystemTime>("1511885454870"),
            UNIX_EPOCH + Duration::from_millis(1511885454870));
    }

    #[test]
    fn test_systemtime_future() {
        assert_eq!(encode(UNIX_EPOCH + Duration::from_millis(2147483647000)),
            "2147483647000");
        assert_eq!(decode::<SystemTime>("2147483647000"),
            UNIX_EPOCH + Duration::from_millis(2147483647000));
    }

    #[test]
    fn test_instant_past() {
        let early = Instant::now();
        let converted = decode::<Instant>(
            &encode(Instant::now() - Duration::from_millis(2000)));
        let late = Instant::now();
        assert!(converted >= early - Duration::from_millis(2002));
        assert!(converted <= late - Duration::from_millis(1998));
    }

    #[test]
    fn test_instant_future() {
        let early = Instant::now();
        let converted = decode::<Instant>(
            &encode(Instant::now() + Duration::from_millis(2000)));
        let late = Instant::now();
        assert!(converted >= early + Duration::from_millis(1998));
        assert!(converted <= late + Duration::from_millis(2002));
    }

    #[test]
    fn test_option() {
        assert_eq!(encode(Some(Duration::new(1, 0))), "1000");
        assert_eq!(encode(None::<Duration>), "null");
        assert_eq!(decode::<Option<Duration>>("null"), None);
        assert_eq!(decode::<Option<Duration>>("1000"),
            Some(Duration::new(1, 0)));
        assert_eq!(encode(Some(
            UNIX_EPOCH + Duration::from_millis(1511885454870))),
            "1511885454870");
        assert_eq!(encode(None::<SystemTime>), "null");
        assert_eq!(decode::<Option<SystemTime>>("null"), None);
        assert_eq!(decode::<Option<SystemTime>>("1511885454870"),
            Some(UNIX_EPOCH + Duration::from_millis(1511885454870)));

        let early = SystemTime::now();
        let converted = decode::<Option<SystemTime>>(
            &encode(Some(Instant::now())));
        let later = SystemTime::now();
        assert!(converted.unwrap() >= early - Duration::from_millis(1));
        assert!(converted.unwrap() <= later);

        let early = Instant::now();
        let converted = decode::<Option<Instant>>(
            &encode(Some(SystemTime::now())));
        let later = Instant::now();
        assert!(converted.unwrap() >= early - Duration::from_millis(1));
        assert!(converted.unwrap() <= later);

        let converted = decode::<Option<Instant>>("null");
        assert!(converted.is_none());
    }
}
