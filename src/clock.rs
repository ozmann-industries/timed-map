#[cfg(feature = "std")]
use super::*;

/// Provides elapsed time since the creation of the implementer, in seconds.
///
/// This is designed to enable `TimedMap` to work in both `std` and `no_std` environments.
///
/// When the `std` feature is enabled, the implementer is expected to use `StdClock`,
/// which relies on `std::time::Instant` for timekeeping.
///
/// In `no_std` environments, users should implement the `elapsed_seconds_since_creation` manually,
/// typically using a custom time source such as embedded system's hardware timer.
///
/// # Example usage:
/// ```rs
/// struct CustomClock;
///
/// impl Clock for CustomClock {
///     fn elapsed_seconds_since_creation(&self) -> u64 {
///     // Hardware-specific implementation to measure the elapsed time.
///         0 // placeholder
///     }
/// }
///
/// let clock = CustomClock;
/// let current_time = clock.elapsed_seconds_since_creation();
/// ```
pub trait Clock {
    /// Returns the elapsed time since the creation of the implementer, in seconds.
    fn elapsed_seconds_since_creation(&self) -> u64;
}

/// A default `Clock` implementation when `std` is enabled.
///
/// When `std` is enabled, this is automatically utilized in `TimedMap`
/// to avoid requiring users to implement the `Clock` trait themselves.
#[cfg(feature = "std")]
#[derive(Clone, Debug)]
pub struct StdClock {
    creation: Instant,
}

#[cfg(feature = "std")]
impl Default for StdClock {
    fn default() -> Self {
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
