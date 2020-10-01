//! This module contains information about possible errors, such as internal and `libmtp` errors.

use libmtp_sys as ffi;
use std::string::FromUtf8Error;
use thiserror::Error as ErrorTrait;

/// Enumeration of possible `libmtp` errors, check
/// [`Error::MtpError`](enum.Error.html#variant.MtpError) for more information.
#[derive(Debug, Clone, Copy)]
pub enum MtpErrorKind {
    General,
    PtpLayer,
    UsbLayer,
    MemoryAllocation,
    NoDeviceAttached,
    StorageFull,
    Connecting,
    Cancelled,
}

impl MtpErrorKind {
    pub(crate) fn from_error_number(error_code: ffi::LIBMTP_error_number_t) -> Option<Self> {
        match error_code {
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_NONE => None,
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_GENERAL => Some(Self::General),
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_PTP_LAYER => Some(Self::PtpLayer),
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_USB_LAYER => Some(Self::UsbLayer),
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_MEMORY_ALLOCATION => {
                Some(Self::MemoryAllocation)
            }
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_NO_DEVICE_ATTACHED => {
                Some(Self::NoDeviceAttached)
            }
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_STORAGE_FULL => Some(Self::StorageFull),
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_CONNECTING => Some(Self::Connecting),
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_CANCELLED => Some(Self::Cancelled),
            _ => None,
        }
    }
}

/// Main Error type, containing a possible *unknown* error, an specific `libmtp` error
/// and some other internal errors like invalid UTF-8 in string conversion.
#[derive(Debug, Clone, ErrorTrait)]
pub enum Error {
    /// Unknown error, probably some `libmtp` undocumented error.
    #[error("Unknown error (possibly a libmtp undocumented error)")]
    Unknown,

    /// Specific `libmtp` error, contains the kind of the error and extra information
    /// about what went wrong.
    #[error("Internal libmtp ({kind:?}): {text}")]
    MtpError { kind: MtpErrorKind, text: String },

    /// Internal error when converting strings with invalid UTF-8 encoding.
    #[error("Utf8 error ({source})")]
    Utf8Error { source: FromUtf8Error },
}

impl Default for Error {
    fn default() -> Self {
        Error::Unknown
    }
}

impl Error {
    pub(crate) unsafe fn from_latest_error(mut list: *const ffi::LIBMTP_error_t) -> Option<Self> {
        if list.is_null() {
            None
        } else {
            while !(*list).next.is_null() {
                list = (*list).next;
            }

            let error_t = &*list;

            let kind = MtpErrorKind::from_error_number(error_t.errornumber)?;
            let u8vec = cstr_to_u8vec!(error_t.error_text);
            let text = String::from_utf8_lossy(&u8vec).into_owned();

            Some(Error::MtpError { kind, text })
        }
    }
}

impl From<FromUtf8Error> for Error {
    fn from(source: FromUtf8Error) -> Self {
        Error::Utf8Error { source }
    }
}
