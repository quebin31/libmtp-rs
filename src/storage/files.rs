use chrono::{DateTime, TimeZone, Utc};
use libmtp_sys as ffi;
use num_traits::FromPrimitive;
use std::{
    ffi::CStr,
    fmt::{self, Debug},
    path::Path,
};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

use crate::{
    device::MtpDevice,
    object::{filetypes::Filetype, AsObjectId, Object},
    util::data_get_func_handler,
    util::data_put_func_handler,
    util::progress_func_handler,
    util::HandlerReturn,
    Result,
};

use super::Parent;

/// Handler of a file object (file or folder).
/// It implements `Identifiable` since it's an object.
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

    /// Returns the type of this file.
    pub fn file_type(&self) -> Filetype {
        let ftype = unsafe { (*self.inner).filetype };
        Filetype::from_u32(ftype).expect("Unexpected raw variant of Filetype")
    }

    /// Returns the latest modification date in UTC.
    pub fn modification_date(&self) -> DateTime<Utc> {
        let epoch = unsafe { (*self.inner).modificationdate };
        Utc.timestamp(epoch, 0)
    }

    /// Rename this file in-place.
    pub fn rename(&mut self, new_name: &str) -> Result<()> {
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

/// Used as a parameter to send local files to the device.
pub struct FileMetadata<'a> {
    pub file_size: u64,
    pub file_name: &'a str,
    pub file_type: Filetype,
    pub modification_date: DateTime<Utc>,
}

pub(crate) fn get_file_to_path<C>(
    mtpdev: &MtpDevice,
    file: impl AsObjectId,
    path: impl AsRef<Path>,
    callback: Option<C>,
) -> Result<()>
where
    C: FnMut(u64, u64) -> bool,
{
    let path = path.as_ref();
    let path = path_to_cvec!(path);

    let res = if let Some(mut callback) = callback {
        let mut callback: &mut dyn FnMut(u64, u64) -> bool = &mut callback;
        let callback = &mut callback;
        let callback = callback as *mut _ as *mut libc::c_void as *const _;

        unsafe {
            ffi::LIBMTP_Get_File_To_File(
                mtpdev.inner,
                file.as_id(),
                path.as_ptr() as *const _,
                Some(progress_func_handler),
                callback,
            )
        }
    } else {
        unsafe {
            ffi::LIBMTP_Get_File_To_File(
                mtpdev.inner,
                file.as_id(),
                path.as_ptr() as *const _,
                None,
                std::ptr::null(),
            )
        }
    };

    if res != 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
pub(crate) fn get_file_to_descriptor<C>(
    mtpdev: &MtpDevice,
    file: impl AsObjectId,
    descriptor: impl AsRawFd,
    callback: Option<C>,
) -> Result<()>
where
    C: FnMut(u64, u64) -> bool,
{
    let res = if let Some(mut callback) = callback {
        let mut callback: &mut dyn FnMut(u64, u64) -> bool = &mut callback;
        let callback = &mut callback;
        let callback = callback as *mut _ as *mut libc::c_void as *const _;

        unsafe {
            ffi::LIBMTP_Get_File_To_File_Descriptor(
                mtpdev.inner,
                file.as_id(),
                descriptor.as_raw_fd(),
                Some(progress_func_handler),
                callback,
            )
        }
    } else {
        unsafe {
            ffi::LIBMTP_Get_File_To_File_Descriptor(
                mtpdev.inner,
                file.as_id(),
                descriptor.as_raw_fd(),
                None,
                std::ptr::null(),
            )
        }
    };

    if res != 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok(())
    }
}

pub(crate) fn get_file_to_handler<H, C>(
    mtpdev: &MtpDevice,
    file: impl AsObjectId,
    handler: H,
    callback: Option<C>,
) -> Result<()>
where
    H: FnMut(&[u8], &mut u32) -> HandlerReturn,
    C: FnMut(u64, u64) -> bool,
{
    let mut handler = handler;
    let mut handler: &mut dyn FnMut(&[u8], &mut u32) -> HandlerReturn = &mut handler;
    let handler = &mut handler;
    let handler = handler as *mut _ as *mut libc::c_void;

    let res = if let Some(mut callback) = callback {
        let mut callback: &mut dyn FnMut(u64, u64) -> bool = &mut callback;
        let callback = &mut callback;
        let callback = callback as *mut _ as *mut libc::c_void as *const _;

        unsafe {
            ffi::LIBMTP_Get_File_To_Handler(
                mtpdev.inner,
                file.as_id(),
                Some(data_put_func_handler),
                handler,
                Some(progress_func_handler),
                callback,
            )
        }
    } else {
        unsafe {
            ffi::LIBMTP_Get_File_To_Handler(
                mtpdev.inner,
                file.as_id(),
                Some(data_put_func_handler),
                handler,
                None,
                std::ptr::null(),
            )
        }
    };

    if res != 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok(())
    }
}

