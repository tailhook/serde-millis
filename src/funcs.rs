use serde::ser::{Serializer};
use serde::de::{Deserializer};

use traits::{Milliseconds, Sealed};

/// Deserialize function, see crate docs to see how to use it
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: Milliseconds + Sized,
          D: Deserializer<'de>
{
    <T as Sealed>::decode(deserializer)
}

/// Deserialize function, see crate docs to see how to use it
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where T: Milliseconds,
          S: Serializer
{
    Sealed::encode(value, serializer)
}
