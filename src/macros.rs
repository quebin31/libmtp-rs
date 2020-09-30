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

macro_rules! fill_file_t {
    ($filemetadata:expr, $parent:expr, $storage:expr, $file:ident) => {{
        use num_traits::ToPrimitive;
        use std::ffi::CString;

        let file_t = $file;
        let metadata = $filemetadata;

        (*file_t).parent_id = $parent;
        (*file_t).storage_id = $storage;
        (*file_t).filesize = metadata.file_size;
        (*file_t).filetype = metadata
            .file_type
            .to_u32()
            .expect("Unexpected variant in Filetype");
        (*file_t).modificationdate = metadata.modification_date.timestamp() as libc::time_t;

        let filename = CString::new(metadata.file_name).unwrap();
        libc::strcpy((*file_t).filename, filename.as_c_str().as_ptr());
    }};
}
