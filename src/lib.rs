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
//! #### In `std` environments:
//! ```rs
//! use timed_map::{TimedMap, StdClock};
//! use std::time::Duration;
//!
//! let mut map: TimedMap<StdClock, _, _> = TimedMap::new();
//!
//! map.insert_expirable(1, "expirable value", Duration::from_secs(60));
//! assert_eq!(map.get(&1), Some(&"expirable value"));
//! assert!(map.get_remaining_duration(&1).is_some());
//!
//! map.insert_constant(2, "constant value");
//! assert_eq!(map.get(&2), Some(&"constant value"));
//! assert!(map.get_remaining_duration(&2).is_none());
//! ```
//!
//! #### In `no_std` environments:
//! ```rs
//! use core::time::Duration;
//! use timed_map::{Clock, TimedMap};
//!
//! struct CustomClock;
//!
//! impl Clock for CustomClock {
//!     fn elapsed_seconds_since_creation(&self) -> u64 {
//!         // Hardware-specific implementation to measure the elapsed time.
//!         0 // placeholder
//!     }
//! }
//!
//! let clock = CustomClock;
//! let mut map = TimedMap::new(clock);
//!
//! map.insert_expirable(1, "expirable value", Duration::from_secs(60));
//! assert_eq!(map.get(&1), Some(&"expirable value"));
//! assert!(map.get_remaining_duration(&1).is_some());
//!
//! map.insert_constant(2, "constant value");
//! assert_eq!(map.get(&2), Some(&"constant value"));
//! assert!(map.get_remaining_duration(&2).is_none());
//! ```

#![no_std]

mod clock;
mod entry;
mod map;

macro_rules! cfg_std_feature {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "std")]
            $item
        )*
    };
}

macro_rules! cfg_not_std_feature {
    ($($item:item)*) => {
        $(
            #[cfg(not(feature = "std"))]
            $item
        )*
    };
}

cfg_std_feature! {
    extern crate std;

    use std::marker::PhantomData;
    use std::time::Duration;
    use std::collections::{BTreeMap, HashMap};
    use std::hash::Hash;
    use clock::Clock;

    #[cfg(not(target_arch = "wasm32"))]
    use std::time::Instant;

    #[cfg(target_arch = "wasm32")]
    use web_time::Instant;

    pub use clock::StdClock;
    pub use map::MapKind;
}

cfg_not_std_feature! {
    extern crate alloc;

    use core::time::Duration;
    use alloc::collections::BTreeMap;

    pub use clock::Clock;
}

use entry::EntryStatus;
use entry::ExpirableEntry;

#[cfg(all(feature = "std", feature = "rustc-hash"))]
use rustc_hash::FxHashMap;

pub use map::TimedMap;
