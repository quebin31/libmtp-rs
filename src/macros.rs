macro_rules! cstr_to_u8vec {
    ($ptr:expr) => {{
        let mut u8_vec = Vec::new();

        let mut offset = 0;
        let mut ch = *$ptr.offset(offset);
        while ch as u8 != 0x0 {
            u8_vec.push(ch as u8);
            offset += 1;
            ch = *$ptr.offset(offset);
        }

        u8_vec
    }};
}

macro_rules! prim_array_ptr_to_vec {
    ($ptr:expr, $to:ty, $len:expr) => {{
        let mut vec = Vec::new();

        for offset in 0..($len as isize) {
            let item = *$ptr.offset(offset);
            vec.push(item as $to);
        }

        vec
    }};
}

macro_rules! path_to_cvec {
    ($path:expr) => {{
        let mut buf = Vec::new();

        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                use std::iter::once;
                use std::os::windows::ffi::OsStrExt;

                buf.extend($path.as_os_str()
                    .encode_wide()
                    .chain(once(0))
                    .flat_map(|b| {
                        let b = b.to_ne_bytes();
                        once(b[0]).chain(once(b[1]))
                    }));
            } else {
                use std::os::unix::ffi::OsStrExt;

                buf.extend($path.as_os_str().as_bytes());
                buf.push(0);
            }
        }

        buf
    }};
}

