use super::device::MtpDevice;
use crate::error::Error;
use libmtp_sys as ffi;
use std::{ffi::CStr, ops::Index};

pub enum Parent {
    Root,
    Folder(u32),
}

pub struct File<'a> {
    inner: *mut ffi::LIBMTP_file_t,
    owner: &'a MtpDevice,
}

impl<'a> Drop for File<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::LIBMTP_destroy_file_t(self.inner);
        }
    }
}

impl<'a> File<'a> {
    pub fn id(&self) -> u32 {
        unsafe { (*self.inner).item_id }
    }

    pub fn parent_id(&self) -> u32 {
        unsafe { (*self.inner).parent_id }
    }

    pub fn storage_id(&self) -> u32 {
        unsafe { (*self.inner).storage_id }
    }

    pub fn file_size(&self) -> u64 {
        unsafe { (*self.inner).filesize }
    }

    pub fn file_name(&self) -> &str {
        unsafe {
            let cstr = CStr::from_ptr((*self.inner).filename);
            cstr.to_str().unwrap()
        }
    }

    pub fn rename(&mut self, new_name: &str) -> Result<(), Error> {
        unsafe {
            let res = ffi::LIBMTP_Set_File_Name(
                self.owner.inner,
                self.inner,
                new_name.as_ptr() as *const _,
            );

            if res != 0 {
                Err(self.owner.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }
}

/// Storage descriptor of some MTP device, note that at any time anything can
/// happen with the device and one of these descriptors *may be invalid*.
pub struct Storage<'a> {
    pub(crate) inner: *mut ffi::LIBMTP_devicestorage_t,
    pub(crate) owner: &'a MtpDevice,
}

impl<'a> Storage<'a> {
    /// Formats this storage (if its device supports the operation).
    ///
    /// *WARNING:*  This WILL DELETE ALL DATA from the device, make sure
    /// you've got confirmation from the user before calling this function.
    pub fn format_storage(&self) -> Result<(), Error> {
        unsafe {
            let res = ffi::LIBMTP_Format_Storage(self.owner.inner, self.inner);

            if res != 0 {
                Err(self.owner.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }
}

/// Represents all the storage "pool" of one MTP device.
pub struct StoragePool<'a> {
    pub(crate) pool: Vec<Storage<'a>>,
    pub(crate) owner: &'a MtpDevice,
}

impl<'a> Index<usize> for StoragePool<'a> {
    type Output = Storage<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        self.pool.index(index)
    }
}

impl<'a> StoragePool<'a> {}
