//! Everything on the Media Transfer Protocol is an *object*, this module groups common behavior
//! and items of many higher abstractions like files, tracks, albums, etc.
//!
//! Note that most operations on attributes should be managed with other APIs exposed in this
//! crate, the most useful utilities here serve to delete, move and copy objects (`Object` trait).

pub mod filetypes;
pub mod properties;

use std::ffi::CString;

use crate::device::MtpDevice;
use crate::storage::Parent;
use crate::Result;

use libmtp_sys as ffi;
use num_traits::ToPrimitive;
use properties::Property;

/// Trait to allow the usage of certain structures or plain `u32` in places where an object id is
/// required. By default every `Object` implementor automagically implements this trait.
///
/// Beware that although some functions accept any `AsObjectId` implementor, this isn't going to be
/// always correct, because some operations are made to work only on certain types of objects (like
/// files, tracks, folders, etc). Also note that using plain `u32` is dangerous, unless you know
/// what you are doing.
pub trait AsObjectId {
    /// Treat the implementor as an object id.
    fn as_id(&self) -> u32;
}

/// All [`Object`](trait.Object.html) implementors can be treated as an object id given that they already
/// have the [`Object::id`](trait.Object.html#tymethod.id) method.
impl<T> AsObjectId for T
where
    T: Object,
{
    fn as_id(&self) -> u32 {
        self.id()
    }
}

/// Note that this is just a convenience implementaion in case you have *known valid* object id as
/// `u32` somewhere else, or you just want to use the [`Object::id`](trait.Object.html#tymethod.id)
/// method to pass the plain `u32`.
impl AsObjectId for u32 {
    fn as_id(&self) -> u32 {
        *self
    }
}

/// Common behavior of many higher abstractions is grouped in this trait, basically everything on
/// MTP is an object with some attributes, even though this API is exposed, it's not recommended to
/// use it to modify or get attributes that can be managed with other specefic APIs (like files,
/// folders, tracks, etc).
pub trait Object {
    /// Must return the id of the object.
    fn id(&self) -> u32;

    /// Must return a valid reference of an `MtpDevice`, where this object resides in.
    fn device(&self) -> &MtpDevice;

    /// Retrieves a string from an object attribute.
    fn get_string(&self, property: Property) -> Result<String> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        let string = unsafe { ffi::LIBMTP_Get_String_From_Object(device.inner, id, property) };

        if string.is_null() {
            Err(device.latest_error().unwrap_or_default())
        } else {
            unsafe {
                let u8vec = cstr_to_u8vec!(string);
                libc::free(string as *mut _);
                Ok(String::from_utf8(u8vec)?)
            }
        }
    }

    /// Sets an object attribute from a string.
    fn set_string(&self, property: Property, string: &str) -> Result<()> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();
        let string = CString::new(string).expect("Nul byte");

        let res =
            unsafe { ffi::LIBMTP_Set_Object_String(device.inner, id, property, string.as_ptr()) };

        if res != 0 {
            Err(device.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }

    /// Retrieves an `u64` from an object attribute.
    fn get_u64(&self, property: Property) -> Result<u64> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        let val = unsafe { ffi::LIBMTP_Get_u64_From_Object(device.inner, id, property, 0) };

        if let Some(err) = device.latest_error() {
            Err(err)
        } else {
            Ok(val)
        }
    }

    /// Retrieves an `u32` from an object attribute, returns the value of `default` on failure.
    fn get_u32(&self, property: Property) -> Result<u32> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        let val = unsafe { ffi::LIBMTP_Get_u32_From_Object(device.inner, id, property, 0) };

        if let Some(err) = device.latest_error() {
            Err(err)
        } else {
            Ok(val)
        }
    }

    /// Sets an object attribute from an `u32`.
    fn set_u32(&self, property: Property, value: u32) -> Result<()> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        let res = unsafe { ffi::LIBMTP_Set_Object_u32(device.inner, id, property, value) };

        if res != 0 {
            Err(device.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }

    /// Retrieves an `u16` from an object attribute, returns the value of `default` on failure.
    fn get_u16(&self, property: Property) -> Result<u16> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        let val = unsafe { ffi::LIBMTP_Get_u16_From_Object(device.inner, id, property, 0) };

        if let Some(err) = device.latest_error() {
            Err(err)
        } else {
            Ok(val)
        }
    }

    /// Sets an object attribute from an `u16`.
    fn set_u16(&self, property: Property, value: u16) -> Result<()> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        let res = unsafe { ffi::LIBMTP_Set_Object_u16(device.inner, id, property, value) };

        if res != 0 {
            Err(device.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }

    /// Retrieves an `u8` from an object attribute, returns the value of `default` on failure.
    fn get_u8(&self, property: Property) -> Result<u8> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        let val = unsafe { ffi::LIBMTP_Get_u8_From_Object(device.inner, id, property, 0) };

        if let Some(err) = device.latest_error() {
            Err(err)
        } else {
            Ok(val)
        }
    }

    /// Sets an object attribute from an `u8`.
    fn set_u8(&self, property: Property, value: u8) -> Result<()> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        let res = unsafe { ffi::LIBMTP_Set_Object_u8(device.inner, id, property, value) };

        if res != 0 {
            Err(device.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }

    /// Deletes a *single* file, track, playlist, folder or any other object off the MTP device.
    /// Note that deleting folders may no be remove its contents, in turn this is the expected
    /// behavior.
    ///
    /// If you want to delete a folder first recursively delete all files and folders contained in
    /// this folder, then the folder itself. Finally, if the operation is sucessful you should
    /// discard the object given that now it holds an **invalid id**.
    fn delete(&self) -> Result<()> {
        let id = self.id();
        let device = self.device();

        let res = unsafe { ffi::LIBMTP_Delete_Object(device.inner, id) };

        if res != 0 {
            Err(device.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }

    /// Moves the object to the specified storage (by its id) and parent folder. Moving objects
    /// may or not be supported on the device.
    ///
    /// Note that moving an object may take a significant amount of time, particularly if being
    /// moved between storages, MTP doesn't provide any kind of progress mechanism, so the operation
    /// will simply block for the duration.
    fn move_to(&self, storage_id: u32, parent: Parent) -> Result<()> {
        let id = self.id();
        let device = self.device();
        let parent = parent.to_id();

        let res = unsafe { ffi::LIBMTP_Move_Object(device.inner, id, storage_id, parent) };

        if res != 0 {
            Err(device.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }

    /// Copies the object to the specified storage (by its id) and parent folder. Copying objects
    /// may or not be supported on the device.
    ///
    /// Note that copying an object may take a significant amount of time, particularly if being
    /// copied between storages, MTP doesn't provide any kind of progress mechanism, so the
    /// operation will simply block for the duration.
    fn copy_to(&self, storage_id: u32, parent: Parent) -> Result<()> {
        let id = self.id();
        let device = self.device();
        let parent = parent.to_id();

        let res = unsafe { ffi::LIBMTP_Copy_Object(device.inner, id, storage_id, parent) };

        if res != 0 {
            Err(device.latest_error().unwrap_or_default())
        } else {
            Ok(())
        }
    }
}
