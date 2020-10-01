//! This crate aims to provide a flexible and high-level interface to the `libmtp` library, at
//! its current state is alpha software and may not be used in production since some features
//! are still missing, said this contributions are welcome.
//!
//! The usual way to start using this library is with the
//! [`detect_raw_devices`](device/raw/index.html) function which returns a list of
//! [`RawDevice`](device/raw/struct.RawDevice.html)s, i.e. the connected USB devices, using these
//! devices you can open an [`MtpDevice`](device/struct.MtpDevice.html), with this you can gather
//! device properties like manufacturer, model, battery level, etc; and manage objects like files,
//! tracks, albums, etc with [`Storage`](storage/struct.Storage.html) and [`StoragePool`](storage/struct.StoragePool.html).
//!
//! Here we list the more important modules:
//! - [`device`](device/index.html): Gather/set properties and obtain storage.
//! - [`storage`](storage/index.html): Send/get objects (files, tracks, etc) and manage storage.
//! - [`object`](object/index.html): Copying, moving and deleting objects.
//!
//! Aditionally if you want a more low-level control on the attributes of certain objects you may
//! want to check the methods to get and set properties in the [`Object`](object/trait.Object.html)
//! trait to see how to use it with instances of its [implementors](trait.Object.html#implementors).

use error::Error;

#[macro_use]
mod macros;

pub mod error;
pub mod internals;

pub mod util;

pub mod values;

pub mod device;
pub mod object;
pub mod storage;

/// Re-export for support convenience.
pub use chrono;

/// Custom Result type, this is the most used Result in this crate.
pub type Result<T> = std::result::Result<T, Error>;
