use super::*;

#[derive(Clone, Copy)]
pub enum EntryStatus {
    Constant,
    ExpiresAtSeconds(u64),
}

/// Enum representing the status of an entry in the map.
///
/// - `Constant`: Entry is not expirable and remains accessible until removed.
/// - `ExpiresAtSeconds`: Entry will expire once reached to the given time.
impl EntryStatus {
    /// Creates an instance based on the given duration.
    ///
    /// If `duration` is `Some`, the entry will be marked as expirable; otherwise,
    /// it will be constant.
    fn from_duration<C: Clock>(clock: &C, duration: Option<Duration>) -> Self {
        match duration {
            Some(duration) => Self::ExpiresAtSeconds(
                clock
                    .now_seconds()
                    .checked_add(duration.as_secs())
                    .unwrap_or(u64::MAX),
            ),
            None => Self::Constant,
        }
    }
}

/// The entry holds a value `V` and an associated `EntryStatus` which determines
/// whether the entry is constant or expirable.
pub(crate) struct ExpirableEntry<V> {
    value: V,
    status: EntryStatus,
}

impl<V> ExpirableEntry<V> {
    /// Creates a new instance.
    ///
    /// If `duration` is `None`, entry will be constant/unexpirable.
    pub(crate) fn new<C: Clock>(clock: &C, v: V, duration: Option<Duration>) -> Self {
        Self {
            value: v,
            status: EntryStatus::from_duration(clock, duration),
        }
    }

    pub(crate) fn status(&self) -> &EntryStatus {
        &self.status
    }

    pub(crate) fn value(&self) -> &V {
        &self.value
    }

    /// Returns owned `V` and consumes `self`.
    pub(crate) fn owned_value(self) -> V {
        self.value
    }

    /// Checks if the entry has expired based on the current time.
    pub(crate) fn is_expired<C: Clock>(&self, clock: &C) -> bool {
        match self.status {
            EntryStatus::Constant => false,
            EntryStatus::ExpiresAtSeconds(expires_at_seconds) => {
                clock.now_seconds() > expires_at_seconds
            }
        }
    }

    /// Returns the remaining `Duration` before entry expires if it's expirable,
    /// or `None` if it's constant.
    pub(crate) fn remaining_duration<C: Clock>(&self, clock: &C) -> Option<Duration> {
        match self.status {
            EntryStatus::Constant => None,
            EntryStatus::ExpiresAtSeconds(expires_at_seconds) => Some(Duration::from_secs(
                expires_at_seconds.saturating_sub(clock.now_seconds()),
            )),
        }
    }
}

#[cfg(test)]
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
    fn test_entry_status() {
        let clock = MockClock { current_time: 1000 };

        let entry_status = EntryStatus::from_duration(&clock, None);
        assert!(matches!(entry_status, EntryStatus::Constant));

        let duration = Duration::from_secs(60);
        let entry_status = EntryStatus::from_duration(&clock, Some(duration));
        assert!(matches!(entry_status, EntryStatus::ExpiresAtSeconds(1060)));
    }

    #[test]
    fn test_constant_entry() {
        let clock = MockClock { current_time: 1000 };
        let entry = ExpirableEntry::new(&clock, "constant value", None);

        assert_eq!(entry.value(), &"constant value");
        assert!(!entry.is_expired(&clock));
        assert!(matches!(entry.status(), EntryStatus::Constant));
    }

    #[test]
    fn test_expirable_entry() {
        let clock = MockClock { current_time: 1000 };
        let duration = Duration::from_secs(60);
        let entry = ExpirableEntry::new(&clock, "expirable value", Some(duration));

        assert_eq!(entry.value(), &"expirable value");
        assert!(!entry.is_expired(&clock));
        assert!(matches!(
            entry.status(),
            EntryStatus::ExpiresAtSeconds(1060)
        ));
    }

    #[test]
    fn test_expirable_entry_is_expired() {
        let clock = MockClock { current_time: 1000 };
        let duration = Duration::from_secs(60);
        let entry = ExpirableEntry::new(&clock, "expirable value", Some(duration));

        // Entry should not be expired yet
        assert!(!entry.is_expired(&clock));

        // Simulate time passing
        let clock = MockClock { current_time: 1070 };
        assert!(entry.is_expired(&clock));
    }

    #[test]
    fn test_remaining_duration_for_expires_at_seconds() {
        let clock = MockClock { current_time: 1000 };
        let duration = Duration::from_secs(60);
        let entry = ExpirableEntry::new(&clock, "expirable value", Some(duration));

        assert!(!entry.is_expired(&clock));
        assert_eq!(
            entry.remaining_duration(&clock),
            Some(Duration::from_secs(60))
        );

        // Simulate time passing
        let clock = MockClock { current_time: 1050 };
        assert!(!entry.is_expired(&clock));
        assert_eq!(
            entry.remaining_duration(&clock),
            Some(Duration::from_secs(10))
        );

        // Time passed beyond expiration
        let clock = MockClock { current_time: 1070 };
        assert!(entry.is_expired(&clock));
        assert_eq!(
            entry.remaining_duration(&clock),
            Some(Duration::from_secs(0))
        );
    }

    #[test]
    fn test_remaining_duration_for_constant() {
        let clock = MockClock { current_time: 1000 };
        let entry = ExpirableEntry::new(&clock, "constant value", None);

        assert_eq!(entry.remaining_duration(&clock), None);
    }
}
