use libmtp_sys as ffi;

pub fn init() {
    unsafe {
        ffi::LIBMTP_Init();
    }
}
