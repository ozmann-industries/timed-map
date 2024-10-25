#[cfg(feature = "std")]
use super::*;

/// Provides the elapsed time since the creation of the implementer in seconds.
///
/// This is designed to make `TimedMap` work across both `std` and `no_std` environments.
///
/// When compiled with `std` feature, this is handled internally with `StdClock`,
/// which utilizes `std::time::Instant` behind the hood.
///
/// When `std` feature is disabled, user must implement the `elapsed_seconds_since_creation` method themselves
/// typically using a custom time source (such as an embedded system's hardware timer).
///
/// # Example usage:
/// ```rs
/// struct CustomClock;
///
/// impl Clock for CustomClock {
///     fn elapsed_seconds_since_creation(&self) -> u64 {
///         // Hardware specific implementation to measure time.
///     }
/// }
///
/// let clock = CustomClock;
/// let current_time = clock.elapsed_seconds_since_creation();
/// ```
pub trait Clock {
    /// Returns the elapsed time since the creation of the implementer in seconds.
    fn elapsed_seconds_since_creation(&self) -> u64;
}

/// A default `Clock` implementation when `std` is enabled.
///
/// When `std` is enabled, this is automatically utilized in `TimedMap`
/// to avoid requiring users to implement the `Clock` trait themselves.
#[cfg(feature = "std")]
pub struct StdClock {
    creation: Instant,
}

#[cfg(feature = "std")]
impl StdClock {
    pub(crate) fn new() -> Self {
        Self {
            creation: Instant::now(),
        }
    }
}

#[cfg(feature = "std")]
impl Clock for StdClock {
    fn elapsed_seconds_since_creation(&self) -> u64 {
        self.creation.elapsed().as_secs()
    }
}
