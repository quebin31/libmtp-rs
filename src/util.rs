//! Utilities that doesn't fit anywhere else, mostly contains internal crate functions
//! (which are not public) and other useful public items.

use libmtp_sys as ffi;

/// Must return type on callbacks (send and get files)
#[derive(Debug, Copy, Clone)]
pub enum CallbackReturn {
    /// Return this to continue the operation.
    Continue,
    /// Return this to cancel the operation.
    Cancel,
}

#[allow(clippy::transmute_ptr_to_ref)]
pub(crate) unsafe extern "C" fn progress_func_handler(
    sent: u64,
    total: u64,
    data: *const libc::c_void,
) -> libc::c_int {
    let closure: &mut &mut dyn FnMut(u64, u64) -> CallbackReturn = std::mem::transmute(data);
    match closure(sent, total) {
        CallbackReturn::Continue => 0,
        CallbackReturn::Cancel => 1,
    }
}

/// Must return type of send and getter handlers that deal with raw bytes.
#[derive(Debug, Copy, Clone)]
pub enum HandlerReturn {
    /// Return this if every went ok together with how many bytes
    /// you read or writed.
    Ok(u32),

    /// Return this if there was an error.
    Error,

    /// Return this if you want to cancel the operation earlier.
    Cancel,
}

impl HandlerReturn {
    pub(crate) fn is_error(&self) -> bool {
        matches!(&self, HandlerReturn::Error)
    }

    pub(crate) fn is_cancel(&self) -> bool {
        matches!(&self, HandlerReturn::Cancel)
    }
}

#[allow(clippy::transmute_ptr_to_ref)]
pub(crate) unsafe extern "C" fn data_put_func_handler(
    _params: *mut libc::c_void,
    private: *mut libc::c_void,
    sendlen: u32,
    data: *mut libc::c_uchar,
    putlen: *mut u32,
) -> u16 {
    let (handler_return, closure): &mut (
        &mut HandlerReturn,
        &mut dyn FnMut(&[u8]) -> HandlerReturn,
    ) = std::mem::transmute(private);

    let data = prim_array_ptr_to_vec!(data, u8, sendlen);

    **handler_return = closure(&data);
    let ret = match **handler_return {
        HandlerReturn::Ok(len) => {
            // Shouldn't be null
            *putlen = len;

            ffi::LIBMTP_HANDLER_RETURN_OK
        }

        HandlerReturn::Error => ffi::LIBMTP_HANDLER_RETURN_ERROR,
        HandlerReturn::Cancel => ffi::LIBMTP_HANDLER_RETURN_CANCEL,
    };

    ret as u16
}

#[allow(clippy::transmute_ptr_to_ref)]
pub(crate) unsafe extern "C" fn data_get_func_handler(
    _params: *mut libc::c_void,
    private: *mut libc::c_void,
    wantlen: u32,
    data: *mut libc::c_uchar,
    gotlen: *mut u32,
) -> u16 {
    let (handler_return, closure): &mut (
        &mut HandlerReturn,
        &mut dyn FnMut(&mut [u8]) -> HandlerReturn,
    ) = std::mem::transmute(private);

    let mut rsdata = vec![0u8; wantlen as usize];

    **handler_return = closure(&mut rsdata);
    let ret = match **handler_return {
        HandlerReturn::Ok(len) => {
            // Shouldn't be null
            *gotlen = len;

            libc::memcpy(
                data as *mut _,
                rsdata.as_ptr() as *const _,
                wantlen as usize,
            );

            ffi::LIBMTP_HANDLER_RETURN_OK
        }

        HandlerReturn::Error => ffi::LIBMTP_HANDLER_RETURN_ERROR,
        HandlerReturn::Cancel => ffi::LIBMTP_HANDLER_RETURN_CANCEL,
    };

    ret as u16
}
