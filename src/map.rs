#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::marker::PhantomData;

#[cfg(feature = "std")]
use std::time::Duration;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use core::time::Duration;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap;

use crate::clock::Clock;
use crate::entry::EntryStatus;
use crate::entry::ExpirableEntry;

#[cfg(feature = "std")]
use crate::clock::StdClock;

pub struct TimedMap<C, K, V>
where
    C: Clock,
    K: Eq + Copy,
{
    #[cfg(feature = "std")]
    clock: StdClock,
    #[cfg(feature = "std")]
    marker: PhantomData<C>,

    #[cfg(not(feature = "std"))]
    clock: C,

    map: BTreeMap<K, ExpirableEntry<V>>,
    expiries: BTreeMap<u64, K>,
}

#[cfg(feature = "std")]
impl<C: Clock, K: Copy + Eq + Ord, V> Default for TimedMap<C, K, V> {
    fn default() -> Self {
        Self {
            clock: StdClock::default(),
            map: BTreeMap::default(),
            expiries: BTreeMap::default(),
            marker: PhantomData,
        }
    }
}

impl<C: Clock, K: Copy + Eq + Ord, V> TimedMap<C, K, V> {
    #[cfg(feature = "std")]
    /// Creates an empty map.
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(not(feature = "std"))]
    /// Creates an empty map.
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            map: BTreeMap::default(),
            expiries: BTreeMap::default(),
        }
    }

    /// Returns the associated value if present and not expired.
    pub fn get(&self, k: &K) -> Option<&V> {
        self.map
            .get(k)
            .filter(|v| !v.is_expired(&self.clock))
            .map(|v| v.value())
    }

    /// Returns the associated value's [`Duration`] if present and not expired.
    ///
    /// This returns `None` in 2 cases:
    ///  - When entry doesn't exists.
    ///  - When entry is not expirable and is constant.
    pub fn get_remaining_duration(&self, k: &K) -> Option<Duration> {
        self.map
            .get(k)
            .filter(|v| !v.is_expired(&self.clock))
            .map(|v| v.remaining_duration(&self.clock))?
    }

    /// Inserts a key-value pair with an expiration duration. If duration is `None`,
    /// entry will be stored in a non-expirable way.
    ///
    /// If a value already exists for the given key, it will be updated and then
    /// the old one will be returned.
    pub fn insert(&mut self, k: K, v: V, duration: Option<Duration>) -> Option<V> {
        self.drop_expired_entries();

        let entry = ExpirableEntry::new(&self.clock, v, duration);

        if let EntryStatus::ExpiresAtSeconds(expires_at_seconds) = entry.status() {
            self.expiries.insert(*expires_at_seconds, k);
        }

        self.map.insert(k, entry).map(|v| v.owned_value())
    }

    /// Removes a key-value pair from the map and returns the associated value if present
    /// and not expired.
    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.map
            .remove(k)
            .filter(|v| !v.is_expired(&self.clock))
            .map(|v| {
                if let EntryStatus::ExpiresAtSeconds(expires_at_seconds) = v.status() {
                    self.expiries.remove(expires_at_seconds);
                }

                v.owned_value()
            })
    }

    /// Clears expired entries from the map.
    fn drop_expired_entries(&mut self) {
        let now_seconds = self.clock.now_seconds();

        // Iterates through `expiries` in order and drops expired ones.
        //
        // We break the iteration on the first non-expired entry as `expiries`
        // are in sorted order, this makes the process much cheaper than iterating
        // over the entire map.
        while let Some((exp, key)) = self.expiries.pop_first() {
            if exp > now_seconds {
                self.expiries.insert(exp, key);
                break;
            }

            self.map.remove(&key);
        }
    }
}
