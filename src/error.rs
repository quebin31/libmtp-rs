use std::string::FromUtf8Error;

use libmtp_sys as ffi;
use thiserror::Error as DError;

#[derive(Debug, Clone, DError)]
pub enum Error {
    #[error("Unknown error!")]
    Unknown,
    #[error("")]
    General,
    #[error("")]
    PTPLayer,
    #[error("")]
    USBLayer,
    #[error("")]
    MemoryAllocation,
    #[error("")]
    NoDeviceAttached,
    #[error("")]
    StorageFull,
    #[error("")]
    Connecting,
    #[error("")]
    Cancelled,
    #[error("There was an error when converting UTF-8 ({source})")]
    UTF8Error { source: FromUtf8Error },
}

impl Error {
    pub(crate) fn from_code(error_code: ffi::LIBMTP_error_number_t) -> Option<Self> {
        match error_code {
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_NONE => None,
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_GENERAL => Some(Self::General),
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_PTP_LAYER => Some(Self::PTPLayer),
            ffi::LIBMTP_error_number_t_LIBMTP_ERROR_USB_LAYER => Some(Self::USBLayer),
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

impl From<FromUtf8Error> for Error {
    fn from(source: FromUtf8Error) -> Self {
        Self::UTF8Error { source }
    }
}
