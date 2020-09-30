use bitflags::bitflags;
use libmtp_sys as ffi;
use std::ffi::CStr;

use crate::{error::Error, Result};

pub(crate) fn maybe_init() {
    static mut ALREADY_INIT: bool = false;
    unsafe {
        if !ALREADY_INIT {
            ffi::LIBMTP_Init();
            ALREADY_INIT = true;
        }
    }
}

bitflags! {
    pub struct DebugLevel: i32 {
        const NONE = ffi::LIBMTP_DEBUG_NONE as i32;
        const PTP = ffi::LIBMTP_DEBUG_PTP as i32;
        const PLST = ffi::LIBMTP_DEBUG_PLST as i32;
        const USB = ffi::LIBMTP_DEBUG_USB as i32;
        const DATA = ffi::LIBMTP_DEBUG_DATA as i32;
        const ALL = ffi::LIBMTP_DEBUG_ALL as i32;
    }
}

/// Set the internal debug level of libmtp (C library).
pub fn set_debug(level: DebugLevel) {
    maybe_init();

    unsafe {
        ffi::LIBMTP_Set_Debug(level.bits());
    }
}

#[derive(Debug, Clone)]
pub struct DeviceEntry {
    pub vendor: &'static str,
    pub vendor_id: u16,
    pub product: &'static str,
    pub product_id: u16,
    pub device_flags: u32,
}

/// Get a list of the supported devices.
pub fn get_supported_devices_list() -> Result<Vec<DeviceEntry>> {
    maybe_init();

    let mut devices_ptr = std::ptr::null_mut();
    let mut len = 0;

    let res = unsafe { ffi::LIBMTP_Get_Supported_Devices_List(&mut devices_ptr, &mut len) };

    if res != 0 {
        Err(Error::Unknown)
    } else {
        let mut devices = Vec::new();
        for offset in 0..len as isize {
            unsafe {
                let device = &*devices_ptr.offset(offset);
                let vendor = CStr::from_ptr(device.vendor);
                let product = CStr::from_ptr(device.product);

                devices.push(DeviceEntry {
                    vendor: vendor.to_str().expect("Invalid UTF-8 in music-players.h?"),
                    vendor_id: device.vendor_id,
                    product: product.to_str().expect("Invalid UTF-8 in music-players.h?"),
                    product_id: device.product_id,
                    device_flags: device.device_flags,
                });
            }
        }

        Ok(devices)
    }
}
