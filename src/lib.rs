//! # Serde Millis
//!
//! [Documentation](https://docs.rs/serde_millis) |
//! [Github](https://github.com/tailhook/serde-millis) |
//! [Crate](https://crates.io/crates/serde_millis)
//!
//! A serde wrapper, that can be used to serialize timestamps and durations as
//! milliseconds. It's often useful together with `serde_json` to communicate
//! with Javascript.
//!
//! # Example
//!
//! ```rust
//! #[macro_use]
//! extern crate serde_derive;
//!
//! extern crate serde;
//! extern crate serde_millis;
//!
//! use std::time::{Duration, SystemTime, Instant};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Timestamps {
//!     #[serde(with = "serde_millis")]
//!     time: SystemTime,
//!
//!     #[serde(with = "serde_millis")]
//!     latency: Duration,
//!
//!     // Instant is serialized relative to the current time, or in
//!     // other words by formula (but works for future instants too):
//!     //
//!     //   ts = SystemTime::now() - (Instant::now() - target_instant)
//!     //
//!     #[serde(with = "serde_millis")]
//!     timestamp: Instant,
//! }
//!
//! #
//! # fn main() {}
//! ```
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
extern crate serde;

mod traits;
mod funcs;
mod impls;

pub use traits::Milliseconds;
pub use funcs::{serialize, deserialize};
