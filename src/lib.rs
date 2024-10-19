#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod entry;
mod map;

pub use map::TimedMap;

pub trait Clock {
    #[cfg(feature = "std")]
    fn now_seconds(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    #[cfg(not(feature = "std"))]
    fn now_seconds(&self) -> u64;
}
