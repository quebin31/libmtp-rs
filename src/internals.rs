use crate::error::Error;
use bitflags::bitflags;
use libmtp_sys as ffi;
use std::ffi::CStr;

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

pub fn get_supported_devices_list() -> Result<Vec<DeviceEntry>, Error> {
    maybe_init();

    unsafe {
        let mut devices = std::ptr::null_mut();
        let mut len = 0;

        let res = ffi::LIBMTP_Get_Supported_Devices_List(&mut devices, &mut len);

        if res != 0 {
            return Err(Error::Unknown);
        }

        let devices_vec = (0..len as isize)
            .map(|i| {
                let device = &*devices.offset(i);
                let vendor = CStr::from_ptr(device.vendor);
                let product = CStr::from_ptr(device.product);

                DeviceEntry {
                    vendor: vendor.to_str().expect("Invalid UTF-8"),
                    vendor_id: device.vendor_id,
                    product: product.to_str().expect("Invalid UTF-8"),
                    product_id: device.product_id,
                    device_flags: device.device_flags,
                }
            })
            .collect();

        Ok(devices_vec)
    }
}
