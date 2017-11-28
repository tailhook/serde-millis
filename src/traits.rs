pub trait Sealed {
    fn to_millis(&self) -> Option<u64>;
    fn from_millis(val: u64) -> Option<Self> where Self: Sized;
}

/// A value convertible to milliseconds since unix epoch
///
/// This trait is currently sealed. This might change in future.
pub trait Milliseconds: Sealed {
}
