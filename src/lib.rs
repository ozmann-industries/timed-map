//! Lightweight map implementation that supports expiring entries and fully
//! compatible with both `std` and `no_std` environments.
//!
//! `TimedMap` allows storing key-value pairs with optional expiration times. Expiration is
//! handled by an implementation of the `Clock` trait, which abstracts time handling for
//! `no_std` environments.
//!
//! When `std` feature is enabled (which is the default case), `Clock` trait is handled
//! automatically from the crate internals with `std::time::SystemTime`.
//!
//! ### Examples:
//!
//! #### In `std` environment:
//! ```no_run
//! use timed_map::TimedMap;
//! use std::time::Duration;
//!
//! let mut map = TimedMap::new();
//!
//! map.insert(1, "value", Some(Duration::from_secs(60)));
//! assert_eq!(map.get(&1), Some(&"value"));
//! ```
//!
//! #### In `no_std` environment:
//! ```no_run
//! use timed_map::{Clock, TimedMap};
//!
//! struct CustomClock;
//!
//! impl Clock for CustomClock {
//!     fn now_seconds(&self) -> u64 {
//!         // Custom time implementation depending on the hardware.
//!     }
//! }
//!
//! let clock = CustomClock;
//! let mut map = TimedMap::new(clock);
//!
//! map.insert(1, "value", None);
//! assert_eq!(map.get(&1), Some(&"value"));
//! ```

#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod clock;
mod entry;
mod map;

#[cfg(not(feature = "std"))]
pub use clock::Clock;

pub use map::TimedMap;
