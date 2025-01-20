use super::*;

#[derive(Clone, Copy, Debug)]
pub enum EntryStatus {
    Constant,
    ExpiresAtSeconds(u64),
}

/// Enum representing the status of an entry in the map.
///
/// - `Constant`: Entry is not expirable and remains accessible until removed.
/// - `ExpiresAtSeconds`: Entry will expire once reached to the given time.
impl EntryStatus {
    /// Creates expirable or constant entry based on `expires_at`.
    ///
    /// If `expires_at` is `Some`, entry will be created as expirable; otherwise,
    /// it will be constant.
    #[inline(always)]
    fn new(expires_at: Option<u64>) -> Self {
        match expires_at {
            Some(t) => Self::ExpiresAtSeconds(t),
            None => Self::Constant,
        }
    }
}

/// The entry holds a value `V` and an associated `EntryStatus` which determines
/// whether the entry is constant or expirable.
#[derive(Debug)]
pub(crate) struct ExpirableEntry<V> {
    value: V,
    status: EntryStatus,
}

impl<V> ExpirableEntry<V> {
    /// Creates a new instance.
    ///
    /// If `expires_at` is `None`, entry will be constant/unexpirable.
    #[inline(always)]
    pub(crate) fn new(v: V, expires_at: Option<u64>) -> Self {
        Self {
            value: v,
            status: EntryStatus::new(expires_at),
        }
    }

    #[inline(always)]
    pub(crate) fn status(&self) -> &EntryStatus {
        &self.status
    }

    #[inline(always)]
    pub(crate) fn value(&self) -> &V {
        &self.value
    }

    #[inline(always)]
    pub(crate) fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }

    /// Returns owned `V` and consumes `self`.
    #[inline(always)]
    pub(crate) fn owned_value(self) -> V {
        self.value
    }

    /// Checks if the entry has expired based on the current time.
    #[inline(always)]
    pub(crate) fn is_expired(&self, now_seconds: u64) -> bool {
        match self.status {
            EntryStatus::Constant => false,
            EntryStatus::ExpiresAtSeconds(expires_at_seconds) => now_seconds > expires_at_seconds,
        }
    }

    /// Returns the remaining `Duration` before entry expires if it's expirable,
    /// or `None` if it's constant.
    #[inline(always)]
    pub(crate) fn remaining_duration(&self, now_seconds: u64) -> Option<Duration> {
        match self.status {
            EntryStatus::Constant => None,
            EntryStatus::ExpiresAtSeconds(expires_at_seconds) => Some(Duration::from_secs(
                expires_at_seconds.saturating_sub(now_seconds),
            )),
        }
    }

    // Update status duration time, aka expiration time.
    pub(crate) fn update_status(&mut self, status: EntryStatus) {
        self.status = status;
    }
}

#[cfg(test)]
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
    fn test_entry_status() {
        let clock = MockClock { current_time: 1000 };

        let entry_status = EntryStatus::new(None);
        assert!(matches!(entry_status, EntryStatus::Constant));

        let duration = Duration::from_secs(60);
        let entry_status = EntryStatus::new(Some(
            clock.elapsed_seconds_since_creation() + duration.as_secs(),
        ));
        assert!(matches!(entry_status, EntryStatus::ExpiresAtSeconds(1060)));
    }

    #[test]
    fn test_constant_entry() {
        let clock = MockClock { current_time: 1000 };
        let entry = ExpirableEntry::new("constant value", None);

        assert_eq!(entry.value(), &"constant value");
        assert!(!entry.is_expired(clock.elapsed_seconds_since_creation()));
        assert!(matches!(entry.status(), EntryStatus::Constant));
    }

    #[test]
    fn test_expirable_entry() {
        let clock = MockClock { current_time: 1000 };
        let duration = Duration::from_secs(60);
        let entry = ExpirableEntry::new(
            "expirable value",
            Some(clock.elapsed_seconds_since_creation() + duration.as_secs()),
        );

        assert_eq!(entry.value(), &"expirable value");
        assert!(!entry.is_expired(clock.elapsed_seconds_since_creation()));
        assert!(matches!(
            entry.status(),
            EntryStatus::ExpiresAtSeconds(1060)
        ));
    }

    #[test]
    fn test_expirable_entry_is_expired() {
        let clock = MockClock { current_time: 1000 };
        let duration = Duration::from_secs(60);
        let entry = ExpirableEntry::new(
            "expirable value",
            Some(clock.elapsed_seconds_since_creation() + duration.as_secs()),
        );

        // Entry should not be expired yet
        assert!(!entry.is_expired(clock.elapsed_seconds_since_creation()));

        // Simulate time passing
        let clock = MockClock { current_time: 1070 };
        assert!(entry.is_expired(clock.elapsed_seconds_since_creation()));
    }

    #[test]
    fn test_remaining_duration_for_expires_at_seconds() {
        let clock = MockClock { current_time: 1000 };
        let duration = Duration::from_secs(60);
        let entry = ExpirableEntry::new(
            "expirable value",
            Some(clock.elapsed_seconds_since_creation() + duration.as_secs()),
        );

        assert!(!entry.is_expired(clock.elapsed_seconds_since_creation()));
        assert_eq!(
            entry.remaining_duration(clock.elapsed_seconds_since_creation()),
            Some(Duration::from_secs(60))
        );

        // Simulate time passing
        let clock = MockClock { current_time: 1050 };
        assert!(!entry.is_expired(clock.elapsed_seconds_since_creation()));
        assert_eq!(
            entry.remaining_duration(clock.elapsed_seconds_since_creation()),
            Some(Duration::from_secs(10))
        );

        // Time passed beyond expiration
        let clock = MockClock { current_time: 1070 };
        assert!(entry.is_expired(clock.elapsed_seconds_since_creation()));
        assert_eq!(
            entry.remaining_duration(clock.elapsed_seconds_since_creation()),
            Some(Duration::from_secs(0))
        );
    }

    #[test]
    fn test_remaining_duration_for_constant() {
        let clock = MockClock { current_time: 1000 };
        let entry = ExpirableEntry::new("constant value", None);

        assert_eq!(
            entry.remaining_duration(clock.elapsed_seconds_since_creation()),
            None
        );
    }
}
