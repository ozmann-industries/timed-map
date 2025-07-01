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
//! ### Basic Usage:
//!
//! #### In `std` environments:
//! ```rs
//! use timed_map::TimedMap;
//! use std::time::Duration;
//!
//! let mut map = TimedMap::new();
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
//!     // Hardware-specific implementation to measure the elapsed time.
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
//!
//! ### Advanced Usage & Tuning:
//!
//! #### Customizing the Internal Map
//!
//! By default, `TimedMap` uses `BTreeMap` to store data, but you can switch to `FxHashMap` or `HashMap`.
//!
//! This is only available on `std` environments.
//!
//! ```rs
//! use timed_map::{MapKind, TimedMap};
//!
//! let mut map = TimedMap::new_with_map_kind(MapKind::FxHashMap);
//! ```
//!
//! #### Manual Expiration Control
//!
//! To have fully control over expired entries, use the `*_unchecked` functions and `drop_expired_entries` to handle expiration manually.
//! This can boost performance by running expiration logic only when it's necessary to maximize the performance.
//!
//! ```rs
//! let mut map = TimedMap::new();
//!
//! map.insert_expirable_unchecked(1, "expirable value", Duration::from_secs(60));
//! assert_eq!(map.get_unchecked(&1), Some(&"expirable value"));
//!
//! map.insert_constant_unchecked(2, "constant value");
//! assert_eq!(map.get_unchecked(&2), Some(&"constant value"));
//!
//! map.drop_expired_entries();
//! ```
//!
//! #### Setting Expiration Check Frequency
//!
//! In cases where inserts are frequent, `expiration_tick_cap` can be set to control how often expired entries are removed. For instance,
//! if there are 100 inserts per second, setting `expiration_tick_cap` to 100 will trigger the expiration check every 100 inserts which will
//! reduce the expiration logic overhead significantly.
//!
//! ```rs
//! use timed_map::TimedMap;
//!
//! let mut map = TimedMap::new().expiration_tick_cap(500);
//! ```

#![no_std]

mod clock;
mod entry;
mod iter;
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

    use std::time::Duration;
    use std::collections::{btree_map, hash_map, BTreeMap, HashMap, BTreeSet};
    use std::hash::Hash;
    use std::vec::Vec;
    use clock::{Clock, StdClock};

    #[cfg(not(feature = "wasm"))]
    use std::time::Instant;

    #[cfg(feature = "wasm")]
    use web_time::Instant;

    pub use map::MapKind;
}

cfg_not_std_feature! {
    extern crate alloc;

    use core::time::Duration;
    use alloc::vec::Vec;
    use alloc::collections::{btree_map, BTreeMap, BTreeSet};

    pub use clock::Clock;
}

use entry::EntryStatus;
use entry::ExpirableEntry;

#[cfg(all(feature = "std", feature = "rustc-hash"))]
use rustc_hash::FxHashMap;

pub use map::TimedMap;
