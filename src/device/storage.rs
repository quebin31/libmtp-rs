use chrono::{DateTime, TimeZone, Utc};
use libmtp_sys as ffi;
use std::{collections::HashMap, ffi::CStr};
use std::{
    fmt::{self, Debug},
    path::Path,
};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

use crate::{
    util::data_put_func_handler, util::progress_func_handler, util::HandlerReturn,
    util::Identifiable, Result,
};

use super::MtpDevice;

/// Represents the parent of an object, the top-most parent is called the "root".
#[derive(Debug, Copy, Clone)]
pub enum Parent {
    Root,
    Folder(u32),
}

pub struct File<'a> {
    inner: *mut ffi::LIBMTP_file_t,
    owner: &'a MtpDevice,
}

impl Drop for File<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::LIBMTP_destroy_file_t(self.inner);
        }
    }
}

impl Identifiable for File<'_> {
    type Id = u32;

    /// Returns the object id of this file.
    fn id(&self) -> Self::Id {
        unsafe { (*self.inner).item_id }
    }
}

impl Identifiable for &File<'_> {
    type Id = u32;

    /// Returns the object id of this file.
    fn id(&self) -> Self::Id {
        unsafe { (*self.inner).item_id }
    }
}

impl Debug for File<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("id", &self.id())
            .field("parent_id", &self.parent_id())
            .field("storage_id", &self.storage_id())
            .field("file_size", &self.file_size())
            .field("file_name", &self.file_name())
            .field("modification_date", &self.modification_date())
            .finish()
    }
}

impl<'a> File<'a> {
    /// Returns the id of the storage it belongs to.
    pub fn storage_id(&self) -> u32 {
        unsafe { (*self.inner).storage_id }
    }

    /// Returns the id of its parent.
    pub fn parent_id(&self) -> Parent {
        let id = unsafe { (*self.inner).parent_id };
        if id == ffi::LIBMTP_FILES_AND_FOLDERS_ROOT {
            Parent::Root
        } else {
            Parent::Folder(id)
        }
    }

    /// Returns the size of this file.
    pub fn file_size(&self) -> u64 {
        unsafe { (*self.inner).filesize }
    }

    /// Returns the name of this file.
    pub fn file_name(&self) -> &str {
        unsafe {
            let cstr = CStr::from_ptr((*self.inner).filename);
            cstr.to_str().expect("Invalid UTF-8 on file name")
        }
    }

    /// Returns the latest modification date in UTC.
    pub fn modification_date(&self) -> DateTime<Utc> {
        let epoch = unsafe { (*self.inner).modificationdate };
        Utc.timestamp(epoch, 0)
    }

