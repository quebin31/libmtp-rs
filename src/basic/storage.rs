use super::device::MTPDevice;
use crate::error::ErrorKind;
use libmtp_sys as ffi;

/// Storage descriptor of some MTP device, note that at any time anything can
/// happen with the device and one of these descriptors *may be invalid*.
pub struct Storage<'a> {
    pub(crate) inner: *mut ffi::LIBMTP_devicestorage_t,
    pub(crate) owner: &'a MTPDevice,
}

impl<'a> Storage<'a> {
    /// Formats this storage (if its device supports the operation).
    ///
    /// *WARNING:*  This WILL DELETE ALL DATA from the device, make sure
    /// you've got confirmation from the user before calling this function.
    pub fn format_storage(&self) -> Result<(), ErrorKind> {
        unsafe {
            let res = ffi::LIBMTP_Format_Storage(self.owner.inner, self.inner);

            if let Some(err) = ErrorKind::from_code(res as u32) {
                Err(err)
            } else {
                Ok(())
            }
        }
    }
}
