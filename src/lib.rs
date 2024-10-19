#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod clock;
mod entry;
mod map;

#[cfg(not(feature = "std"))]
pub use clock::Clock;

pub use map::TimedMap;
