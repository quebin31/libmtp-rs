/// Group of things related to file types.
pub mod filetypes;

/// Group of things related to properties.
pub mod properties;

use super::device::MtpDevice;

use crate::Result;

use libmtp_sys as ffi;
use num_traits::ToPrimitive;
use properties::Property;

pub trait AsObjectId {
    fn as_id(&self) -> u32;
}

impl<T> AsObjectId for T
where
    T: Object,
{
    fn as_id(&self) -> u32 {
        self.id()
    }
}

impl AsObjectId for u32 {
    fn as_id(&self) -> u32 {
        *self
    }
}

pub trait Object {
    fn id(&self) -> u32;
    fn device(&self) -> &MtpDevice;

    /// Retrieves a string from an object attribute.
    fn get_string(&self, property: Property) -> Result<String> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        unsafe {
            let string = ffi::LIBMTP_Get_String_From_Object(device.inner, id, property);

            if string.is_null() {
                Err(device.latest_error().unwrap_or_default())
            } else {
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

        unsafe {
            let res = ffi::LIBMTP_Set_Object_String(
                device.inner,
                id,
                property,
                string.as_ptr() as *const _,
            );

            if res != 0 {
                Err(device.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }

    /// Retrieves an `u64` from an object attribute.
    fn get_u64(&self, property: Property) -> Result<u64> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        unsafe {
            let val = ffi::LIBMTP_Get_u64_From_Object(device.inner, id, property, 0);

            if let Some(err) = device.latest_error() {
                Err(err)
            } else {
                Ok(val)
            }
        }
    }

    /// Retrieves an `u32` from an object attribute, returns the value of `default` on failure.
    fn get_u32(&self, property: Property) -> Result<u32> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        unsafe {
            let val = ffi::LIBMTP_Get_u32_From_Object(device.inner, id, property, 0);

            if let Some(err) = device.latest_error() {
                Err(err)
            } else {
                Ok(val)
            }
        }
    }

    /// Sets an object attribute from an `u32`.
    fn set_u32(&self, property: Property, value: u32) -> Result<()> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        unsafe {
            let res = ffi::LIBMTP_Set_Object_u32(device.inner, id, property, value);

            if res != 0 {
                Err(device.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }

    /// Retrieves an `u16` from an object attribute, returns the value of `default` on failure.
    fn get_u16(&self, property: Property) -> Result<u16> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        unsafe {
            let val = ffi::LIBMTP_Get_u16_From_Object(device.inner, id, property, 0);

            if let Some(err) = device.latest_error() {
                Err(err)
            } else {
                Ok(val)
            }
        }
    }

    /// Sets an object attribute from an `u16`.
    fn set_u16(&self, property: Property, value: u16) -> Result<()> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        unsafe {
            let res = ffi::LIBMTP_Set_Object_u16(device.inner, id, property, value);

            if res != 0 {
                Err(device.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }

    /// Retrieves an `u8` from an object attribute, returns the value of `default` on failure.
    fn get_u8(&self, property: Property) -> Result<u8> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        unsafe {
            let val = ffi::LIBMTP_Get_u8_From_Object(device.inner, id, property, 0);

            if let Some(err) = device.latest_error() {
                Err(err)
            } else {
                Ok(val)
            }
        }
    }

    /// Sets an object attribute from an `u8`.
    fn set_u8(&self, property: Property, value: u8) -> Result<()> {
        let property = property.to_u32().unwrap();
        let id = self.id();
        let device = self.device();

        unsafe {
            let res = ffi::LIBMTP_Set_Object_u8(device.inner, id, property, value);

            if res != 0 {
                Err(device.latest_error().unwrap_or_default())
            } else {
                Ok(())
            }
        }
    }
}
