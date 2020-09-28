/// Group of things related to device capabilities.
pub mod capabilities;

/// Group of things related to the storage of a device.
pub mod storage;

/// Group of things related to the management of raw devices.
pub mod raw;

use capabilities::DeviceCapability;
use libmtp_sys as ffi;
use num_derive::ToPrimitive;
use num_traits::{FromPrimitive, ToPrimitive};
use std::fmt::{self, Debug};
use storage::StoragePool;

use crate::{
    error::Error, object::filetypes::Filetype, object::properties::Property, values::AllowedValues,
};

/// Sorting types when updating the inner storage list of an MTP device.
#[derive(Debug, Clone, Copy, ToPrimitive)]
pub enum StorageSort {
    NotSorted = 0,
    ByFreeSpace,
    ByMaxSpace,
}

/// Result given when updating the inner storage list of an MTP device.
#[derive(Debug, Clone, Copy)]
pub enum UpdateResult {
    /// No errors, everything went fine.
    Success,
    /// Partial success, couldn't get storage properties.
    OnlyIds,
}

/// Information about the battery level.
#[derive(Debug, Copy, Clone)]
pub enum BatteryLevel {
    /// The device is currently on battery.
    OnBattery(u8),
    /// The device is currently on external power.
    OnExternalPower,
}

/// Check if a specific device, given its bus and device number, has an
/// MTP type device descriptor.
pub fn check_specific_device(busno: u32, devno: u32) -> bool {
    unsafe {
        let res = ffi::LIBMTP_Check_Specific_Device(busno as i32, devno as i32);
        res == 1
    }
}

/// Result from opening a raw device descriptor, holds information
/// about the device, storage, etc.
pub struct MtpDevice {
    pub(crate) inner: *mut ffi::LIBMTP_mtpdevice_t,
}

impl Drop for MtpDevice {
    fn drop(&mut self) {
        unsafe {
            ffi::LIBMTP_Release_Device(self.inner);
        }
    }
}

impl Debug for MtpDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MTPDevice")
            .field("default_music_folder", &self.default_music_folder())
            .finish()
    }
}

impl MtpDevice {
    pub(crate) fn latest_error(&self) -> Option<Error> {
        unsafe {
            let list = ffi::LIBMTP_Get_Errorstack(self.inner);
            let err = Error::from_latest_error(list)?;
            ffi::LIBMTP_Clear_Errorstack(self.inner);
            Some(err)
        }
    }
}

impl MtpDevice {
    /// Retrieves the default music folder, if there isn't one this value may be garbage.
    /// Therefore it's not recommended to depend on this value.
    pub fn default_music_folder(&self) -> u32 {
        unsafe { (*self.inner).default_music_folder }
    }

    /// Gets the friendly name of this device, e.g. "Kevin's Android"
    pub fn get_friendly_name(&self) -> Result<String, Error> {
        unsafe {
            let friendly_name = ffi::LIBMTP_Get_Friendlyname(self.inner);

            if friendly_name.is_null() {
                Err(self.latest_error().unwrap_or_default())
            } else {
                let u8vec = cstr_to_u8vec!(friendly_name);
                libc::free(friendly_name as *mut _);
                Ok(String::from_utf8(u8vec)?)
            }
        }
    }