    /// Rename the name of this file.
    pub fn rename(&mut self, new_name: &str) -> Result<()> {
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

impl Identifiable for Storage<'_> {
    type Id = u32;

    /// Returns the id of this device storage
    fn id(&self) -> Self::Id {
        unsafe { (*self.inner).id }
    }
}

impl<'a> Storage<'a> {
    /// Formats this storage (if its device supports the operation).
    ///
    /// *WARNING:*  This WILL DELETE ALL DATA from the device, make sure
    /// you've got confirmation from the user before calling this function.
    pub fn format_storage(&self) -> Result<()> {
        unsafe {
            let res = ffi::LIBMTP_Format_Storage(self.owner.inner, self.inner);

            if res != 0 {
                Err(self.owner.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }

    /// Retrieves the contents of a certain folder (`parent`) in this storage, the result contains
    /// both files and folders, note that this request will always perform I/O with the device.
    pub fn files_and_folders(&self, parent: Parent) -> Vec<File<'a>> {
        let parent = match parent {
            Parent::Root => ffi::LIBMTP_FILES_AND_FOLDERS_ROOT,
            Parent::Folder(id) => id,
        };

        unsafe {
            let mut head =
                ffi::LIBMTP_Get_Files_And_Folders(self.owner.inner, (*self.inner).id, parent);

            let mut files = Vec::new();
            while !head.is_null() {
                files.push(File {
                    inner: head,
                    owner: &self.owner,
                });

                head = (*head).next;
            }

            files
        }
    }

    /// Retrieves a file from the device storage to a local file identified by a filename.
    /// *WARNING:* Although this function is supported on Windows, it hasn't been tested yet.
    pub fn get_file_to_path<F, P, C>(&self, file: F, path: P, callback: Option<C>) -> Result<()>
    where
        F: Identifiable<Id = u32>,
        P: AsRef<Path>,
        C: FnMut(u64, u64) -> bool,
    {
        let path = path.as_ref();
        let mut buf = Vec::new();

        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                use std::iter::once;
                use std::os::windows::ffi::OsStrExt;

                buf.extend(path.as_os_str()
                    .encode_wide()
                    .chain(once(0))
                    .flat_map(|b| {
                        let b = b.to_ne_bytes();
                        once(b[0]).chain(once(b[1]))
                    }));
            } else {
                use std::os::unix::ffi::OsStrExt;

                buf.extend(path.as_os_str().as_bytes());
                buf.push(0);
            }
        }

        let res = if let Some(mut callback) = callback {
            let mut cb: &mut dyn FnMut(u64, u64) -> bool = &mut callback;
            let cb = &mut cb;
            unsafe {
                ffi::LIBMTP_Get_File_To_File(
                    self.owner.inner,
                    file.id(),
                    buf.as_ptr() as *const _,
                    Some(progress_func_handler),
                    cb as *mut _ as *mut libc::c_void as *const _,
                )
            }
        } else {
            unsafe {
                ffi::LIBMTP_Get_File_To_File(
                    self.owner.inner,
                    file.id(),
                    buf.as_ptr() as *const _,
                    None,
                    std::ptr::null(),
                )
            }
        };

        if res != 0 {
            Err(self.owner.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }

    /// Retrieves a file from the device storage to a local file identified by a descriptor.
    #[cfg(unix)]
    pub fn get_file_to_descriptor<F, D, C>(&self, file: F, fd: D, callback: Option<C>) -> Result<()>
    where
        F: Identifiable<Id = u32>,
        D: AsRawFd,
        C: FnMut(u64, u64) -> bool,
    {
        let res = if let Some(mut callback) = callback {
            let mut cb: &mut dyn FnMut(u64, u64) -> bool = &mut callback;
            let cb = &mut cb;
            unsafe {
                ffi::LIBMTP_Get_File_To_File_Descriptor(
                    self.owner.inner,
                    file.id(),
                    fd.as_raw_fd(),
                    Some(progress_func_handler),
                    cb as *mut _ as *mut libc::c_void as *const _,
                )
            }
        } else {
            unsafe {
                ffi::LIBMTP_Get_File_To_File_Descriptor(
                    self.owner.inner,
                    file.id(),
                    fd.as_raw_fd(),
                    None,
                    std::ptr::null(),
                )
            }
        };

        if res != 0 {
            Err(self.owner.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }

    /// Retrieves a file from the device storage and calls put_func with chunks of data.
    pub fn get_file_to_handler<F, P, C>(&self, file: F, pfunc: P, callback: Option<C>) -> Result<()>
    where
        F: Identifiable<Id = u32>,
        P: FnMut(&[u8], &mut u32) -> HandlerReturn,
        C: FnMut(u64, u64) -> bool,
    {
        let mut pfunc = pfunc;
        let mut pf: &mut dyn FnMut(&[u8], &mut u32) -> HandlerReturn = &mut pfunc;
        let pf = &mut pf;

        let res = if let Some(mut callback) = callback {
            let mut cb: &mut dyn FnMut(u64, u64) -> bool = &mut callback;
            let cb = &mut cb;

            unsafe {
                ffi::LIBMTP_Get_File_To_Handler(
                    self.owner.inner,
                    file.id(),
                    Some(data_put_func_handler),
                    pf as *mut _ as *mut libc::c_void,
                    Some(progress_func_handler),
                    cb as *mut _ as *mut libc::c_void,
                )
            }
        } else {
            unsafe {
                ffi::LIBMTP_Get_File_To_Handler(
                    self.owner.inner,
                    file.id(),
                    Some(data_put_func_handler),
                    pf as *mut _ as *mut libc::c_void,
                    None,
                    std::ptr::null(),
                )
            }
        };

        if res != 0 {
            Err(self.owner.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }

    fn send_file_from_path(&self, path: impl AsRef<Path>, parent: Parent) {}
}

/// Represents all the storage "pool" of one MTP device.
pub struct StoragePool<'a> {
    order: Vec<u32>,
    pool: HashMap<u32, Storage<'a>>,
    owner: &'a MtpDevice,
}

pub struct StoragePoolIter<'a> {
    pool: &'a HashMap<u32, Storage<'a>>,
    itr: usize,
    order: &'a [u32],
}

impl<'a> Iterator for StoragePoolIter<'a> {
    type Item = (u32, &'a Storage<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.itr > self.pool.len() {
            None
        } else {
            let next_id = self.order[self.itr];
            let next_val = self.pool.get(&next_id)?;

            self.itr += 1;

            Some((next_id, next_val))
        }
    }
}

impl<'a> StoragePool<'a> {
    /// Build a StoragePool from a raw ptr of devicestorage_t
    pub(crate) fn from_raw(
        owner: &'a MtpDevice,
        mut ptr: *mut ffi::LIBMTP_devicestorage_t,
    ) -> Self {
        unsafe {
            let mut pool = HashMap::new();
            let mut order = Vec::new();
            while !ptr.is_null() {
                let id = (*ptr).id;
                order.push(id);
                pool.insert(id, Storage { inner: ptr, owner });

                ptr = (*ptr).next;
            }

            Self { order, pool, owner }
        }
    }

    /// Returns the storage that has the given id, if there's one.
    pub fn by_id(&self, id: u32) -> Option<&Storage<'a>> {
        self.pool.get(&id)
    }

    /// Returns an iterator over the storages, this is a HashMap iterator.
    pub fn iter(&'a self) -> StoragePoolIter<'a> {
        StoragePoolIter {
            pool: &self.pool,
            itr: 0,
            order: &self.order,
        }
    }

    /// Retrieves the contents of a certain folder (`parent`) in all storages, the result contains
    /// both files and folders, note that this request will always perform I/O with the device.
    pub fn files_and_folders(&self, parent: Parent) -> Vec<File<'a>> {
        let parent = match parent {
            Parent::Root => ffi::LIBMTP_FILES_AND_FOLDERS_ROOT,
            Parent::Folder(id) => id,
        };

        unsafe {
            // Storage id 0 is meant to be the pool
            let mut head = ffi::LIBMTP_Get_Files_And_Folders(self.owner.inner, 0, parent);
            let mut files = Vec::new();
            while !head.is_null() {
                files.push(File {
                    inner: head,
                    owner: &self.owner,
                });

                head = (*head).next;
            }

            files
        }
    }
}
