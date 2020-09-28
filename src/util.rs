use num_derive::ToPrimitive;
use num_traits::ToPrimitive;

#[allow(clippy::transmute_ptr_to_ref)]
pub(crate) unsafe extern "C" fn progress_func_handler(
    sent: u64,
    total: u64,
    data: *const libc::c_void,
) -> libc::c_int {
    let closure: &mut &mut dyn FnMut(u64, u64) -> bool = std::mem::transmute(data);
    if closure(sent, total) {
        0
    } else {
        1
    }
}

#[derive(Debug, Copy, Clone, ToPrimitive)]
pub enum HandlerReturn {
    Ok = 0,
    Error,
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

pub trait Identifiable {
    type Id;

    fn id(&self) -> Self::Id;
}

impl Identifiable for u32 {
    type Id = u32;

    fn id(&self) -> Self::Id {
        *self
    }
}
