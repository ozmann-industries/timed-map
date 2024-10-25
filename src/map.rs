use super::*;

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

cfg_not_std_feature! {
    trait GenericKey: Copy + Eq + Ord {}
    impl<T: Copy + Eq + Ord> GenericKey for T {}
}

cfg_std_feature! {
    trait GenericKey: Copy + Eq + Ord + Hash {}
    impl<T: Copy + Eq + Ord + Hash> GenericKey for T {}
}

enum GenericMap<K, V> {
    BTreeMap(BTreeMap<K, V>),
}

impl<K, V> Default for GenericMap<K, V> {
    fn default() -> Self {
        Self::BTreeMap(BTreeMap::default())
    }
}

impl<K, V> GenericMap<K, V>
where
    K: GenericKey,
{
    fn get(&self, k: &K) -> Option<&V> {
        match self {
            Self::BTreeMap(inner) => inner.get(k),
        }
    }

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self {
            Self::BTreeMap(inner) => inner.insert(k, v),
        }
    }

    fn remove(&mut self, k: &K) -> Option<V> {
        match self {
            Self::BTreeMap(inner) => inner.remove(k),
        }
    }
}

/// Associates keys of type `K` with values of type `V`. Each entry may optionally expire after a
/// specified duration.
///
/// Mutable functions automatically clears expired entries when called.
///
/// If no expiration is set, the entry remains constant.
pub struct TimedMap<C, K, V> {
    #[cfg(feature = "std")]
    clock: StdClock,
    #[cfg(feature = "std")]
    marker: PhantomData<C>,

    #[cfg(not(feature = "std"))]
    clock: C,

    map: GenericMap<K, ExpirableEntry<V>>,
    expiries: BTreeMap<u64, K>,
}

#[cfg(feature = "std")]
impl<C, K, V> Default for TimedMap<C, K, V> {
    fn default() -> Self {
        Self {
            clock: StdClock::default(),
            map: GenericMap::default(),
            expiries: BTreeMap::default(),
            marker: PhantomData,
        }
    }
}

