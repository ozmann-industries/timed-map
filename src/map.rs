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
    /// Generic trait for `no_std` keys that is gated by the `std` feature
    /// and handled at compile time.
    pub trait GenericKey: Clone + Eq + Ord {}
    impl<T: Clone + Eq + Ord> GenericKey for T {}
}

cfg_std_feature! {
    /// Generic trait for `std` keys that is gated by the `std` feature
    /// and handled at compile time.
    pub trait GenericKey: Clone + Eq + Ord + Hash {}
    impl<T: Clone + Eq + Ord + Hash> GenericKey for T {}
}

/// Wraps different map implementations and provides a single interface to access them.
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
enum GenericMap<K, V> {
    BTreeMap(BTreeMap<K, V>),
    #[cfg(feature = "std")]
    HashMap(HashMap<K, V>),
    #[cfg(all(feature = "std", feature = "rustc-hash"))]
    FxHashMap(FxHashMap<K, V>),
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
    #[inline(always)]
    fn get(&self, k: &K) -> Option<&V> {
        match self {
            Self::BTreeMap(inner) => inner.get(k),
            #[cfg(feature = "std")]
            Self::HashMap(inner) => inner.get(k),
            #[cfg(all(feature = "std", feature = "rustc-hash"))]
            Self::FxHashMap(inner) => inner.get(k),
        }
    }

    #[inline(always)]
    fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        match self {
            Self::BTreeMap(inner) => inner.get_mut(k),
            #[cfg(feature = "std")]
            Self::HashMap(inner) => inner.get_mut(k),
            #[cfg(all(feature = "std", feature = "rustc-hash"))]
            Self::FxHashMap(inner) => inner.get_mut(k),
        }
    }

    #[inline(always)]
    fn len(&self) -> usize {
        match self {
            Self::BTreeMap(inner) => inner.len(),
            #[cfg(feature = "std")]
            Self::HashMap(inner) => inner.len(),
            #[cfg(all(feature = "std", feature = "rustc-hash"))]
            Self::FxHashMap(inner) => inner.len(),
        }
    }

    #[inline(always)]
    fn keys(&self) -> Vec<K> {
        match self {
            Self::BTreeMap(inner) => inner.keys().cloned().collect(),
            #[cfg(feature = "std")]
            Self::HashMap(inner) => inner.keys().cloned().collect(),
            #[cfg(all(feature = "std", feature = "rustc-hash"))]
            Self::FxHashMap(inner) => inner.keys().cloned().collect(),
        }
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        match self {
            Self::BTreeMap(inner) => inner.is_empty(),
            #[cfg(feature = "std")]
            Self::HashMap(inner) => inner.is_empty(),
            #[cfg(all(feature = "std", feature = "rustc-hash"))]
            Self::FxHashMap(inner) => inner.is_empty(),
        }
    }

    #[inline(always)]
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self {
            Self::BTreeMap(inner) => inner.insert(k, v),
            #[cfg(feature = "std")]
            Self::HashMap(inner) => inner.insert(k, v),
            #[cfg(all(feature = "std", feature = "rustc-hash"))]
            Self::FxHashMap(inner) => inner.insert(k, v),
        }
    }

    #[inline(always)]
    fn clear(&mut self) {
        match self {
            Self::BTreeMap(inner) => inner.clear(),
            #[cfg(feature = "std")]
            Self::HashMap(inner) => inner.clear(),
            #[cfg(all(feature = "std", feature = "rustc-hash"))]
            Self::FxHashMap(inner) => inner.clear(),
        }
    }

    #[inline(always)]
    fn remove(&mut self, k: &K) -> Option<V> {
        match self {
            Self::BTreeMap(inner) => inner.remove(k),
            #[cfg(feature = "std")]
            Self::HashMap(inner) => inner.remove(k),
            #[cfg(all(feature = "std", feature = "rustc-hash"))]
            Self::FxHashMap(inner) => inner.remove(k),
        }
    }
}

