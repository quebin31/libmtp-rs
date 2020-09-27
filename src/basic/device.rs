use super::storage::Storage;
use crate::{
    capabilities::DeviceCap, error::ErrorKind, error::StackError, filetypes::Filetype,
    properties::Property,
};
use libmtp_sys as ffi;
use num_derive::ToPrimitive;
use num_traits::{FromPrimitive, ToPrimitive};

/// Sorting types when updating the inner storage list of an MTP device.
#[derive(Debug, Clone, Copy, ToPrimitive)]
pub enum StorageSort {
    NotSorted = 0,
    ByFreeSpace,
    ByMaxSpace,
}

/// Result given when updating the inner storage list of an MTP device.
#[derive(Debug, Clone, Copy)]
pub enum UpdateStorage {
    /// No errors, everything went fine.
    Success,
    /// Partial success, couldn't get storage properties.
    OnlyIds,
    /// Something were wrong.
    Failure,
}

/// Check if a specific device, given its bus and device number, has an
/// MTP type device descriptor.
pub fn check_specific_device(busno: u32, devno: u32) -> bool {
    unsafe {
        let res = ffi::LIBMTP_Check_Specific_Device(busno as i32, devno as i32);
        res == 1
    }
}

/// Information about the battery level.
#[derive(Debug, Copy, Clone)]
pub enum BatteryLevel {
    /// The device is currently on battery.
    OnBattery(u8),
    /// The device is currently on external power.
    OnExternalPower,
}

/// Result from opening a raw device descriptor, holds information
/// about the device, storage, etc.
pub struct MTPDevice {
    pub(crate) inner: *mut ffi::LIBMTP_mtpdevice_t,
}

impl Drop for MTPDevice {
    fn drop(&mut self) {
        unsafe {
            ffi::LIBMTP_Release_Device(self.inner);
        }
    }
}

impl MTPDevice {
    /// Gets the friendly name of this device, e.g. "Kevin's super smartphone"
    pub fn get_friendly_name(&self) -> Result<String, ErrorKind> {
        unsafe {
            let friendly_name = ffi::LIBMTP_Get_Friendlyname(self.inner);
            if friendly_name.is_null() {
                Err(ErrorKind::Unknown)
            } else {
                let vec = c_charp_to_u8v!(friendly_name);
                libc::free(friendly_name as *mut _);
                Ok(String::from_utf8(vec)?)
            }
        }
    }

    /// Sets the friendly name of this device
    pub fn set_friendly_name(&self, name: &str) -> Result<(), ErrorKind> {
        unsafe {
            let res =
                ffi::LIBMTP_Set_Friendlyname(self.inner, name.as_ptr() as *const libc::c_char);
            if let Some(err) = ErrorKind::from_code(res as u32) {
                Err(err)
            } else {
                Ok(())
            }
        }
    }

    /// Retrieves the synchronization partner of this device.
    pub fn get_sync_partner(&self) -> Result<String, ErrorKind> {
        unsafe {
            let partner = ffi::LIBMTP_Get_Syncpartner(self.inner);
            let vec = c_charp_to_u8v!(partner);
            libc::free(partner as *mut _);
            Ok(String::from_utf8(vec)?)
        }
    }

    /// Sets the synchronization partner of this device.
    pub fn set_sync_partner(&self, partner: &str) -> Result<(), ErrorKind> {
        unsafe {
            let res =
                ffi::LIBMTP_Set_Syncpartner(self.inner, partner.as_ptr() as *const libc::c_char);

            if let Some(err) = ErrorKind::from_code(res as u32) {
                Err(err)
            } else {
                Ok(())
            }
        }
    }

    /// Returns the manufacturer name of this device, may fail.
    pub fn manufacturer_name(&self) -> Result<String, ErrorKind> {
        unsafe {
            let manufacturer = ffi::LIBMTP_Get_Manufacturername(self.inner);
            if manufacturer.is_null() {
                Err(ErrorKind::Unknown)
            } else {
                let vec = c_charp_to_u8v!(manufacturer);
                libc::free(manufacturer as *mut _);
                Ok(String::from_utf8(vec)?)
            }
        }
    }

    /// Returns the model name of this device, may fail.
    pub fn model_name(&self) -> Result<String, ErrorKind> {
        unsafe {
            let model = ffi::LIBMTP_Get_Modelname(self.inner);
            if model.is_null() {
                Err(ErrorKind::Unknown)
            } else {
                let vec = c_charp_to_u8v!(model);
                libc::free(model as *mut _);
                Ok(String::from_utf8(vec)?)
            }
        }
    }

    /// Returns the serial number of this device, may fail.
    pub fn serial_number(&self) -> Result<String, ErrorKind> {
        unsafe {
            let serial = ffi::LIBMTP_Get_Serialnumber(self.inner);
            if serial.is_null() {
                Err(ErrorKind::Unknown)
            } else {
                let vec = c_charp_to_u8v!(serial);
                libc::free(serial as *mut _);
                Ok(String::from_utf8(vec)?)
            }
        }
    }

    /// Returns the device (public key) certificate as an XML document string, may fail.
    pub fn device_certificate(&self) -> Result<String, ErrorKind> {
        unsafe {
            let mut devcert = std::ptr::null_mut();
            let res = ffi::LIBMTP_Get_Device_Certificate(self.inner, &mut devcert);

            if let Some(err) = ErrorKind::from_code(res as u32) {
                return Err(err);
            }

            if devcert.is_null() {
                return Err(ErrorKind::Unknown);
            }

            let vec = c_charp_to_u8v!(devcert);
            libc::free(devcert as *mut _);
            Ok(String::from_utf8(vec)?)
        }
    }

