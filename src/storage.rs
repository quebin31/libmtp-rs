//! Module with `Storage` and `StoragePool` that are able to manage the storage of
//! an specific device, and perform certain operations like sending and getting
//! files, tracks, etc.

pub mod files;
pub mod folders;

use derivative::Derivative;
use files::{File, FileMetadata};
use libmtp_sys as ffi;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::{borrow::Cow, collections::HashMap, ffi::CStr};
use std::{
    fmt::{self, Debug},
    path::Path,
};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

use crate::{
    device::MtpDevice, object::AsObjectId, util::CallbackReturn, util::HandlerReturn, Result,
};

use self::folders::{create_folder, get_folder_list, get_folder_list_storage, Folder};

/// Internal function to retrieve files and folders from a single storage or the whole storage pool.
fn files_and_folders<'a>(mtpdev: &'a MtpDevice, storage_id: u32, parent: Parent) -> Vec<File<'a>> {
    let parent_id = parent.faf_id();

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

/// Represents the parent folder of an object, the top-most parent is called the "root" as in
/// *nix like systems.
#[derive(Debug, Copy, Clone)]
pub enum Parent {
    Root,
    Folder(u32),
}

impl Parent {
    pub(crate) fn faf_id(self) -> u32 {
        match self {
            Parent::Root => ffi::LIBMTP_FILES_AND_FOLDERS_ROOT,
            Parent::Folder(id) => id,
        }
    }

    pub(crate) fn to_id(self) -> u32 {
        match self {
            Parent::Root => 0,
            Parent::Folder(id) => id,
        }
    }
}

#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum StorageType {
    Undefined = 0,
    FixedRom,
    RemovableRom,
    FixedRam,
    RemovableRam,
}

#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum FilesystemType {
    Undefined = 0,
    GenericFlat,
    GenericHierarchical,
    DesignCameraFilesystem,
}

#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum AccessCapability {
    ReadWrite = 0,
    ReadOnly,
    ReadOnlyWithObjectDeletion,
}

/// Storage descriptor of some MTP device, note that updating the storage and
/// keeping a old copy of this struct is impossible.
pub struct Storage<'a> {
    pub(crate) inner: *mut ffi::LIBMTP_devicestorage_t,
    pub(crate) owner: &'a MtpDevice,
}

impl Debug for Storage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Storage")
            .field("id", &self.id())
            .field("storage_type", &self.storage_type())
            .field("filesystem_type", &self.filesystem_type())
            .field("access_capability", &self.access_capability())
            .field("maximum_capacity", &self.maximum_capacity())
            .field("free_space_in_bytes", &self.free_space_in_bytes())
            .field("free_space_in_objects", &self.free_space_in_objects())
            .field("volume_identifier", &self.volume_identifier())
            .field("description", &self.description())
            .finish()
    }
}

impl<'a> Storage<'a> {
    /// Retrieves the id of this storage.
    pub fn id(&self) -> u32 {
        unsafe { (*self.inner).id }
    }

    /// Returns the storage type
    pub fn storage_type(&self) -> StorageType {
        let stype = unsafe { (*self.inner).StorageType };
        StorageType::from_u16(stype).unwrap_or_else(|| StorageType::Undefined)
    }

    /// Returns the file system type
    pub fn filesystem_type(&self) -> FilesystemType {
        let ftype = unsafe { (*self.inner).FilesystemType };
        FilesystemType::from_u16(ftype).unwrap_or_else(|| FilesystemType::Undefined)
    }

    /// Returns the access capability
    pub fn access_capability(&self) -> AccessCapability {
        let cap = unsafe { (*self.inner).AccessCapability };
        AccessCapability::from_u16(cap).expect("Unknown access capability")
    }

    /// Returns the maximum capacity
    pub fn maximum_capacity(&self) -> u64 {
        unsafe { (*self.inner).MaxCapacity }
    }

    /// Returns the free space in bytes
    pub fn free_space_in_bytes(&self) -> u64 {
        unsafe { (*self.inner).FreeSpaceInBytes }
    }

    /// Returns the free space in objects
    pub fn free_space_in_objects(&self) -> u64 {
        unsafe { (*self.inner).FreeSpaceInObjects }
    }

    /// Returns the storage description
    pub fn description(&self) -> Option<&str> {
        unsafe {
            if (*self.inner).StorageDescription.is_null() {
                None
            } else {
                let cstr = CStr::from_ptr((*self.inner).StorageDescription);
                Some(cstr.to_str().expect("Invalid UTF-8"))
            }
        }
    }

    /// Returns the volume identifier
    pub fn volume_identifier(&self) -> Option<&str> {
        unsafe {
            if (*self.inner).VolumeIdentifier.is_null() {
                None
            } else {
                let cstr = CStr::from_ptr((*self.inner).VolumeIdentifier);
                Some(cstr.to_str().expect("Invalid UTF-8"))
            }
        }
    }