/// Specifies the inner map implementation for `TimedMap`.
#[cfg(feature = "std")]
#[allow(clippy::enum_variant_names)]
pub enum MapKind {
    BTreeMap,
    HashMap,
    #[cfg(feature = "rustc-hash")]
    FxHashMap,
}

/// Associates keys of type `K` with values of type `V`. Each entry may optionally expire after a
/// specified duration.
///
/// Mutable functions automatically clears expired entries when called.
///
/// If no expiration is set, the entry remains constant.
#[derive(Debug)]
pub struct TimedMap<C, K, V> {
    #[cfg(feature = "std")]
    clock: StdClock,
    #[cfg(feature = "std")]
    marker: PhantomData<C>,

    #[cfg(not(feature = "std"))]
    clock: C,

    map: GenericMap<K, ExpirableEntry<V>>,
    expiries: BTreeMap<u64, BTreeSet<K>>,

    expiration_tick: u16,
    expiration_tick_cap: u16,
}

#[cfg(feature = "std")]
impl<C, K, V> Default for TimedMap<C, K, V> {
    fn default() -> Self {
        Self {
            clock: StdClock::new(),
            map: GenericMap::default(),
            expiries: BTreeMap::default(),
            marker: PhantomData,

            expiration_tick: 0,
            expiration_tick_cap: 1,
        }
    }
}