#[allow(private_bounds)]
impl<C, K, V> TimedMap<C, K, V>
where
    C: Clock,
    K: GenericKey,
{
    /// Creates an empty map.
    #[inline(always)]
    #[cfg(feature = "std")]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty `TimedMap`.
    ///
    /// Uses the provided `clock` to handle expiration times.
    #[inline(always)]
    #[cfg(not(feature = "std"))]
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            map: GenericMap::default(),
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

    /// Returns the associated value's `Duration` if present and not expired.
    ///
    /// Returns `None` if the entry does not exist or is constant.
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
    fn insert(&mut self, k: K, v: V, duration: Option<Duration>) -> Option<V> {
        self.drop_expired_entries();

        let entry = ExpirableEntry::new(&self.clock, v, duration);

        if let EntryStatus::ExpiresAtSeconds(expires_at_seconds) = entry.status() {
            self.expiries.insert(*expires_at_seconds, k);
        }

        self.map.insert(k, entry).map(|v| v.owned_value())
    }

    /// Inserts a key-value pair with an expiration duration.
    ///
    /// If a value already exists for the given key, it will be updated and then
    /// the old one will be returned.
    #[inline(always)]
    pub fn insert_expirable(&mut self, k: K, v: V, duration: Duration) -> Option<V> {
        self.insert(k, v, Some(duration))
    }

    /// Inserts a key-value pair with that doesn't expire.
    ///
    /// If a value already exists for the given key, it will be updated and then
    /// the old one will be returned.
    #[inline(always)]
    pub fn insert_constant(&mut self, k: K, v: V) -> Option<V> {
        self.insert(k, v, None)
    }

    /// Removes a key-value pair from the map and returns the associated value if present
    /// and not expired.
    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.drop_expired_entries();

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

#[cfg(test)]
#[cfg(not(feature = "std"))]
mod tests {
    use super::*;

    struct MockClock {
        current_time: u64,
    }

    impl Clock for MockClock {
        fn now_seconds(&self) -> u64 {
            self.current_time
        }
    }

    #[test]
    fn nostd_insert_and_get_constant_entry() {
        let clock = MockClock { current_time: 1000 };
        let mut map: TimedMap<MockClock, u32, &str> = TimedMap::new(clock);

        map.insert_constant(1, "constant value");

        assert_eq!(map.get(&1), Some(&"constant value"));
        assert_eq!(map.get_remaining_duration(&1), None);
    }

    #[test]
    fn nostd_insert_and_get_expirable_entry() {
        let clock = MockClock { current_time: 1000 };
        let mut map: TimedMap<MockClock, u32, &str> = TimedMap::new(clock);
        let duration = Duration::from_secs(60);

        map.insert_expirable(1, "expirable value", duration);

        assert_eq!(map.get(&1), Some(&"expirable value"));
        assert_eq!(map.get_remaining_duration(&1), Some(duration));
    }

    #[test]
    fn nostd_expired_entry() {
        let clock = MockClock { current_time: 1000 };
        let mut map: TimedMap<MockClock, u32, &str> = TimedMap::new(clock);
        let duration = Duration::from_secs(60);

        // Insert entry that expires in 60 seconds
        map.insert_expirable(1, "expirable value", duration);

        // Simulate time passage beyond expiration
        let clock = MockClock { current_time: 1070 };
        map.clock = clock;

        // The entry should be considered expired
        assert_eq!(map.get(&1), None);
        assert_eq!(map.get_remaining_duration(&1), None);
    }

    #[test]
    fn nostd_remove_entry() {
        let clock = MockClock { current_time: 1000 };
        let mut map: TimedMap<MockClock, u32, &str> = TimedMap::new(clock);

        map.insert_constant(1, "constant value");

        assert_eq!(map.remove(&1), Some("constant value"));
        assert_eq!(map.get(&1), None);
    }

    #[test]
    fn nostd_drop_expired_entries() {
        let clock = MockClock { current_time: 1000 };
        let mut map: TimedMap<MockClock, u32, &str> = TimedMap::new(clock);

        // Insert one constant and 2 expirable entries
        map.insert_expirable(1, "expirable value1", Duration::from_secs(50));
        map.insert_expirable(2, "expirable value2", Duration::from_secs(70));
        map.insert_constant(3, "constant value");

        // Simulate time passage beyond the expiration of the first entry
        let clock = MockClock { current_time: 1055 };
        map.clock = clock;

        // Entry 1 should be removed and entry 2 and 3 should still exist
        assert_eq!(map.get(&1), None);
        assert_eq!(map.get(&2), Some(&"expirable value2"));
        assert_eq!(map.get(&3), Some(&"constant value"));

        // Simulate time passage again to expire second expirable entry
        let clock = MockClock { current_time: 1071 };
        map.clock = clock;

        assert_eq!(map.get(&1), None);
        assert_eq!(map.get(&2), None);
        assert_eq!(map.get(&3), Some(&"constant value"));
    }

    #[test]
    fn nostd_update_existing_entry() {
        let clock = MockClock { current_time: 1000 };
        let mut map: TimedMap<MockClock, u32, &str> = TimedMap::new(clock);

        map.insert_constant(1, "initial value");
        assert_eq!(map.get(&1), Some(&"initial value"));

        // Update the value of the existing key and make it expirable
        map.insert_expirable(1, "updated value", Duration::from_secs(15));
        assert_eq!(map.get(&1), Some(&"updated value"));

        // Simulate time passage and expire the updated entry
        let clock = MockClock { current_time: 1016 };
        map.clock = clock;

        assert_eq!(map.get(&1), None);
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod std_tests {
    use super::*;

    #[test]
    fn std_expirable_and_constant_entries() {
        let mut map: TimedMap<StdClock, u32, &str> = TimedMap::new();

        map.insert_constant(1, "constant value");
        map.insert_expirable(2, "expirable value", Duration::from_secs(2));

        assert_eq!(map.get(&1), Some(&"constant value"));
        assert_eq!(map.get(&2), Some(&"expirable value"));

        assert_eq!(map.get_remaining_duration(&1), None);
        assert!(map.get_remaining_duration(&2).is_some());
    }

    #[test]
    fn std_expired_entry_removal() {
        let mut map: TimedMap<StdClock, u32, &str> = TimedMap::new();
        let duration = Duration::from_secs(2);

        map.insert_expirable(1, "expirable value", duration);

        // Wait for expiration
        std::thread::sleep(Duration::from_secs(3));

        // Entry should now be expired
        assert_eq!(map.get(&1), None);
        assert_eq!(map.get_remaining_duration(&1), None);
    }

    #[test]
    fn std_remove_entry() {
        let mut map: TimedMap<StdClock, _, _> = TimedMap::new();

        map.insert_constant(1, "constant value");
        map.insert_expirable(2, "expirable value", Duration::from_secs(2));

        assert_eq!(map.remove(&1), Some("constant value"));
        assert_eq!(map.remove(&2), Some("expirable value"));

        assert_eq!(map.get(&1), None);
        assert_eq!(map.get(&2), None);
    }

    #[test]
    fn std_drop_expired_entries() {
        let mut map: TimedMap<StdClock, u32, &str> = TimedMap::new();

        map.insert_expirable(1, "expirable value1", Duration::from_secs(2));
        map.insert_expirable(2, "expirable value2", Duration::from_secs(4));

        // Wait for expiration
        std::thread::sleep(Duration::from_secs(3));

        // Entry 1 should be removed and entry 2 should still exist
        assert_eq!(map.get(&1), None);
        assert_eq!(map.get(&2), Some(&"expirable value2"));
    }

    #[test]
    fn std_update_existing_entry() {
        let mut map: TimedMap<StdClock, u32, &str> = TimedMap::new();

        map.insert_constant(1, "initial value");
        assert_eq!(map.get(&1), Some(&"initial value"));

        // Update the value of the existing key and make it expirable
        map.insert_expirable(1, "updated value", Duration::from_secs(1));
        assert_eq!(map.get(&1), Some(&"updated value"));

        std::thread::sleep(Duration::from_secs(2));

        // Should be expired now
        assert_eq!(map.get(&1), None);
    }

    #[test]
    fn std_insert_constant_and_expirable_combined() {
        let mut map: TimedMap<StdClock, u32, &str> = TimedMap::new();

        // Insert a constant entry and an expirable entry
        map.insert_constant(1, "constant value");
        map.insert_expirable(2, "expirable value", Duration::from_secs(2));

        // Check both entries exist
        assert_eq!(map.get(&1), Some(&"constant value"));
        assert_eq!(map.get(&2), Some(&"expirable value"));

        // Simulate passage of time beyond expiration
        std::thread::sleep(Duration::from_secs(3));

        // Constant entry should still exist, expirable should be expired
        assert_eq!(map.get(&1), Some(&"constant value"));
        assert_eq!(map.get(&2), None);
    }

    #[test]
    fn std_expirable_entry_still_valid_before_expiration() {
        let mut map: TimedMap<StdClock, u32, &str> = TimedMap::new();

        // Insert an expirable entry with a duration of 60 seconds
        map.insert_expirable(1, "expirable value", Duration::from_secs(3));

        // Simulate a short sleep of 30 seconds (still valid)
        std::thread::sleep(Duration::from_secs(2));

        // The entry should still be valid
        assert_eq!(map.get(&1), Some(&"expirable value"));
        assert!(map.get_remaining_duration(&1).unwrap().as_secs() == 1);
    }
}
