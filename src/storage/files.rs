//! Contains relevant items to handle file objects in the device.

use chrono::{DateTime, TimeZone, Utc};
use libmtp_sys as ffi;
use num_traits::FromPrimitive;
use std::ffi::{CStr, CString};
use std::fmt::{self, Debug};
use std::path::Path;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

use crate::device::MtpDevice;
use crate::object::filetypes::Filetype;
use crate::object::{AsObjectId, Object};
use crate::storage::Parent;
use crate::util::data_get_func_handler;
use crate::util::data_put_func_handler;
use crate::util::progress_func_handler;
use crate::util::{CallbackReturn, HandlerReturn};
use crate::Result;

/// Abstraction of a file object, it implements `Object`, you may want to use
/// this struct to create a tree representation of one storage.
pub struct File<'a> {
    pub(crate) inner: *mut ffi::LIBMTP_file_t,
    pub(crate) owner: &'a MtpDevice,
}

impl Drop for File<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::LIBMTP_destroy_file_t(self.inner);
        }
    }
}

impl Object for File<'_> {
    fn id(&self) -> u32 {
        unsafe { (*self.inner).item_id }
    }

    fn device(&self) -> &MtpDevice {
        self.owner
    }
}

impl Object for &File<'_> {
    fn id(&self) -> u32 {
        unsafe { (*self.inner).item_id }
    }

    fn device(&self) -> &MtpDevice {
        self.owner
    }
}

impl Debug for File<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("id", &self.id())
            .field("parent_id", &self.parent_id())
            .field("storage_id", &self.storage_id())
            .field("size", &self.size())
            .field("name", &self.name())
            .field("ftype", &self.ftype())
            .field("modification_date", &self.modification_date())
            .finish()
    }
}

impl File<'_> {
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
    pub fn size(&self) -> u64 {
        unsafe { (*self.inner).filesize }
    }

    /// Returns the name of this file.
    pub fn name(&self) -> &str {
        unsafe {
            let cstr = CStr::from_ptr((*self.inner).filename);
            cstr.to_str().expect("Invalid UTF-8 on file name")
        }
    }

    /// Returns the type of this file.
    pub fn ftype(&self) -> Filetype {
        let ftype = unsafe { (*self.inner).filetype };
        Filetype::from_u32(ftype).expect("Unexpected raw variant of Filetype")
    }

    /// Returns the latest modification date in UTC.
    pub fn modification_date(&self) -> DateTime<Utc> {
        let epoch = unsafe { (*self.inner).modificationdate };
        Utc.timestamp_opt(epoch, 0).unwrap()
    }

    /// Rename this file in-place.
    pub fn rename(&mut self, new_name: &str) -> Result<()> {
        let new_name = CString::new(new_name).expect("Nul byte");

        let res = unsafe {
            ffi::LIBMTP_Set_File_Name(self.owner.inner, self.inner, new_name.as_ptr() as *const _)
        };

        if res != 0 {
            Err(self.owner.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }
}

/// Convenience struct used as a parameter to send local files to an MTP device.
#[derive(Debug, Clone)]
pub struct FileMetadata<'a> {
    pub file_size: u64,
    pub file_name: &'a str,
    pub file_type: Filetype,
    pub modification_date: DateTime<Utc>,
}

pub(crate) fn get_file_to_path(
    mtpdev: &MtpDevice,
    file: impl AsObjectId,
    path: impl AsRef<Path>,
) -> Result<()> {
    let path = path.as_ref();
    let path = path_to_cvec!(path);

    let res = unsafe {
        ffi::LIBMTP_Get_File_To_File(
            mtpdev.inner,
            file.as_id(),
            path.as_ptr() as *const _,
            None,
            std::ptr::null(),
        )
    };

    if res != 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok(())
    }
}