impl<C, K, V> TimedMap<C, K, V>
where
    C: Clock,
    K: GenericKey,
{
    /// Creates an empty map.
    #[cfg(feature = "std")]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty map based on the chosen map implementation specified by `MapKind`.
    #[cfg(feature = "std")]
    pub fn new_with_map_kind(map_kind: MapKind) -> Self {
        let map = match map_kind {
            MapKind::BTreeMap => GenericMap::<K, ExpirableEntry<V>>::BTreeMap(BTreeMap::default()),
            MapKind::HashMap => GenericMap::HashMap(HashMap::default()),
            #[cfg(feature = "rustc-hash")]
            MapKind::FxHashMap => GenericMap::FxHashMap(FxHashMap::default()),
        };

        Self {
            map,
            clock: StdClock::new(),
            expiries: BTreeMap::default(),

            #[cfg(feature = "std")]
            marker: PhantomData,
            expiration_tick: 0,
            expiration_tick_cap: 1,
        }
    }

    /// Creates an empty `TimedMap`.
    ///
    /// Uses the provided `clock` to handle expiration times.
    #[cfg(not(feature = "std"))]
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            map: GenericMap::default(),
            expiries: BTreeMap::default(),
            expiration_tick: 0,
            expiration_tick_cap: 1,
        }
    }

    /// Configures `expiration_tick_cap`, which sets how often `TimedMap::drop_expired_entries`
    /// is automatically called. The default value is 1.
    ///
    /// On each insert (excluding `unchecked` ones), an internal counter `expiration_tick` is incremented.
    /// When `expiration_tick` meets or exceeds `expiration_tick_cap`, `TimedMap::drop_expired_entries` is
    /// triggered to remove expired entries.
    ///
    /// Use this to control cleanup frequency and optimize performance. For example, if your workload
    /// involves about 100 inserts within couple seconds, setting `expiration_tick_cap` to 100 can improve
    /// the performance significantly.
    #[inline(always)]
    pub fn expiration_tick_cap(mut self, expiration_tick_cap: u16) -> Self {
        self.expiration_tick_cap = expiration_tick_cap;
        self
    }

    /// Returns the associated value if present and not expired.
    ///
    /// To retrieve the value without checking expiration, use `TimedMap::get_unchecked`.
    pub fn get(&self, k: &K) -> Option<&V> {
        self.map
            .get(k)
            .filter(|v| !v.is_expired(self.clock.elapsed_seconds_since_creation()))
            .map(|v| v.value())
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// To retrieve the value without checking expiration, use `TimedMap::get_mut_unchecked`.
    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.map
            .get_mut(k)
            .filter(|v| !v.is_expired(self.clock.elapsed_seconds_since_creation()))
            .map(|v| v.value_mut())
    }

    /// Returns the associated value if present, regardless of whether it is expired.
    ///
    /// If you only want non-expired entries, use `TimedMap::get` instead.
    #[inline(always)]
    pub fn get_unchecked(&self, k: &K) -> Option<&V> {
        self.map.get(k).map(|v| v.value())
    }

    /// Returns a mutable reference to the associated value if present, regardless of
    /// whether it is expired.
    ///
    /// If you only want non-expired entries, use `TimedMap::get_mut` instead.
    #[inline(always)]
    pub fn get_mut_unchecked(&mut self, k: &K) -> Option<&mut V> {
        self.map.get_mut(k).map(|v| v.value_mut())
    }

    /// Returns the associated value's `Duration` if present and not expired.
    ///
    /// Returns `None` if the entry does not exist or is constant.
    pub fn get_remaining_duration(&self, k: &K) -> Option<Duration> {
        match self.map.get(k) {
            Some(v) => {
                let now = self.clock.elapsed_seconds_since_creation();
                if v.is_expired(now) {
                    return None;
                }

                v.remaining_duration(now)
            }
            None => None,
        }
    }

    /// Returns the number of unexpired elements in the map.
    ///
    /// See `TimedMap::len_expired` and `TimedMap::len_unchecked` for other usages.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.map.len() - self.len_expired()
    }

    /// Returns the number of expired elements in the map.
    ///
    /// See `TimedMap::len` and `TimedMap::len_unchecked` for other usages.
    #[inline(always)]
    pub fn len_expired(&self) -> usize {
        let now = self.clock.elapsed_seconds_since_creation();
        self.expiries
            .iter()
            .filter_map(
                |(exp, keys)| {
                    if exp <= &now {
                        Some(keys.len())
                    } else {
                        None
                    }
                },
            )
            .sum()
    }

    /// Returns the total number of elements (including expired ones) in the map.
    ///
    /// See `TimedMap::len` and `TimedMap::len_expired` for other usages.
    #[inline(always)]
    pub fn len_unchecked(&self) -> usize {
        self.map.len()
    }

    /// Returns keys of the map
    #[inline(always)]
    pub fn keys(&self) -> Vec<K> {
        self.map.keys()
    }

    /// Returns true if the map contains no elements.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Inserts a key-value pair with an expiration duration. If duration is `None`,
    /// entry will be stored in a non-expirable way.
    ///
    /// If a value already exists for the given key, it will be updated and then
    /// the old one will be returned.
    #[inline(always)]
    fn insert(&mut self, k: K, v: V, expires_at: Option<u64>) -> Option<V> {
        let entry = ExpirableEntry::new(v, expires_at);
        match self.map.insert(k.clone(), entry) {
            Some(old) => {
                // Remove the old expiry record
                if let EntryStatus::ExpiresAtSeconds(e) = old.status() {
                    self.drop_key_from_expiry(e, &k)
                }

                Some(old.owned_value())
            }
            None => None,
        }
    }

    /// Inserts a key-value pair with an expiration duration, and then drops the
    /// expired entries.
    ///
    /// If a value already exists for the given key, it will be updated and then
    /// the old one will be returned.
    ///
    /// If you don't want to the check expired entries, consider using `TimedMap::insert_expirable_unchecked`
    /// instead.
    pub fn insert_expirable(&mut self, k: K, v: V, duration: Duration) -> Option<V> {
        self.expiration_tick += 1;

        let now = self.clock.elapsed_seconds_since_creation();
        if self.expiration_tick >= self.expiration_tick_cap {
            self.drop_expired_entries_inner(now);
            self.expiration_tick = 0;
        }

        let expires_at = now + duration.as_secs();

        let res = self.insert(k.clone(), v, Some(expires_at));

        self.expiries.entry(expires_at).or_default().insert(k);

        res
    }

    /// Inserts a key-value pair with an expiration duration, without checking the expired
    /// entries.
    ///
    /// If a value already exists for the given key, it will be updated and then
    /// the old one will be returned.
    ///
    /// If you want to check the expired entries, consider using `TimedMap::insert_expirable`
    /// instead.
    pub fn insert_expirable_unchecked(&mut self, k: K, v: V, duration: Duration) -> Option<V> {
        let now = self.clock.elapsed_seconds_since_creation();
        let expires_at = now + duration.as_secs();
        self.insert(k, v, Some(expires_at))
    }

    /// Inserts a key-value pair with that doesn't expire, and then drops the
    /// expired entries.
    ///
    /// If a value already exists for the given key, it will be updated and then
    /// the old one will be returned.
    ///
    /// If you don't want to check the expired entries, consider using `TimedMap::insert_constant_unchecked`
    /// instead.
    pub fn insert_constant(&mut self, k: K, v: V) -> Option<V> {
        self.expiration_tick += 1;

        let now = self.clock.elapsed_seconds_since_creation();
        if self.expiration_tick >= self.expiration_tick_cap {
            self.drop_expired_entries_inner(now);
            self.expiration_tick = 0;
        }

        self.insert(k, v, None)
    }

    /// Inserts a key-value pair with that doesn't expire without checking the expired
    /// entries.
    ///
    /// If a value already exists for the given key, it will be updated and then
    /// the old one will be returned.
    ///
    /// If you want to check the expired entries, consider using `TimedMap::insert_constant`
    /// instead.
    pub fn insert_constant_unchecked(&mut self, k: K, v: V) -> Option<V> {
        self.expiration_tick += 1;
        self.insert(k, v, None)
    }

    /// Removes a key-value pair from the map and returns the associated value if present
    /// and not expired.
    ///
    /// If you want to retrieve the entry after removal even if it is expired, consider using
    /// `TimedMap::remove_unchecked`.
    #[inline(always)]
    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.map
            .remove(k)
            .filter(|v| {
                if let EntryStatus::ExpiresAtSeconds(expires_at_seconds) = v.status() {
                    self.drop_key_from_expiry(expires_at_seconds, k);
                }

                !v.is_expired(self.clock.elapsed_seconds_since_creation())
            })
            .map(|v| v.owned_value())
    }

    /// Removes a key-value pair from the map and returns the associated value if present,
    /// regardless of expiration status.
    ///
    /// If you only want the entry when it is not expired, consider using `TimedMap::remove`.
    #[inline(always)]
    pub fn remove_unchecked(&mut self, k: &K) -> Option<V> {
        self.map
            .remove(k)
            .filter(|v| {
                if let EntryStatus::ExpiresAtSeconds(expires_at_seconds) = v.status() {
                    self.drop_key_from_expiry(expires_at_seconds, k);
                }

                true
            })
            .map(|v| v.owned_value())
    }

    /// Clears the map, removing all elements.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.map.clear()
    }

    /// Updates the expiration status of an entry and returns the old status.
    ///
    /// If the entry does not exist, returns `Err(TimedMapError::EntryNotFound)`.
    /// If the entry's old status is `EntryStatus::Constant`, returns None
    pub fn update_expiration_status(
        &mut self,
        key: K,
        duration: Duration,
    ) -> Result<Option<EntryStatus>, &'static str> {
        match self.map.get_mut(&key) {
            Some(entry) => {
                let old_status = *entry.status();
                let now = self.clock.elapsed_seconds_since_creation();
                let expires_at = now + duration.as_secs();
                entry.update_status(EntryStatus::ExpiresAtSeconds(expires_at));

                let res = match &old_status {
                    EntryStatus::Constant => None,
                    EntryStatus::ExpiresAtSeconds(t) => {
                        self.drop_key_from_expiry(t, &key);
                        Some(old_status)
                    }
                };

                self.expiries
                    .entry(expires_at)
                    .or_default()
                    .insert(key.clone());

                Ok(res)
            }
            None => Err("entry not found"),
        }
    }

    /// Clears expired entries from the map.
    ///
    /// Call this function when using `*_unchecked` inserts, as these do not
    /// automatically clear expired entries.
    #[inline(always)]
    pub fn drop_expired_entries(&mut self) {
        let now = self.clock.elapsed_seconds_since_creation();
        self.drop_expired_entries_inner(now);
    }

    fn drop_expired_entries_inner(&mut self, now: u64) {
        // Iterates through `expiries` in order and drops expired ones.
        while let Some((exp, keys)) = self.expiries.pop_first() {
            // It's safe to do early-break here as keys are sorted by expiration.
            if exp > now {
                self.expiries.insert(exp, keys);
                break;
            }

            for key in keys {
                self.map.remove(&key);
            }
        }
    }

    fn drop_key_from_expiry(&mut self, expiry_key: &u64, map_key: &K) {
        if let Some(list) = self.expiries.get_mut(expiry_key) {
            list.remove(map_key);

            if list.is_empty() {
                self.expiries.remove(expiry_key);
            }
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
        fn elapsed_seconds_since_creation(&self) -> u64 {
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

    #[test]
    fn nostd_update_expirable_entry_status() {
        let clock = MockClock { current_time: 1000 };
        let mut map: TimedMap<MockClock, u32, &str> = TimedMap::new(clock);

        map.insert_constant(1, "initial value");
        assert_eq!(map.get(&1), Some(&"initial value"));

        // Update the value of the existing key and make it expirable
        map.update_expiration_status(1, Duration::from_secs(16))
            .expect("shouldn't fails");
        assert_eq!(map.get(&1), Some(&"initial value"));

        // Simulate time passage and expire the updated entry
        let clock = MockClock { current_time: 1017 };
        map.clock = clock;
        assert_eq!(map.get(&1), None);
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod std_tests {
    use core::ops::Add;

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

        // Insert an expirable entry with a duration of 3 seconds
        map.insert_expirable(1, "expirable value", Duration::from_secs(3));

        // Simulate a short sleep of 2 seconds (still valid)
        std::thread::sleep(Duration::from_secs(2));

        // The entry should still be valid
        assert_eq!(map.get(&1), Some(&"expirable value"));
        assert!(map.get_remaining_duration(&1).unwrap().as_secs() == 1);
    }

    #[test]
    fn std_length_functions() {
        let mut map: TimedMap<StdClock, u32, &str> = TimedMap::new();

        map.insert_expirable(1, "expirable value", Duration::from_secs(1));
        map.insert_expirable(2, "expirable value", Duration::from_secs(1));
        map.insert_expirable(3, "expirable value", Duration::from_secs(3));
        map.insert_expirable(4, "expirable value", Duration::from_secs(3));
        map.insert_expirable(5, "expirable value", Duration::from_secs(3));
        map.insert_expirable(6, "expirable value", Duration::from_secs(3));

        std::thread::sleep(Duration::from_secs(2).add(Duration::from_millis(1)));

        assert_eq!(map.len(), 4);
        assert_eq!(map.len_expired(), 2);
        assert_eq!(map.len_unchecked(), 6);
    }

    #[test]
    fn std_update_expirable_entry() {
        let mut map: TimedMap<StdClock, u32, &str> = TimedMap::new();

        map.insert_expirable(1, "expirable value", Duration::from_secs(1));
        map.insert_expirable(1, "expirable value", Duration::from_secs(5));

        std::thread::sleep(Duration::from_secs(2));

        assert!(!map.expiries.contains_key(&1));
        assert!(map.expiries.contains_key(&5));
        assert_eq!(map.get(&1), Some(&"expirable value"));
    }

    #[test]
    fn std_update_expirable_entry_status() {
        let mut map: TimedMap<StdClock, u32, &str> = TimedMap::new();

        map.insert_expirable(1, "expirable value", Duration::from_secs(1));
        map.update_expiration_status(1, Duration::from_secs(5))
            .expect("shouldn't fails");

        std::thread::sleep(Duration::from_secs(3));
        assert!(!map.expiries.contains_key(&1));
        assert!(map.expiries.contains_key(&5));
        assert_eq!(map.get(&1), Some(&"expirable value"));
    }
}
