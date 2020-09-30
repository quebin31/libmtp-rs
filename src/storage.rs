pub mod files;

use files::{File, FileMetadata};
use libmtp_sys as ffi;
use std::collections::HashMap;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

use crate::{device::MtpDevice, object::AsObjectId, util::HandlerReturn, Result};

/// Internal function to retrieve files and folders from a single storage or the whole storage pool.
fn files_and_folders<'a>(mtpdev: &'a MtpDevice, storage_id: u32, parent: Parent) -> Vec<File<'a>> {
    let parent_id = parent.to_id();

    let mut head =
        unsafe { ffi::LIBMTP_Get_Files_And_Folders(mtpdev.inner, storage_id, parent_id) };

    let mut files = Vec::new();
    while !head.is_null() {
        files.push(File {
            inner: head,
            owner: mtpdev,
        });

        head = unsafe { (*head).next };
    }

    files
}

/// Represents the parent of an object, the top-most parent is called the "root".
#[derive(Debug, Copy, Clone)]
pub enum Parent {
    Root,
    Folder(u32),
}

impl Parent {
    pub(crate) fn to_id(self) -> u32 {
        match self {
            Parent::Root => ffi::LIBMTP_FILES_AND_FOLDERS_ROOT,
            Parent::Folder(id) => id,
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
    /// Retrieves the id of this storage.
    pub fn id(&self) -> u32 {
        unsafe { (*self.inner).id }
    }

    /// Formats this storage (if its device supports the operation).
    ///
    /// *WARNING:*  This WILL DELETE ALL DATA from the device, make sure
    /// you've got confirmation from the user before calling this function.
    pub fn format_storage(&self) -> Result<()> {
        let res = unsafe { ffi::LIBMTP_Format_Storage(self.owner.inner, self.inner) };

        if res != 0 {
            Err(self.owner.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }

    /// Retrieves the contents of a certain folder (`parent`) in this storage, the result contains
    /// both files and folders, note that this request will always perform I/O with the device.
    pub fn files_and_folders(&self, parent: Parent) -> Vec<File<'a>> {
        let storage_id = unsafe { (*self.inner).id };
        files_and_folders(self.owner, storage_id, parent)
    }

    /// Retrieves a file from the device storage to a local file identified by a filename.
    /// *WARNING:* Although this function is supported on Windows, it hasn't been tested yet.
    pub fn get_file_to_path<C>(
        &self,
        file: impl AsObjectId,
        path: impl AsRef<Path>,
        callback: Option<C>,
    ) -> Result<()>
    where
        C: FnMut(u64, u64) -> bool,
    {
        files::get_file_to_path(self.owner, file, path, callback)
    }

    /// Retrieves a file from the device storage to a local file identified by a descriptor.
    #[cfg(unix)]
    pub fn get_file_to_descriptor<C>(
        &self,
        file: impl AsObjectId,
        descriptor: impl AsRawFd,
        callback: Option<C>,
    ) -> Result<()>
    where
        C: FnMut(u64, u64) -> bool,
    {
        files::get_file_to_descriptor(self.owner, file, descriptor, callback)
    }

    /// Retrieves a file from the device storage and calls put_func with chunks of data.
    pub fn get_file_to_handler<H, C>(
        &self,
        file: impl AsObjectId,
        handler: H,
        callback: Option<C>,
    ) -> Result<()>
    where
        H: FnMut(&[u8], &mut u32) -> HandlerReturn,
        C: FnMut(u64, u64) -> bool,
    {
        files::get_file_to_handler(self.owner, file, handler, callback)
    }

    /// Sends a local file to the MTP device who this storage belongs to.
    pub fn send_file_from_path<C>(
        &self,
        path: impl AsRef<Path>,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: Option<C>,
    ) -> Result<File<'a>>
    where
        C: FnMut(u64, u64) -> bool,
    {
        let storage_id = self.id();
        files::send_file_from_path(self.owner, storage_id, path, parent, metadata, callback)
    }

    #[cfg(unix)]
    pub fn send_file_from_descriptor<C>(
        &self,
        descriptor: impl AsRawFd,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: Option<C>,
    ) -> Result<File<'a>>
    where
        C: FnMut(u64, u64) -> bool,
    {
        let storage_id = self.id();
        files::send_file_from_descriptor(
            self.owner, storage_id, descriptor, parent, metadata, callback,
        )
    }

    pub fn send_file_from_handler<H, C>(
        &self,
        handler: H,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: Option<C>,
    ) -> Result<File<'a>>
    where
        H: FnMut(&mut [u8], &mut u32) -> HandlerReturn,
        C: FnMut(u64, u64) -> bool,
    {
        let storage_id = self.id();
        files::send_file_from_handler(self.owner, storage_id, handler, parent, metadata, callback)
    }
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
        files_and_folders(self.owner, 0, parent)
    }

    /// Sends a local file to the MTP device who this storage belongs to.
    pub fn send_file_from_path<C>(
        &self,
        path: impl AsRef<Path>,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: Option<C>,
    ) -> Result<File<'a>>
    where
        C: FnMut(u64, u64) -> bool,
    {
        let storage_id = 0;
        files::send_file_from_path(self.owner, storage_id, path, parent, metadata, callback)
    }

    #[cfg(unix)]
    pub fn send_file_from_descriptor<C>(
        &self,
        descriptor: impl AsRawFd,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: Option<C>,
    ) -> Result<File<'a>>
    where
        C: FnMut(u64, u64) -> bool,
    {
        let storage_id = 0;
        files::send_file_from_descriptor(
            self.owner, storage_id, descriptor, parent, metadata, callback,
        )
    }

    pub fn send_file_from_handler<H, C>(
        &self,
        handler: H,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: Option<C>,
    ) -> Result<File<'a>>
    where
        H: FnMut(&mut [u8], &mut u32) -> HandlerReturn,
        C: FnMut(u64, u64) -> bool,
    {
        let storage_id = 0;
        files::send_file_from_handler(self.owner, storage_id, handler, parent, metadata, callback)
    }
}
