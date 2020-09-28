/// Group of things related to file types.
pub mod filetypes;

/// Group of things related to properties.
pub mod properties;

use super::device::MtpDevice;

use crate::{error::Error, util::Identifiable};

use libmtp_sys as ffi;
use num_traits::ToPrimitive;
use properties::Property;

/// Object handler, allows to get and set values on properties, this handler
/// is tied with an MTP device.
#[derive(Debug, Copy, Clone)]
pub struct Object<'a> {
    id: u32,
    owner: &'a MtpDevice,
}

impl Identifiable for Object<'_> {
    type Id = u32;

    fn id(&self) -> Self::Id {
        self.id
    }
}

impl<'a> Object<'a> {
    /// Retrieves a string from an object attribute, may fail.
    pub fn get_string(&self, property: Property) -> Result<String, Error> {
        let property = property.to_u32().expect("Unexpected Property variant");
        unsafe {
            let string = ffi::LIBMTP_Get_String_From_Object(self.owner.inner, self.id, property);

            if string.is_null() {
                Err(self.owner.latest_error().unwrap_or_default())
            } else {
                let u8vec = cstr_to_u8vec!(string);
                libc::free(string as *mut _);
                Ok(String::from_utf8(u8vec)?)
            }
        }
    }

    /// Sets an object attribute from a string, may fail.
    pub fn set_string(&self, property: Property, string: &str) -> Result<(), Error> {
        let property = property.to_u32().expect("Unexpected Property variant");
        unsafe {
            let res = ffi::LIBMTP_Set_Object_String(
                self.owner.inner,
                self.id,
                property,
                string.as_ptr() as *const _,
            );

            if res != 0 {
                Err(self.owner.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }

    /// Retrieves an `u64` from an object attribute, may fail.
    pub fn get_u64(&self, property: Property) -> Result<u64, Error> {
        let property = property.to_u32().expect("Unexpected Property variant");
        unsafe {
            let val = ffi::LIBMTP_Get_u64_From_Object(self.owner.inner, self.id, property, 0);

            if let Some(err) = self.owner.latest_error() {
                Err(err)
            } else {
                Ok(val)
            }
        }
    }

    /// Retrieves an `u32` from an object attribute, returns the value of `default` on failure.
    pub fn get_u32(&self, property: Property) -> Result<u32, Error> {
        let property = property.to_u32().expect("Unexpected Property variant");
        unsafe {
            let val = ffi::LIBMTP_Get_u32_From_Object(self.owner.inner, self.id, property, 0);

            if let Some(err) = self.owner.latest_error() {
                Err(err)
            } else {
                Ok(val)
            }
        }
    }

    /// Sets an object attribute from an `u32`, may fail.
    pub fn set_u32(&self, property: Property, value: u32) -> Result<(), Error> {
        let property = property.to_u32().expect("Unexpected Property variant");
        unsafe {
            let res = ffi::LIBMTP_Set_Object_u32(self.owner.inner, self.id, property, value);

            if res != 0 {
                Err(self.owner.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }

    /// Retrieves an `u16` from an object attribute, returns the value of `default` on failure.
    pub fn get_u16(&self, property: Property) -> Result<u16, Error> {
        let property = property.to_u32().expect("Unexpected Property variant");
        unsafe {
            let val = ffi::LIBMTP_Get_u16_From_Object(self.owner.inner, self.id, property, 0);

            if let Some(err) = self.owner.latest_error() {
                Err(err)
            } else {
                Ok(val)
            }
        }
    }

    /// Sets an object attribute from an `u16`, may fail.
    pub fn set_u16(&self, property: Property, value: u16) -> Result<(), Error> {
        let property = property.to_u32().expect("Unexpected Property variant");
        unsafe {
            let res = ffi::LIBMTP_Set_Object_u16(self.owner.inner, self.id, property, value);

            if res != 0 {
                Err(self.owner.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }

    /// Retrieves an `u8` from an object attribute, returns the value of `default` on failure.
    pub fn get_u8(&self, property: Property) -> Result<u8, Error> {
        let property = property.to_u32().expect("Unexpected Property variant");
        unsafe {
            let val = ffi::LIBMTP_Get_u8_From_Object(self.owner.inner, self.id, property, 0);

            if let Some(err) = self.owner.latest_error() {
                Err(err)
            } else {
                Ok(val)
            }
        }
    }

    /// Sets an object attribute from an `u8`, may fail.
    pub fn set_u8(&self, property: Property, value: u8) -> Result<(), Error> {
        let property = property.to_u32().expect("Unexpected Property variant");
        unsafe {
            let res = ffi::LIBMTP_Set_Object_u8(self.owner.inner, self.id, property, value);

            if res != 0 {
                Err(self.owner.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }
}
