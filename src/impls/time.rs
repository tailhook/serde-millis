use super::{Sealed, Milliseconds};

use serde::ser::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer};


impl Sealed for time::Duration {
    fn encode<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.whole_milliseconds().serialize(serializer)
    }
    fn decode<'de, D>(deserializer: D) -> Result<Self, D::Error>
        where Self: Sized,
              D: Deserializer<'de>,
    {
        Ok(Self::milliseconds(Deserialize::deserialize(deserializer)?))
    }
}

impl Milliseconds for time::Duration {}

#[cfg(test)]
mod test {
    use super::Milliseconds;

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

    mod duration {
        use time::Duration;

        use super::*;

        #[test]
        fn serializes() {
            for (input, expected) in &[
                (Duration::zero(), "0"),
                (Duration::nanoseconds(1), "0"),
                (Duration::nanoseconds(-1), "0"),
                (Duration::microseconds(134), "0"),
                (Duration::microseconds(2000), "2"),
                (Duration::milliseconds(1), "1"),
                (Duration::seconds(1234), "1234000"),
                (Duration::seconds(-1234), "-1234000"),
            ] {
                assert_eq!(encode(*input), *expected);
            }
        }

        #[test]
        fn deserializes() {
            for (input, expected) in &[
                ("0", Duration::zero()),
                ("1002", Duration::milliseconds(1002)),
                ("1000",  Duration::seconds(1)),
                ("1234000",  Duration::seconds(1234)),
            ] {
                assert_eq!(decode::<Duration>(*input), *expected);
            }
        }
    }
}
