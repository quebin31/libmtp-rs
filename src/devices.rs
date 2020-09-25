use crate::error::Error;
use crate::internals::maybe_init;
use libmtp_sys as ffi;
use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct MTPDevice {
    inner: *mut ffi::LIBMTP_mtpdevice_t,
}

impl Drop for MTPDevice {
    fn drop(&mut self) {
        unsafe {
            ffi::LIBMTP_Release_Device(self.inner);
        }
    }
}

impl MTPDevice {
    pub fn get_friendly_name(&self) -> Result<String, Error> {
        unsafe {
            let friendly_name = ffi::LIBMTP_Get_Friendlyname(self.inner);
            if friendly_name.is_null() {
                Err(Error::Unknown)
            } else {
                let vec = c_charp_to_u8v!(friendly_name);
                libc::free(friendly_name as *mut _);
                Ok(String::from_utf8(vec)?)
            }
        }
    }

    pub fn set_friendly_name(&self, name: &str) -> Result<(), Error> {
        unsafe {
            let res =
                ffi::LIBMTP_Set_Friendlyname(self.inner, name.as_ptr() as *const libc::c_char);
            if let Some(err) = Error::from_code(res as u32) {
                Err(err)
            } else {
                Ok(())
            }
        }
    }

    pub fn get_sync_partner(&self) -> Result<String, Error> {
        unsafe {
            let partner = ffi::LIBMTP_Get_Syncpartner(self.inner);
            let vec = c_charp_to_u8v!(partner);
            libc::free(partner as *mut _);
            Ok(String::from_utf8(vec)?)
        }
    }

    pub fn set_sync_partner(&self, partner: &str) -> Result<(), Error> {
        unsafe {
            let res =
                ffi::LIBMTP_Set_Syncpartner(self.inner, partner.as_ptr() as *const libc::c_char);

            if let Some(err) = Error::from_code(res as u32) {
                Err(err)
            } else {
                Ok(())
            }
        }
    }

    pub fn manufacturer_name(&self) -> Result<String, Error> {
        unsafe {
            let manufacturer = ffi::LIBMTP_Get_Manufacturername(self.inner);
            if manufacturer.is_null() {
                Err(Error::Unknown)
            } else {
                let vec = c_charp_to_u8v!(manufacturer);
                libc::free(manufacturer as *mut _);
                Ok(String::from_utf8(vec)?)
            }
        }
    }

    pub fn model_name(&self) -> Result<String, Error> {
        unsafe {
            let model = ffi::LIBMTP_Get_Modelname(self.inner);
            if model.is_null() {
                Err(Error::Unknown)
            } else {
                let vec = c_charp_to_u8v!(model);
                libc::free(model as *mut _);
                Ok(String::from_utf8(vec)?)
            }
        }
    }

    pub fn serial_number(&self) -> Result<String, Error> {
        unsafe {
            let serial = ffi::LIBMTP_Get_Serialnumber(self.inner);
            if serial.is_null() {
                Err(Error::Unknown)
            } else {
                let vec = c_charp_to_u8v!(serial);
                libc::free(serial as *mut _);
                Ok(String::from_utf8(vec)?)
            }
        }
    }

    pub fn device_certificate(&self) -> Result<String, Error> {
        unsafe {
            let mut devcert = std::ptr::null_mut();
            let res = ffi::LIBMTP_Get_Device_Certificate(self.inner, &mut devcert);

            if let Some(err) = Error::from_code(res as u32) {
                return Err(err);
            }

            if devcert.is_null() {
                return Err(Error::Unknown);
            }

            let vec = c_charp_to_u8v!(devcert);
            libc::free(devcert as *mut _);
            Ok(String::from_utf8(vec)?)
        }
    }

    pub fn dump_device_info(&self) {
        unsafe {
            ffi::LIBMTP_Dump_Device_Info(self.inner);
        }
    }

    pub fn dump_error_stack(&self) {
        unsafe {
            ffi::LIBMTP_Dump_Errorstack(self.inner);
        }
    }

    pub fn clear_error_stack(&self) {
        unsafe {
            ffi::LIBMTP_Clear_Errorstack(self.inner);
        }
    }
}

pub struct RawDevice {
    inner: ffi::LIBMTP_raw_device_struct,
}

impl RawDevice {
    pub fn open(&self) -> Option<MTPDevice> {
        unsafe {
            let device = ffi::LIBMTP_Open_Raw_Device(&self.inner);

            if device.is_null() {
                None
            } else {
                Some(MTPDevice { inner: device })
            }
        }
    }

    pub fn open_uncached(&self) -> Option<MTPDevice> {
        unsafe {
            let device = ffi::LIBMTP_Open_Raw_Device_Uncached(&self.inner);

            if device.is_null() {
                None
            } else {
                Some(MTPDevice { inner: device })
            }
        }
    }
}

pub fn detect_raw_devices() -> Result<Vec<RawDevice>, Error> {
    maybe_init();

    unsafe {
        let mut devices = std::ptr::null_mut();
        let mut len = 0;

        let res = ffi::LIBMTP_Detect_Raw_Devices(&mut devices, &mut len);
        if let Some(err) = Error::from_code(res) {
            return Err(err);
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp() {
        let devices = detect_raw_devices().unwrap();
        let mtp_device = devices[0].open_uncached().unwrap();
        println!("{:#?}", mtp_device.manufacturer_name());
        println!("{:#?}", mtp_device.model_name());
        println!("{:#?}", mtp_device.device_certificate());
    }
}