    /// Formats this storage (if its device supports the operation).
    ///
    /// **WARNING:** This **WILL DELETE ALL DATA** from the device, make sure
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

    /// Optionally returns a `Folder`, with this struct you can build a tree
    /// structure (see `Folder` for more info)
    pub fn folder_list(&self) -> Option<Folder<'a>> {
        unsafe { get_folder_list_storage(self.owner, (*self.inner).id) }
    }

    /// Tries to create a new folder in this storage for the relevant `MtpDevice`, returns the id
    /// of the new folder and its name, note that the name may be different due to device file
    /// system restrictions.
    pub fn create_folder<'b>(&self, name: &'b str, parent: Parent) -> Result<(u32, Cow<'b, str>)> {
        unsafe { create_folder(self.owner, name, parent, (*self.inner).id) }
    }

    /// Retrieves a file from the device storage to a local file identified by a filename.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    pub fn get_file_to_path<C>(
        &self,
        file: impl AsObjectId,
        path: impl AsRef<Path>,
        callback: impl Into<Option<C>>,
    ) -> Result<()>
    where
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        files::get_file_to_path(self.owner, file, path, callback.into())
    }

    /// Retrieves a file from the device storage to a local file identified by a descriptor.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    #[cfg(unix)]
    pub fn get_file_to_descriptor<C>(
        &self,
        file: impl AsObjectId,
        descriptor: impl AsRawFd,
        callback: impl Into<Option<C>>,
    ) -> Result<()>
    where
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        files::get_file_to_descriptor(self.owner, file, descriptor, callback.into())
    }

    /// Retrieves a file from the device storage and calls handler with chunks of data.
    ///
    /// The `handler` parameter is the function that receives the chunks of data with
    /// the following signature `(data: &[u8], read_len: &mut u32) -> HandlerReturn`,
    /// where the `read_len` should be modified with the amount of bytes you actually
    /// read, the `HandlerReturn` allows you to specify if the operation was ok, had an
    /// error or if you want to cancel it.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    pub fn get_file_to_handler<H, C>(
        &self,
        file: impl AsObjectId,
        handler: H,
        callback: impl Into<Option<C>>,
    ) -> Result<()>
    where
        H: FnMut(&[u8], &mut u32) -> HandlerReturn,
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        files::get_file_to_handler(self.owner, file, handler, callback.into())
    }

    /// Sends a local file to the MTP device who this storage belongs to.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    pub fn send_file_from_path<C>(
        &self,
        path: impl AsRef<Path>,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: impl Into<Option<C>>,
    ) -> Result<File<'a>>
    where
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        let storage_id = self.id();
        files::send_file_from_path(
            self.owner,
            storage_id,
            path,
            parent,
            metadata,
            callback.into(),
        )
    }

    /// Sends a local file via descriptor to the MTP device who this storage belongs to.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    #[cfg(unix)]
    pub fn send_file_from_descriptor<C>(
        &self,
        descriptor: impl AsRawFd,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: impl Into<Option<C>>,
    ) -> Result<File<'a>>
    where
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        let storage_id = self.id();
        files::send_file_from_descriptor(
            self.owner,
            storage_id,
            descriptor,
            parent,
            metadata,
            callback.into(),
        )
    }

    /// Sends a bunch of data to the MTP device who this storage belongs to.
    ///
    /// The `handler` parameter is the function that receives the chunks of data with
    /// the following signature `(data: &mut [u8], write_len: &mut u32) -> HandlerReturn`,
    /// where the `write_len` should be modified with the amount of bytes you actually
    /// write, the `HandlerReturn` allows you to specify if the operation was ok, had an
    /// error or if you want to cancel it.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    pub fn send_file_from_handler<H, C>(
        &self,
        handler: H,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: impl Into<Option<C>>,
    ) -> Result<File<'a>>
    where
        H: FnMut(&mut [u8], &mut u32) -> HandlerReturn,
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        let storage_id = self.id();
        files::send_file_from_handler(
            self.owner,
            storage_id,
            handler,
            parent,
            metadata,
            callback.into(),
        )
    }
}

/// Represents all the storage "pool" of one MTP device, contain all the storage entries
/// of one MTP device, and contains some methods to send or get files from the primary storage.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct StoragePool<'a> {
    order: Vec<u32>,
    pool: HashMap<u32, Storage<'a>>,

    #[derivative(Debug = "ignore")]
    owner: &'a MtpDevice,
}

/// Iterator that allows us to get each `Storage` with its id.
pub struct StoragePoolIter<'a> {
    pool: &'a HashMap<u32, Storage<'a>>,
    itr: usize,
    order: &'a [u32],
}