pub(crate) fn get_file_to_path_with_callback<C>(
    mtpdev: &MtpDevice,
    file: impl AsObjectId,
    path: impl AsRef<Path>,
    mut callback: C,
) -> Result<()>
where
    C: FnMut(u64, u64) -> CallbackReturn,
{
    let path = path.as_ref();
    let path = path_to_cvec!(path);

    let mut callback: &mut dyn FnMut(u64, u64) -> CallbackReturn = &mut callback;
    let callback = &mut callback as *mut _ as *mut libc::c_void as *const _;

    let res = unsafe {
        ffi::LIBMTP_Get_File_To_File(
            mtpdev.inner,
            file.as_id(),
            path.as_ptr() as *const _,
            Some(progress_func_handler),
            callback,
        )
    };

    if res != 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
pub(crate) fn get_file_to_descriptor(
    mtpdev: &MtpDevice,
    file: impl AsObjectId,
    descriptor: impl AsRawFd,
) -> Result<()> {
    let res = unsafe {
        ffi::LIBMTP_Get_File_To_File_Descriptor(
            mtpdev.inner,
            file.as_id(),
            descriptor.as_raw_fd(),
            None,
            std::ptr::null(),
        )
    };

    if res != 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
pub(crate) fn get_file_to_descriptor_with_callback<C>(
    mtpdev: &MtpDevice,
    file: impl AsObjectId,
    descriptor: impl AsRawFd,
    mut callback: C,
) -> Result<()>
where
    C: FnMut(u64, u64) -> CallbackReturn,
{
    let mut callback: &mut dyn FnMut(u64, u64) -> CallbackReturn = &mut callback;
    let callback = &mut callback as *mut _ as *mut libc::c_void as *const _;

    let res = unsafe {
        ffi::LIBMTP_Get_File_To_File_Descriptor(
            mtpdev.inner,
            file.as_id(),
            descriptor.as_raw_fd(),
            Some(progress_func_handler),
            callback,
        )
    };

    if res != 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok(())
    }
}

pub(crate) fn get_file_to_handler<H>(
    mtpdev: &MtpDevice,
    file: impl AsObjectId,
    mut handler: H,
) -> Result<()>
where
    H: FnMut(&[u8]) -> HandlerReturn,
{
    let handler: &mut dyn FnMut(&[u8]) -> HandlerReturn = &mut handler;
    let mut handler_return = HandlerReturn::Ok(0);

    let private = &mut (&mut handler_return, handler) as *mut _ as *mut libc::c_void;

    let res = unsafe {
        ffi::LIBMTP_Get_File_To_Handler(
            mtpdev.inner,
            file.as_id(),
            Some(data_put_func_handler),
            private,
            None,
            std::ptr::null(),
        )
    };

    if res != 0 && handler_return.is_error() {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        if handler_return.is_cancel() {
            let _ = mtpdev.latest_error();
        }

        Ok(())
    }
}

pub(crate) fn get_file_to_handler_with_callback<H, C>(
    mtpdev: &MtpDevice,
    file: impl AsObjectId,
    mut handler: H,
    mut callback: C,
) -> Result<()>
where
    H: FnMut(&[u8]) -> HandlerReturn,
    C: FnMut(u64, u64) -> CallbackReturn,
{
    let handler: &mut dyn FnMut(&[u8]) -> HandlerReturn = &mut handler;
    let mut handler_return = HandlerReturn::Ok(0);

    let private = &mut (&mut handler_return, handler) as *mut _ as *mut libc::c_void;

    let mut callback: &mut dyn FnMut(u64, u64) -> CallbackReturn = &mut callback;
    let callback = &mut callback as *mut _ as *mut libc::c_void as *const _;

    let res = unsafe {
        ffi::LIBMTP_Get_File_To_Handler(
            mtpdev.inner,
            file.as_id(),
            Some(data_put_func_handler),
            private,
            Some(progress_func_handler),
            callback,
        )
    };

    if res != 0 && handler_return.is_error() {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        if handler_return.is_cancel() {
            let _ = mtpdev.latest_error();
        }

        Ok(())
    }
}

pub(crate) fn send_file_from_path<'a>(
    mtpdev: &'a MtpDevice,
    storage_id: u32,
    path: impl AsRef<Path>,
    parent: Parent,
    metadata: FileMetadata<'_>,
) -> Result<File<'a>> {
    let path = path.as_ref();
    let path = path_to_cvec!(path);

    let file_t = unsafe { ffi::LIBMTP_new_file_t() };
    unsafe { fill_file_t!(metadata, parent.to_id(), storage_id, file_t) };

    let res = unsafe {
        ffi::LIBMTP_Send_File_From_File(
            mtpdev.inner,
            path.as_ptr() as *const _,
            file_t,
            None,
            std::ptr::null(),
        )
    };

    if res != 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok(File {
            inner: file_t,
            owner: mtpdev,
        })
    }
}

pub(crate) fn send_file_from_path_with_callback<'a, C>(
    mtpdev: &'a MtpDevice,
    storage_id: u32,
    path: impl AsRef<Path>,
    parent: Parent,
    metadata: FileMetadata<'_>,
    mut callback: C,
) -> Result<File<'a>>
where
    C: FnMut(u64, u64) -> CallbackReturn,
{
    let path = path.as_ref();
    let path = path_to_cvec!(path);

    let file_t = unsafe { ffi::LIBMTP_new_file_t() };
    unsafe { fill_file_t!(metadata, parent.to_id(), storage_id, file_t) };

    let mut callback: &mut dyn FnMut(u64, u64) -> CallbackReturn = &mut callback;
    let callback = &mut callback as *mut _ as *mut libc::c_void as *const _;

    let res = unsafe {
        ffi::LIBMTP_Send_File_From_File(
            mtpdev.inner,
            path.as_ptr() as *const _,
            file_t,
            Some(progress_func_handler),
            callback,
        )
    };

    if res != 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok(File {
            inner: file_t,
            owner: mtpdev,
        })
    }
}

