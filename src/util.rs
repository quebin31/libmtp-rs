//! Utilities that doesn't fit anywhere else, mostly contains internal crate functions
//! (which are not public) and other useful public items.

use num_derive::ToPrimitive;
use num_traits::ToPrimitive;

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
#[derive(Debug, Copy, Clone, ToPrimitive)]
pub enum HandlerReturn {
    /// Return this if every went ok.
    Ok = 0,

    /// Return this if there was an error.
    Error,

    /// Return this if you want to cancel the operation earlier.
    Cancel,
}

#[allow(clippy::transmute_ptr_to_ref)]
pub(crate) unsafe extern "C" fn data_put_func_handler(
    _params: *mut libc::c_void,
    priv_: *mut libc::c_void,
    sendlen: u32,
    data: *mut libc::c_uchar,
    putlen: *mut u32,
) -> u16 {
    let closure: &mut &mut dyn FnMut(&[u8], &mut u32) -> HandlerReturn = std::mem::transmute(priv_);
    let putlen: &mut u32 = std::mem::transmute(putlen);
    let data = prim_array_ptr_to_vec!(data, u8, sendlen);

    closure(&data, putlen)
        .to_u16()
        .expect("Unexpected variant in HandlerReturn")
}

#[allow(clippy::transmute_ptr_to_ref)]
pub(crate) unsafe extern "C" fn data_get_func_handler(
    _params: *mut libc::c_void,
    priv_: *mut libc::c_void,
    wantlen: u32,
    data: *mut libc::c_uchar,
    gotlen: *mut u32,
) -> u16 {
    let closure: &mut &mut dyn FnMut(&mut [u8], &mut u32) -> HandlerReturn =
        std::mem::transmute(priv_);

    let mut rsdata = vec![0 as u8; wantlen as usize];
    let gotlen: &mut u32 = std::mem::transmute(gotlen);

    let ret = closure(&mut rsdata, gotlen)
        .to_u16()
        .expect("Unexpected variant in HandlerReturn");

    libc::memcpy(
        data as *mut _,
        rsdata.as_ptr() as *const _,
        wantlen as usize,
    );

    ret
}
