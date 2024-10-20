# timed-map

Lightweight map implementation that supports expiring entries and fully
compatible with both `std` and `no_std` environments.

`TimedMap` allows storing key-value pairs with optional expiration times. Expiration is
handled by an implementation of the `Clock` trait, which abstracts time handling for
`no_std` environments.

When `std` feature is enabled (which is the default case), `Clock` trait is handled
automatically from the crate internals with `std::time::SystemTime`.

### Examples:

#### In `std` environments:
```rs
use timed_map::{TimedMap, StdClock};
use std::time::Duration;

let mut map: TimedMap<StdClock, _, _> = TimedMap::new();

map.insert_expirable(1, "expirable value", Duration::from_secs(60));
assert_eq!(map.get(&1), Some(&"expirable value"));
assert!(map.get_remaining_duration(&1).is_some());

map.insert_constant(2, "constant value");
assert_eq!(map.get(&2), Some(&"constant value"));
assert!(map.get_remaining_duration(&2).is_none());
```

#### In `no_std` environments:
```rs
use core::time::Duration;
use timed_map::{Clock, TimedMap};

struct CustomClock;

impl Clock for CustomClock {
    fn now_seconds(&self) -> u64 {
        // Custom time implementation depending on the hardware.
    }
}

let clock = CustomClock;
let mut map = TimedMap::new(clock);

map.insert_expirable(1, "expirable value", Duration::from_secs(60));
assert_eq!(map.get(&1), Some(&"expirable value"));
assert!(map.get_remaining_duration(&1).is_some());

map.insert_constant(2, "constant value");
assert_eq!(map.get(&2), Some(&"constant value"));
assert!(map.get_remaining_duration(&2).is_none());
```