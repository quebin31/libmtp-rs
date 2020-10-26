//! Module to handle raw devices, this will be your entrypoint to manage connected USB
//! devices.

use libmtp_sys as ffi;
use std::ffi::CStr;
use std::fmt::{self, Debug};
use std::mem::MaybeUninit;

use crate::device::MtpDevice;
use crate::error::{Error, MtpErrorKind};
use crate::internals::{maybe_init, DeviceEntry};
use crate::Result;

/// This struct handles a raw device, which should be opened with `open` or `open_uncached`
/// if you want to manage the proper MTP device.
pub struct RawDevice {
    pub(crate) inner: ffi::LIBMTP_raw_device_struct,
}

impl Debug for RawDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawDevice")
            .field("bus_number", &self.bus_number())
            .field("dev_number", &self.dev_number())
            .field("device_entry", &self.device_entry())
            .finish()
    }
}

impl RawDevice {
    /// Open an MTP device from this raw device descriptor, this method
    /// may cache devices, thus may be slower.
    pub fn open(&self) -> Option<MtpDevice> {
        unsafe {
            let ptr = &self.inner as *const _;
            let device = ffi::LIBMTP_Open_Raw_Device(ptr as *mut _);

            if device.is_null() {
                None
            } else {
                Some(MtpDevice { inner: device })
            }
        }
    }

    /// Open an MTP device from this raw device descriptor, uncached version.
    pub fn open_uncached(&self) -> Option<MtpDevice> {
        unsafe {
            let ptr = &self.inner as *const _;
            let device = ffi::LIBMTP_Open_Raw_Device_Uncached(ptr as *mut _);

            if device.is_null() {
                None
            } else {
                Some(MtpDevice { inner: device })
            }
        }
    }

    /// Returns the bus number of this raw device.
    pub fn bus_number(&self) -> u32 {
        self.inner.bus_location
    }

    /// Returns the device number of this raw device.
    pub fn dev_number(&self) -> u8 {
        self.inner.devnum
    }

    /// Returns the device entry of this raw device.
    pub fn device_entry(&self) -> DeviceEntry {
        let vendor;
        let product;

        unsafe {
            vendor = CStr::from_ptr(self.inner.device_entry.vendor);
            product = CStr::from_ptr(self.inner.device_entry.product);
        }

        DeviceEntry {
            vendor: vendor.to_str().expect("Invalid UTF-8 in music-players.h?"),
            vendor_id: self.inner.device_entry.vendor_id,
            product: product.to_str().expect("Invalid UTF-8 in music-players.h?"),
            product_id: self.inner.device_entry.product_id,
            device_flags: self.inner.device_entry.device_flags,
        }
    }
}

/// Detect the raw device descriptors, you will use this function whenever you want
/// to find which devices are connected, then you may open one or all of these devices,
/// to properly manage the device properties, its storage, files, etc.
///
/// ## Example
/// ```
/// use libmtp_rs::raw::detect_raw_devices;
///
/// let raw_devices = detect_raw_devices().expect("Failed to detect raw devices");
///
/// // Try to open the first device
/// let mtp_device = raw_devices
///                     .get(0)
///                     .map(|r| r.open_uncached())
///                     .transpose()
///                     .expect("Couldn't open raw device");
/// ```
pub fn detect_raw_devices() -> Result<Vec<RawDevice>> {
    maybe_init();

    unsafe {
        let mut devices = std::ptr::null_mut();
        let mut len = 0;

        let res = ffi::LIBMTP_Detect_Raw_Devices(&mut devices, &mut len);

        if let Some(kind) = MtpErrorKind::from_error_number(res) {
            Err(Error::MtpError {
                kind,
                text: "Failed to detect raw devices".to_string(),
            })
        } else {
            let mut devices_vec = Vec::with_capacity(len as usize);
            for i in 0..(len as isize) {
                let mut new = MaybeUninit::zeroed().assume_init();

                std::ptr::copy_nonoverlapping(devices.offset(i), &mut new, 1);
                devices_vec.push(RawDevice { inner: new });
            }

            libc::free(devices as *mut _);
            Ok(devices_vec)
        }
    }
}

/// Check if a specific device, given its bus and device number, has an
/// MTP type device descriptor.
pub fn check_specific_device(bus_number: u32, dev_number: u32) -> bool {
    let res = unsafe { ffi::LIBMTP_Check_Specific_Device(bus_number as i32, dev_number as i32) };
    res == 1
}
