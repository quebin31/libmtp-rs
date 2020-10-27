//! Contains relevant items to handle folder objects in the device.

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::fmt::{self, Debug};

use libmtp_sys as ffi;

use crate::device::MtpDevice;
use crate::object::Object;
use crate::storage::Parent;
use crate::Result;

pub struct Folder<'a> {
    inner: *mut ffi::LIBMTP_folder_t,
    owner: &'a MtpDevice,

    sibling_or_child: bool,
}

impl Drop for Folder<'_> {
    fn drop(&mut self) {
        // (Recursively) destroy this folder only if this one was the
        // first folder gathered
        if !self.sibling_or_child {
            unsafe {
                ffi::LIBMTP_destroy_folder_t(self.inner);
            }
        }
    }
}

impl Object for Folder<'_> {
    fn id(&self) -> u32 {
        unsafe { (*self.inner).folder_id }
    }

    fn device(&self) -> &MtpDevice {
        self.owner
    }
}

impl Object for &Folder<'_> {
    fn id(&self) -> u32 {
        unsafe { (*self.inner).folder_id }
    }

    fn device(&self) -> &MtpDevice {
        self.owner
    }
}

impl Debug for Folder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Folder")
            .field("parent_id", &self.parent_id())
            .field("name", &self.name())
            .finish()
    }
}

impl<'a> Folder<'a> {
    pub fn parent_id(&self) -> u32 {
        unsafe { (*self.inner).parent_id }
    }

    pub fn name(&self) -> &str {
        unsafe {
            let cstr = CStr::from_ptr((*self.inner).name);
            cstr.to_str().expect("Invalid UTF-8 on folder name")
        }
    }

    pub fn sibling(&self) -> Option<Folder<'a>> {
        unsafe {
            if (*self.inner).sibling.is_null() {
                None
            } else {
                Some(Folder {
                    inner: (*self.inner).sibling,
                    owner: self.owner,
                    sibling_or_child: true,
                })
            }
        }
    }

    pub fn child(&self) -> Option<Folder<'a>> {
        unsafe {
            if (*self.inner).child.is_null() {
                None
            } else {
                Some(Folder {
                    inner: (*self.inner).child,
                    owner: self.owner,
                    sibling_or_child: true,
                })
            }
        }
    }

    pub fn find(&self, folder_id: u32) -> Option<Folder<'a>> {
        let folder = unsafe { ffi::LIBMTP_Find_Folder(self.inner, folder_id) };

        if folder.is_null() {
            None
        } else {
            Some(Folder {
                inner: folder,
                owner: self.owner,
                sibling_or_child: true,
            })
        }
    }

    pub fn rename(&mut self, new_name: &str) -> Result<()> {
        let new_name = CString::new(new_name).expect("Nul byte");

        let res =
            unsafe { ffi::LIBMTP_Set_Folder_Name(self.owner.inner, self.inner, new_name.as_ptr()) };

        if res != 0 {
            Err(self.owner.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }
}

pub(crate) fn get_folder_list(mtpdev: &MtpDevice) -> Option<Folder<'_>> {
    let folder = unsafe { ffi::LIBMTP_Get_Folder_List(mtpdev.inner) };

    if folder.is_null() {
        None
    } else {
        Some(Folder {
            inner: folder,
            owner: mtpdev,
            sibling_or_child: false,
        })
    }
}

pub(crate) fn get_folder_list_storage(mtpdev: &MtpDevice, storage_id: u32) -> Option<Folder<'_>> {
    let folder = unsafe { ffi::LIBMTP_Get_Folder_List_For_Storage(mtpdev.inner, storage_id) };

    if folder.is_null() {
        None
    } else {
        Some(Folder {
            inner: folder,
            owner: mtpdev,
            sibling_or_child: false,
        })
    }
}

pub(crate) fn create_folder<'a>(
    mtpdev: &MtpDevice,
    name: &'a str,
    parent: Parent,
    storage_id: u32,
) -> Result<(u32, Cow<'a, str>)> {
    let name_cstr = CString::new(name).expect("Nul byte");
    let parent = parent.faf_id();

    let name_in_c = unsafe { libc::strdup(name_cstr.as_ptr()) };
    let folder_id =
        unsafe { ffi::LIBMTP_Create_Folder(mtpdev.inner, name_in_c, parent, storage_id) };

    let name_from_c = unsafe { CStr::from_ptr(name_in_c) };
    let name_from_c = name_from_c.to_str().expect("Invalid UTF-8");

    let name = if name_from_c == name {
        Cow::Borrowed(name)
    } else {
        Cow::Owned(name_from_c.to_string())
    };

    unsafe {
        // Starting from here `name_from_c` is INVALID!  Note that `name` is perfecly
        // valid since it borrows original `name` or creates a new Rust `String`from the
        // contents of `name_from_c` (before it was invalidated)
        libc::free(name_in_c as *mut _);
    }

    if folder_id == 0 {
        Err(mtpdev.latest_error().unwrap_or_default())
    } else {
        Ok((folder_id, name))
    }
}
