#[cfg(feature = "std")]
extern crate std;

/// Provides the current time in seconds.
///
/// This is designed to work across both `std` and `no_std` environments.
///
/// When compiled with `std` feature, a default implementation of the `now_seconds`
/// method is provided, which returns the current system time in seconds since `UNIX_EPOCH`
/// using `SystemTime`.
///
/// When `std` feature is disabled, user must implement the `now_seconds` method themselves
/// typically using a custom time source (such as an embedded system's hardware timer).
///
/// # `std` example:
/// ```rs
/// use std::time::SystemTime;
///
/// struct MyClock;
/// impl Clock for MyClock {}
///
/// let clock = MyClock;
/// let current_time = clock.now_seconds();
/// ```
///
/// # `no_std` example:
/// ```rs
/// struct CustomClock;
///
/// impl Clock for CustomClock {
///     fn now_seconds(&self) -> u64 {
///         // Custom implementation to retrieve the current time.
///         0 // return a fixed dummy value for simplicity
///     }
/// }
///
/// let clock = CustomClock;
/// let current_time = clock.now_seconds();
/// ```
pub trait Clock {
    /// Returns the current time in seconds.
    fn now_seconds(&self) -> u64;
}

/// A default `Clock` implementation when `std` is enabled.
///
/// When `std` is enabled, this is automatically utilized in `TimedMap`
/// to avoid requiring users to implement the `Clock` trait themselves.
#[cfg(feature = "std")]
#[derive(Default)]
pub(crate) struct StdClock {}

#[cfg(feature = "std")]
impl Clock for StdClock {
    fn now_seconds(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }
}
