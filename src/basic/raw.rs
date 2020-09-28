use std::mem::MaybeUninit;

use super::device::MtpDevice;
use crate::{error::Error, error::MtpErrorKind, internals::maybe_init};
use libmtp_sys as ffi;

/// Raw MTP device descriptor, used to manually open an MTP device
pub struct RawDevice {
    pub(crate) inner: ffi::LIBMTP_raw_device_struct,
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
}

/// Detect the raw MTP device descriptors and return a vector of the devices found.
pub fn detect_raw_devices() -> Result<Vec<RawDevice>, Error> {
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