pub(crate) fn send_file_from_path<'a, C>(
    mtpdev: &'a MtpDevice,
    storage_id: u32,
    path: impl AsRef<Path>,
    parent: Parent,
    metadata: FileMetadata<'_>,
    callback: Option<C>,
) -> Result<File<'a>>
where
    C: FnMut(u64, u64) -> bool,
{
    let path = path.as_ref();
    let path = path_to_cvec!(path);

    let file_t = unsafe { ffi::LIBMTP_new_file_t() };
    unsafe { fill_file_t!(metadata, parent.to_id(), storage_id, file_t) };

    let res = if let Some(mut callback) = callback {
        let mut callback: &mut dyn FnMut(u64, u64) -> bool = &mut callback;
        let callback = &mut callback;
        let callback = callback as *mut _ as *mut libc::c_void as *const _;

        unsafe {
            ffi::LIBMTP_Send_File_From_File(
                mtpdev.inner,
                path.as_ptr() as *const _,
                file_t,
                Some(progress_func_handler),
                callback,
            )
        }
    } else {
        unsafe {
            ffi::LIBMTP_Send_File_From_File(
                mtpdev.inner,
                path.as_ptr() as *const _,
                file_t,
                None,
                std::ptr::null(),
            )
        }
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
pub(crate) fn send_file_from_descriptor<'a, C>(
    mtpdev: &'a MtpDevice,
    storage_id: u32,
    descriptor: impl AsRawFd,
    parent: Parent,
    metadata: FileMetadata<'_>,
    callback: Option<C>,
) -> Result<File<'a>>
where
    C: FnMut(u64, u64) -> bool,
{
    let file_t = unsafe { ffi::LIBMTP_new_file_t() };
    unsafe { fill_file_t!(metadata, parent.to_id(), storage_id, file_t) };

    let res = if let Some(mut callback) = callback {
        let mut callback: &mut dyn FnMut(u64, u64) -> bool = &mut callback;
        let callback = &mut callback;
        let callback = callback as *mut _ as *mut libc::c_void as *const _;

        unsafe {
            ffi::LIBMTP_Send_File_From_File_Descriptor(
                mtpdev.inner,
                descriptor.as_raw_fd(),
                file_t,
                Some(progress_func_handler),
                callback,
            )
        }
    } else {
        unsafe {
            ffi::LIBMTP_Send_File_From_File_Descriptor(
                mtpdev.inner,
                descriptor.as_raw_fd(),
                file_t,
                None,
                std::ptr::null(),
            )
        }
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

pub(crate) fn send_file_from_handler<'a, H, C>(
    mtpdev: &'a MtpDevice,
    storage_id: u32,
    handler: H,
    parent: Parent,
    metadata: FileMetadata<'_>,
    callback: Option<C>,
) -> Result<File<'a>>
where
    H: FnMut(&mut [u8], &mut u32) -> HandlerReturn,
    C: FnMut(u64, u64) -> bool,
{
    let mut handler = handler;
    let mut handler: &mut dyn FnMut(&mut [u8], &mut u32) -> HandlerReturn = &mut handler;
    let handler = &mut handler;
    let handler = handler as *mut _ as *mut libc::c_void;

    let file_t = unsafe { ffi::LIBMTP_new_file_t() };
    unsafe { fill_file_t!(metadata, parent.to_id(), storage_id, file_t) };

    let res = if let Some(mut callback) = callback {
        let mut callback: &mut dyn FnMut(u64, u64) -> bool = &mut callback;
        let callback = &mut callback;
        let callback = callback as *mut _ as *mut libc::c_void as *const _;

        unsafe {
            ffi::LIBMTP_Send_File_From_Handler(
                mtpdev.inner,
                Some(data_get_func_handler),
                handler,
                file_t,
                Some(progress_func_handler),
                callback,
            )
        }
    } else {
        unsafe {
            ffi::LIBMTP_Send_File_From_Handler(
                mtpdev.inner,
                Some(data_get_func_handler),
                handler,
                file_t,
                None,
                std::ptr::null(),
            )
        }
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
