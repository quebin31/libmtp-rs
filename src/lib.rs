use error::Error;

#[macro_use]
mod macros;

pub mod error;
pub mod internals;
pub mod util;

pub mod values;

pub mod device;
pub mod object;

/// Re-export for support convenience.
pub use chrono;

/// Custom Result, used (almost) across all the crate.
pub type Result<T> = std::result::Result<T, Error>;

pub trait Identifiable {
    type Id;

    fn id(&self) -> Self::Id;
}

impl Identifiable for u32 {
    type Id = u32;

    fn id(&self) -> Self::Id {
        *self
    }
}