#[cfg(unix)]
pub(crate) fn send_file_from_descriptor<'a>(
    mtpdev: &'a MtpDevice,
    storage_id: u32,
    descriptor: impl AsRawFd,
    parent: Parent,
    metadata: FileMetadata<'_>,
) -> Result<File<'a>> {
    let file_t = unsafe { ffi::LIBMTP_new_file_t() };
    unsafe { fill_file_t!(metadata, parent.to_id(), storage_id, file_t) };

    let res = unsafe {
        ffi::LIBMTP_Send_File_From_File_Descriptor(
            mtpdev.inner,
            descriptor.as_raw_fd(),
            file_t,
            None,
            std::ptr::null(),
        )
    };

    if res != 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok(File {
            inner: file_t,
            owner: mtpdev,
        })
    }
}

#[cfg(unix)]
pub(crate) fn send_file_from_descriptor_with_callback<'a, C>(
    mtpdev: &'a MtpDevice,
    storage_id: u32,
    descriptor: impl AsRawFd,
    parent: Parent,
    metadata: FileMetadata<'_>,
    mut callback: C,
) -> Result<File<'a>>
where
    C: FnMut(u64, u64) -> CallbackReturn,
{
    let file_t = unsafe { ffi::LIBMTP_new_file_t() };
    unsafe { fill_file_t!(metadata, parent.to_id(), storage_id, file_t) };

    let mut callback: &mut dyn FnMut(u64, u64) -> CallbackReturn = &mut callback;
    let callback = &mut callback as *mut _ as *mut libc::c_void as *const _;

    let res = unsafe {
        ffi::LIBMTP_Send_File_From_File_Descriptor(
            mtpdev.inner,
            descriptor.as_raw_fd(),
            file_t,
            Some(progress_func_handler),
            callback,
        )
    };

    if res != 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok(File {
            inner: file_t,
            owner: mtpdev,
        })
    }
}

pub(crate) fn send_file_from_handler<'a, H>(
    mtpdev: &'a MtpDevice,
    storage_id: u32,
    parent: Parent,
    metadata: FileMetadata<'_>,
    mut handler: H,
) -> Result<File<'a>>
where
    H: FnMut(&mut [u8]) -> HandlerReturn,
{
    let handler: &mut dyn FnMut(&mut [u8]) -> HandlerReturn = &mut handler;
    let mut handler_return = HandlerReturn::Ok(0);

    let private = &mut (&mut handler_return, handler) as *mut _ as *mut libc::c_void;

    let file_t = unsafe { ffi::LIBMTP_new_file_t() };
    unsafe { fill_file_t!(metadata, parent.to_id(), storage_id, file_t) };

    let res = unsafe {
        ffi::LIBMTP_Send_File_From_Handler(
            mtpdev.inner,
            Some(data_get_func_handler),
            private,
            file_t,
            None,
            std::ptr::null(),
        )
    };

    if res != 0 && handler_return.is_error() {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        if handler_return.is_cancel() {
            let _ = mtpdev.latest_error();
        }

        Ok(File {
            inner: file_t,
            owner: mtpdev,
        })
    }
}

pub(crate) fn send_file_from_handler_with_callback<'a, H, C>(
    mtpdev: &'a MtpDevice,
    storage_id: u32,
    parent: Parent,
    metadata: FileMetadata<'_>,
    mut handler: H,
    mut callback: C,
) -> Result<File<'a>>
where
    H: FnMut(&mut [u8]) -> HandlerReturn,
    C: FnMut(u64, u64) -> CallbackReturn,
{
    let handler: &mut dyn FnMut(&mut [u8]) -> HandlerReturn = &mut handler;
    let mut handler_return = HandlerReturn::Ok(0);

    let private = &mut (&mut handler_return, handler) as *mut _ as *mut libc::c_void;

    let file_t = unsafe { ffi::LIBMTP_new_file_t() };
    unsafe { fill_file_t!(metadata, parent.to_id(), storage_id, file_t) };

    let mut callback: &mut dyn FnMut(u64, u64) -> CallbackReturn = &mut callback;
    let callback = &mut callback as *mut _ as *mut libc::c_void as *const _;

    let res = unsafe {
        ffi::LIBMTP_Send_File_From_Handler(
            mtpdev.inner,
            Some(data_get_func_handler),
            private,
            file_t,
            Some(progress_func_handler),
            callback,
        )
    };

    if res != 0 && handler_return.is_error() {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        if handler_return.is_cancel() {
            let _ = mtpdev.latest_error();
        }

        Ok(File {
            inner: file_t,
            owner: mtpdev,
        })
    }
}