    /// Sets the friendly name of this device
    pub fn set_friendly_name(&self, name: &str) -> Result<(), Error> {
        unsafe {
            let res =
                ffi::LIBMTP_Set_Friendlyname(self.inner, name.as_ptr() as *const libc::c_char);

            if res != 0 {
                Err(self.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }

    /// Retrieves the synchronization partner of this device.
    pub fn get_sync_partner(&self) -> Result<String, Error> {
        unsafe {
            let partner = ffi::LIBMTP_Get_Syncpartner(self.inner);
            let u8vec = cstr_to_u8vec!(partner);
            libc::free(partner as *mut _);
            Ok(String::from_utf8(u8vec)?)
        }
    }

    /// Sets the synchronization partner of this device.
    pub fn set_sync_partner(&self, partner: &str) -> Result<(), Error> {
        unsafe {
            let res =
                ffi::LIBMTP_Set_Syncpartner(self.inner, partner.as_ptr() as *const libc::c_char);

            if res != 0 {
                Err(self.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }

    /// Returns the manufacturer name of this device, may fail.
    pub fn manufacturer_name(&self) -> Result<String, Error> {
        unsafe {
            let manufacturer = ffi::LIBMTP_Get_Manufacturername(self.inner);

            if manufacturer.is_null() {
                Err(self.latest_error().unwrap_or_default())
            } else {
                let u8vec = cstr_to_u8vec!(manufacturer);
                libc::free(manufacturer as *mut _);
                Ok(String::from_utf8(u8vec)?)
            }
        }
    }

    /// Returns the model name of this device, may fail.
    pub fn model_name(&self) -> Result<String, Error> {
        unsafe {
            let model = ffi::LIBMTP_Get_Modelname(self.inner);

            if model.is_null() {
                Err(self.latest_error().unwrap_or_default())
            } else {
                let u8vec = cstr_to_u8vec!(model);
                libc::free(model as *mut _);
                Ok(String::from_utf8(u8vec)?)
            }
        }
    }

    /// Returns the serial number of this device, may fail.
    pub fn serial_number(&self) -> Result<String, Error> {
        unsafe {
            let serial = ffi::LIBMTP_Get_Serialnumber(self.inner);

            if serial.is_null() {
                Err(self.latest_error().unwrap_or_default())
            } else {
                let u8vec = cstr_to_u8vec!(serial);
                libc::free(serial as *mut _);
                Ok(String::from_utf8(u8vec)?)
            }
        }
    }

    /// Returns the device (public key) certificate as an XML document string, may fail.
    pub fn device_certificate(&self) -> Result<String, Error> {
        unsafe {
            let mut devcert = std::ptr::null_mut();
            let res = ffi::LIBMTP_Get_Device_Certificate(self.inner, &mut devcert);

            if res != 0 || devcert.is_null() {
                Err(self.latest_error().unwrap_or_default())
            } else {
                let u8vec = cstr_to_u8vec!(devcert);
                libc::free(devcert as *mut _);
                Ok(String::from_utf8(u8vec)?)
            }
        }
    }

    /// Retrieves the current and maximum battery level of this device, may fail.
    pub fn battery_level(&self) -> Result<(BatteryLevel, u8), Error> {
        unsafe {
            let mut max_level = 0;
            let mut cur_level = 0;

            let res = ffi::LIBMTP_Get_Batterylevel(self.inner, &mut max_level, &mut cur_level);

            if res != 0 {
                Err(self.latest_error().unwrap_or_default())
            } else {
                let cur_level = if cur_level == 0 {
                    BatteryLevel::OnExternalPower
                } else {
                    BatteryLevel::OnBattery(cur_level)
                };

                Ok((cur_level, max_level))
            }
        }
    }

    /// Returns the secure time as an XML document string, may fail.
    pub fn secure_time(&self) -> Result<String, Error> {
        unsafe {
            let mut secure_time = std::ptr::null_mut();
            let res = ffi::LIBMTP_Get_Secure_Time(self.inner, &mut secure_time);

            if res != 0 || secure_time.is_null() {
                Err(self.latest_error().unwrap_or_default())
            } else {
                let u8vec = cstr_to_u8vec!(secure_time);
                libc::free(secure_time as *mut _);
                Ok(String::from_utf8(u8vec)?)
            }
        }
    }

    /// Retrieves a list of supported file types that this device claims it supports.  
    /// This list is mitigated to include the file types that `libmtp` (C library) can handle.
    pub fn supported_filetypes(&self) -> Result<Vec<Filetype>, Error> {
        unsafe {
            let mut filetypes = std::ptr::null_mut();
            let mut len = 0;

            let res = ffi::LIBMTP_Get_Supported_Filetypes(self.inner, &mut filetypes, &mut len);

            if res != 0 || filetypes.is_null() {
                Err(self.latest_error().unwrap_or_default())
            } else {
                let mut filetypes_vec = Vec::with_capacity(len as usize);
                for i in 0..(len as isize) {
                    let ftype = Filetype::from_u16(*filetypes.offset(i)).unwrap();
                    filetypes_vec.push(ftype);
                }

                libc::free(filetypes as *mut _);
                Ok(filetypes_vec)
            }
        }
    }

    /// Check if this device has some specific capability.
    pub fn check_capability(&self, capability: DeviceCapability) -> bool {
        unsafe {
            let cap_code = capability.to_u32().unwrap();
            let res = ffi::LIBMTP_Check_Capability(self.inner, cap_code);
            res != 0
        }
    }

    /// Reset the device only is this one supports the `PTP_OC_ResetDevice` operation code
    /// (`0x1010`)
    pub fn reset_device(&self) -> Result<(), Error> {
        unsafe {
            let res = ffi::LIBMTP_Reset_Device(self.inner);

            if res != 0 {
                Err(self.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }

    /// Updates all the internal storage ids and properties of this device, it can also
    /// optionally sort the list. This operation may success, partially success
    /// (only ids were retrieved) or fail.
    pub fn update_storage(&mut self, sort_by: StorageSort) -> Result<UpdateResult, Error> {
        unsafe {
            let res = ffi::LIBMTP_Get_Storage(self.inner, sort_by.to_i32().unwrap());
            match res {
                0 => Ok(UpdateResult::Success),
                1 => Ok(UpdateResult::OnlyIds),
                _ => Err(self.latest_error().unwrap_or_default()),
            }
        }
    }

    /// Returns the inner storage pool, you need to call this if you updated
    /// the storage with `update_storage`. Note that the pool may be empty.
    pub fn storage_pool(&self) -> StoragePool<'_> {
        unsafe {
            let storage = (*self.inner).storage;
            StoragePool::from_raw(&self, storage)
        }
    }

    /// Dumps out a large chunk of textual information provided from the PTP protocol and
    /// additionally some extra MTP specific information where applicable.
    pub fn dump_device_info(&self) {
        unsafe {
            ffi::LIBMTP_Dump_Device_Info(self.inner);
        }
    }

    /// Determines wheter a property is supported for a given file type.
    pub fn is_property_supported(
        &self,
        property: Property,
        filetype: Filetype,
    ) -> Result<bool, Error> {
        let property = property.to_u32().unwrap();
        let filetype = filetype.to_u32().unwrap();

        unsafe {
            let res = ffi::LIBMTP_Is_Property_Supported(self.inner, property, filetype);
            match res {
                0 => Ok(false),
                r if r > 0 => Ok(true),
                _ => Err(self.latest_error().unwrap_or_default()),
            }
        }
    }

    /// Retrieves the allowes values (range or enumeration) for an specific property.
    pub fn allowed_property_values(
        &self,
        property: Property,
        filetype: Filetype,
    ) -> Result<AllowedValues, Error> {
        let property = property.to_u32().unwrap();
        let filetype = filetype.to_u32().unwrap();

        unsafe {
            let allowed_values_ptr = std::ptr::null_mut();

            let res = ffi::LIBMTP_Get_Allowed_Property_Values(
                self.inner,
                property,
                filetype,
                allowed_values_ptr,
            );

            if res != 0 || allowed_values_ptr.is_null() {
                Err(self.latest_error().unwrap_or_default())
            } else {
                let allowed_values =
                    AllowedValues::from_raw(allowed_values_ptr).ok_or_else(|| Error::Unknown)?;
                ffi::LIBMTP_destroy_allowed_values_t(allowed_values_ptr);
                Ok(allowed_values)
            }
        }
    }

    // TODO: Custom operation function (c_variadic nightly feature)
    // pub fn custom_operation(&self, code: u16, params: &[u32]) -> Result<(), ErrorKind>;
}

#[cfg(test)]
mod tests {
    use super::{raw::detect_raw_devices, storage::Parent};

    #[test]
    fn temp() {
        let raw_devices = detect_raw_devices().unwrap();
        let mtp_device = raw_devices[0].open_uncached().unwrap();
        println!("{:?}", mtp_device);

        let storage_pool = mtp_device.storage_pool();
        let (id, storage) = storage_pool.iter().nth(1).unwrap();
        let files = storage.files_and_folders(Parent::Root);
        println!("storage_id: {}", id);
        println!("files: {:#?}", files);

        storage
            .get_file_to_path(
                27,
                "image.jpg",
                Some(|sent, total| {
                    println!("{} / {}", sent, total);
                    true
                }),
            )
            .expect("Failed to transfer");
    }
}
