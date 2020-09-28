use error::Error;

#[macro_use]
mod macros;

pub mod error;
pub mod internals;
pub mod util;

pub mod values;

pub mod device;
pub mod object;

pub type Result<T> = std::result::Result<T, Error>;