    /// Retrieves the current and maximum battery level of this device, may fail.
    pub fn battery_level(&self) -> Result<(BatteryLevel, u8), ErrorKind> {
        unsafe {
            let mut max_level = 0;
            let mut cur_level = 0;

            let res = ffi::LIBMTP_Get_Batterylevel(self.inner, &mut max_level, &mut cur_level);
            if let Some(err) = ErrorKind::from_code(res as u32) {
                Err(err)
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
    pub fn secure_time(&self) -> Result<String, ErrorKind> {
        unsafe {
            let mut secure_time = std::ptr::null_mut();
            let res = ffi::LIBMTP_Get_Secure_Time(self.inner, &mut secure_time);

            if let Some(err) = ErrorKind::from_code(res as u32) {
                return Err(err);
            }

            if secure_time.is_null() {
                return Err(ErrorKind::Unknown);
            }

            let vec = c_charp_to_u8v!(secure_time);
            libc::free(secure_time as *mut _);
            Ok(String::from_utf8(vec)?)
        }
    }

    /// Retrieves a list of supported file types that this device claims it supports.  
    /// This list is mitigated to include the file types that `libmtp` (C library) can handle.
    pub fn supported_filetypes(&self) -> Result<Vec<Filetype>, ErrorKind> {
        unsafe {
            let mut filetypes = std::ptr::null_mut();
            let mut len = 0;

            let res = ffi::LIBMTP_Get_Supported_Filetypes(self.inner, &mut filetypes, &mut len);
            if let Some(err) = ErrorKind::from_code(res as u32) {
                return Err(err);
            }

            if filetypes.is_null() {
                return Err(ErrorKind::Unknown);
            }

            let mut filetypes_vec = Vec::with_capacity(len as usize);
            for i in 0..(len as isize) {
                let ftype = Filetype::from_u16(*filetypes.offset(i)).unwrap();
                filetypes_vec.push(ftype);
            }

            libc::free(filetypes as *mut _);
            Ok(filetypes_vec)
        }
    }

    /// Check if this device has some specific capability.
    pub fn check_capability(&self, capability: DeviceCap) -> bool {
        unsafe {
            let cap_code = capability.to_u32().unwrap();
            let res = ffi::LIBMTP_Check_Capability(self.inner, cap_code);
            res != 0
        }
    }

    /// Reset the device only is this one supports the `PTP_OC_ResetDevice` operation code
    /// (`0x1010`)
    pub fn reset_device(&self) -> Result<(), ErrorKind> {
        unsafe {
            let res = ffi::LIBMTP_Reset_Device(self.inner);
            if let Some(err) = ErrorKind::from_code(res as u32) {
                Err(err)
            } else {
                Ok(())
            }
        }
    }

    /// Updates all the internal storage ids and properties of this device, it can also
    /// optionally sort the list. This operation may success, partially success
    /// (only ids were retrieved) or fail.
    pub fn update_storage(&self, sort_by: StorageSort) -> UpdateStorage {
        unsafe {
            let res = ffi::LIBMTP_Get_Storage(self.inner, sort_by.to_i32().unwrap());
            match res {
                0 => UpdateStorage::Success,
                1 => UpdateStorage::OnlyIds,
                _ => UpdateStorage::Failure,
            }
        }
    }

    /// Returns the inner storage entries, you need to call this if you updated
    /// the storage with `update_storage`.
    pub fn storage_entries(&self) -> Vec<Storage> {
        unsafe {
            let mut storage = (*self.inner).storage;

            let mut entries = Vec::new();
            while !storage.is_null() {
                entries.push(Storage {
                    inner: storage,
                    owner: &self,
                });

                storage = (*storage).next;
            }

            entries
        }
    }

    /// Dumps out a large chunk of textual information provided from the PTP protocol and
    /// additionally some extra MTP specific information where applicable.
    pub fn dump_device_info(&self) {
        unsafe {
            ffi::LIBMTP_Dump_Device_Info(self.inner);
        }
    }

    /// Returns a list of the stack errors.
    pub fn error_stack(&self) -> Vec<StackError> {
        unsafe {
            let list = ffi::LIBMTP_Get_Errorstack(self.inner);
            StackError::from_error_list(list)
        }
    }

    /// Dumps the error stack to `stderr`, beware that this doesn't clean the error stack.
    pub fn dump_error_stack(&self) {
        unsafe {
            ffi::LIBMTP_Dump_Errorstack(self.inner);
        }
    }

    /// Clears the error stack.
    pub fn clear_error_stack(&self) {
        unsafe {
            ffi::LIBMTP_Clear_Errorstack(self.inner);
        }
    }

    /// Determines wheter a property is supported for a given file type.
    pub fn is_property_supported(
        &self,
        property: Property,
        filetype: Filetype,
    ) -> Result<bool, ErrorKind> {
        let property = property.to_u32().unwrap();
        let filetype = filetype.to_u32().unwrap();

        unsafe {
            let res = ffi::LIBMTP_Is_Property_Supported(self.inner, property, filetype);
            match res {
                0 => Ok(false),
                r if r > 0 => Ok(true),
                _ => Err(ErrorKind::Unknown),
            }
        }
    }

    pub fn allowed_property_values(&self, property: Property, filetype: Filetype) {
        let property = property.to_u32().unwrap();
        let filetype = filetype.to_u32().unwrap();
    }

    // TODO: Custom operation function (c_variadic)
    // pub fn custom_operation(&self, code: u16, params: &[u32]) -> Result<(), ErrorKind>;
}
