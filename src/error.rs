#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "std")]
use std::fmt;

#[derive(Debug)]
pub enum TimedMapError {
    EntryNotFound,
}

impl fmt::Display for TimedMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimedMapError::EntryNotFound => write!(f, "Entry not found for given key"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TimedMapError {}
