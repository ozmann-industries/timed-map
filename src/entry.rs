#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::time::Duration;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use core::time::Duration;

use crate::clock::Clock;

#[derive(Clone, Copy)]
pub enum EntryStatus {
    Constant,
    ExpiresAtSeconds(u64),
}

impl EntryStatus {
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

pub(crate) struct ExpirableEntry<V> {
    value: V,
    status: EntryStatus,
}

impl<V> ExpirableEntry<V> {
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

    pub(crate) fn owned_value(self) -> V {
        self.value
    }

    pub(crate) fn is_expired<C: Clock>(&self, clock: &C) -> bool {
        match self.status {
            EntryStatus::Constant => false,
            EntryStatus::ExpiresAtSeconds(expires_at_seconds) => {
                clock.now_seconds() > expires_at_seconds
            }
        }
    }

    pub(crate) fn remaining_duration<C: Clock>(&self, clock: &C) -> Option<Duration> {
        match self.status {
            EntryStatus::Constant => None,
            EntryStatus::ExpiresAtSeconds(expires_at_seconds) => Some(Duration::from_secs(
                expires_at_seconds.saturating_sub(clock.now_seconds()),
            )),
        }
    }
}
