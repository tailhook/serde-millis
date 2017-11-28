use serde::ser::{Serialize, Serializer, Error as SerError};
use serde::de::{Deserialize, Deserializer, Error};

use traits::Milliseconds;

/// Deserialize function, see crate docs to see how to use it
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: Milliseconds + Sized,
          D: Deserializer<'de>
{
    let val: u64 = Deserialize::deserialize(deserializer)?;
    match T::from_millis(val) {
        Some(x) => Ok(x),
        None => Err(D::Error::custom("millisecond value is out of range")),
    }
}

/// Deserialize function, see crate docs to see how to use it
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where T: Milliseconds,
          S: Serializer
{
    match value.to_millis() {
        Some(x) => x.serialize(serializer),
        None => Err(S::Error::custom("millisecond value is out of range")),
    }
}