impl<'a> Iterator for StoragePoolIter<'a> {
    type Item = (u32, &'a Storage<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.itr >= self.pool.len() {
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

    /// Optionally returns a `Folder`, with this struct you can build a tree
    /// structure (see `Folder` for more info)
    pub fn folder_list(&self) -> Option<Folder<'_>> {
        get_folder_list(self.owner)
    }

    /// Tries to create a new folder in the default storage of the relevant `MtpDevice`, returns
    /// the id of the new folder and its name, note that the name may be different due to device
    /// file system restrictions.
    pub fn create_folder<'b>(&self, name: &'b str, parent: Parent) -> Result<(u32, Cow<'b, str>)> {
        create_folder(self.owner, name, parent, 0)
    }

    /// Retrieves a file from the device storage to a local file identified by a filename, note
    /// that this is just a convenience method since it's not necessary to depend on the `Storage`,
    /// this is because objects have unique ids across all the device.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    pub fn get_file_to_path<C>(
        &self,
        file: impl AsObjectId,
        path: impl AsRef<Path>,
        callback: impl Into<Option<C>>,
    ) -> Result<()>
    where
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        files::get_file_to_path(self.owner, file, path, callback.into())
    }

    /// Retrieves a file from the device storage to a local file identified by a descriptor, note
    /// that this is just a convenience method since it's not necessary to depend on the `Storage`,
    /// this is because objects have unique ids across all the device.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    #[cfg(unix)]
    pub fn get_file_to_descriptor<C>(
        &self,
        file: impl AsObjectId,
        descriptor: impl AsRawFd,
        callback: impl Into<Option<C>>,
    ) -> Result<()>
    where
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        files::get_file_to_descriptor(self.owner, file, descriptor, callback.into())
    }

    /// Retrieves a file from the device storage and calls handler with chunks of data, note that
    /// this is just a convenience method since it's not necessary to depend on the `Storage`, this
    /// is because objects have unique ids across all the device.
    ///
    /// The `handler` parameter is the function that receives the chunks of data with
    /// the following signature `(data: &[u8], read_len: &mut u32) -> HandlerReturn`,
    /// where the `read_len` should be modified with the amount of bytes you actually
    /// read, the `HandlerReturn` allows you to specify if the operation was ok, had an
    /// error or if you want to cancel it.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    pub fn get_file_to_handler<H, C>(
        &self,
        file: impl AsObjectId,
        handler: H,
        callback: impl Into<Option<C>>,
    ) -> Result<()>
    where
        H: FnMut(&[u8], &mut u32) -> HandlerReturn,
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        files::get_file_to_handler(self.owner, file, handler, callback.into())
    }

    /// Sends a local file to the MTP device who this storage belongs to, note that this method
    /// will send the file to the primary storage.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    pub fn send_file_from_path<C>(
        &self,
        path: impl AsRef<Path>,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: impl Into<Option<C>>,
    ) -> Result<File<'a>>
    where
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        let storage_id = 0;
        files::send_file_from_path(
            self.owner,
            storage_id,
            path,
            parent,
            metadata,
            callback.into(),
        )
    }

    /// Sends a local file via descriptor to the MTP device who this storage belongs to, note
    /// that this method will send the file to the primary storage.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    #[cfg(unix)]
    pub fn send_file_from_descriptor<C>(
        &self,
        descriptor: impl AsRawFd,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: impl Into<Option<C>>,
    ) -> Result<File<'a>>
    where
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        let storage_id = 0;
        files::send_file_from_descriptor(
            self.owner,
            storage_id,
            descriptor,
            parent,
            metadata,
            callback.into(),
        )
    }

    /// Sends a bunch of data to the MTP device who this storage belongs to, note that this
    /// method will send the file to primary storage.
    ///
    /// The `handler` parameter is the function that receives the chunks of data with
    /// the following signature `(data: &mut [u8], write_len: &mut u32) -> HandlerReturn`,
    /// where the `write_len` should be modified with the amount of bytes you actually
    /// write, the `HandlerReturn` allows you to specify if the operation was ok, had an
    /// error or if you want to cancel it.
    ///
    /// The `callback` parameter is an optional progress function with the following
    /// signature `(sent_bytes: u64, total_bytes: u64) -> CallbackReturn`, this way you
    /// can check the progress and if you want to cancel operation you just return
    /// `CallbackReturn::Cancel`.
    pub fn send_file_from_handler<H, C>(
        &self,
        handler: H,
        parent: Parent,
        metadata: FileMetadata<'_>,
        callback: impl Into<Option<C>>,
    ) -> Result<File<'a>>
    where
        H: FnMut(&mut [u8], &mut u32) -> HandlerReturn,
        C: FnMut(u64, u64) -> CallbackReturn,
    {
        let storage_id = 0;
        files::send_file_from_handler(
            self.owner,
            storage_id,
            handler,
            parent,
            metadata,
            callback.into(),
        )
    }
}
